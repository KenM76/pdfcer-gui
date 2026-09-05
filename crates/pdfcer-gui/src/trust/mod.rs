//! # `trust` — where the anchors come from, and the three facts they let this
//! shell state about a signature
//!
//! The shell half of `pdfcer-core`'s `Pass 10.2` / `10.3` / `10.4` / `10.5`.
//! `ENGINE_BACKLOG.md` carried four rows for this — importing an installed
//! Acrobat/Reader trust store, evaluating a signature's trust against those
//! anchors, keeping the choice across sessions, and the deterministic
//! no-network parts of RFC 5280 path validation — and they are one feature.
//! This module is its model: locating a store, reading it, and turning
//! `signature::verify_all_with_trust` into something a panel can render without
//! ever saying more than the engine said.
//!
//! ## ★★★ THE RULE THAT GOVERNS THIS WHOLE MODULE
//!
//! **This is the one place in the product where a wrong answer is worse than no
//! answer.**
//!
//! A UI that says *"trusted"* about a chain it did not really validate is
//! precisely the failure mode `pdfcer-core`'s design exists to prevent. Its
//! `SignatureVerdict` carries **three facts that never collapse into one
//! bool** — `integrity`, `coverage` and `trust` — and every surface built on
//! this module reports them **separately**. There is no composite badge, no
//! green tick, and no arithmetic that turns three answers into one.
//!
//! In particular [`pdfcer_core::signature::Trust::NotChecked`] renders **as
//! itself**: *"not checked"*, never as a soft "no", never as a grey tick, and
//! never omitted. A shell that hid `NotChecked` would be indistinguishable, on
//! screen, from a shell that had checked and found nothing wrong — which is the
//! exact inversion this feature exists to prevent.
//!
//! ## 1. What "importing" means here, and why nothing is copied
//!
//! Adobe's own downloaded trust list lives in `addressbook.acrodata`, a
//! `%PPKLITE-` COS file that `pdfcer_core::trust_store` parses with pdfcer's own
//! COS + X.509 code. No Acrobat automation, no network, read-only.
//!
//! **pdfcer never copies it.** There is no pdfcer-side anchor file, no snapshot,
//! no cached DER on disk. Every evaluation reads the operator's own file as it
//! is at that moment. That is a decision and the argument is
//! `ENGINE_BACKLOG.md`'s own:
//!
//! > an anchor set that silently went stale is worse than one that was never
//! > imported.
//!
//! A snapshot has no way to say how old it is that an operator will ever read.
//! A live read has one that costs nothing: the file's **modification time**,
//! which is Adobe's own record of when it last refreshed AATL/EUTL. So
//! [`Store::modified`] is carried everywhere the anchors are, and every surface
//! that names the store names its date. ★ On the machine this was written on,
//! that date is **2024-05-27** — sixteen months stale — which is exactly the
//! condition a "1,780 anchors imported ✓" badge would have hidden.
//!
//! ## 2. Off by default, and the opt-in is the engine's own setting
//!
//! [`pdfcer_core::settings::AcrobatTrustStore`] is `Off` by default and this
//! shell does not second-guess it. `AtOwnRisk` is the operator's explicit,
//! disclosed opt-in, and the engine's own reasoning is why it is a setting
//! rather than a default: reading Adobe's downloaded file is a local read, and
//! *whether relying on it fits the Adobe Reader licence is the operator's call,
//! not a pdfcer legal determination.*
//!
//! Because it is the **engine's** setting rather than one of this shell's
//! preferences, the same choice governs `pdfcer verify-signatures` at the
//! command line. That is the point: one answer to *"may pdfcer read Acrobat's
//! trust list?"*, in one file, for both front ends.
//!
//! ## 3. Where the store is, and why the path is a preference of ours
//!
//! The engine deliberately does not locate the file — its own module header:
//! *"Locating the file is the shell's job."* [`candidate_paths`] is this
//! shell's list and mirrors the CLI's exactly, so the two front ends look in
//! the same places in the same order.
//!
//! [`crate::app::prefs::Prefs::acrobat_trust_store_path`] overrides it, and it
//! exists for the same reason [`crate::app::prefs::Prefs::acrobat_path`] does
//! (O122): discovery is a list of conventional locations, and a conventional
//! location is wrong the first time somebody's profile is redirected, or their
//! Acrobat is a track this build's list does not name, or their store was
//! handed to them by an administrator. **R9's escape hatch applies**: the
//! *inspect* control is absent when there is no store to inspect, but the path
//! field is visible in Settings whether or not discovery succeeded, because it
//! is the only thing that can fix the case where discovery failed.
//!
//! ## 4. The four states of "what anchors were used", and why four
//!
//! [`Anchors`] has four variants because collapsing any two of them makes this
//! shell say something untrue:
//!
//! | variant | the operator's real situation | what collapsing it would claim |
//! |---|---|---|
//! | [`Anchors::OptedOut`] | pdfcer was told not to look | that it looked and found nothing |
//! | [`Anchors::NoStore`] | it looked and this machine has no store | that the operator declined |
//! | [`Anchors::Unreadable`] | it found one and could not read it | that there was none — hiding a fixable fault |
//! | [`Anchors::Used`] | these anchors, from this file, of this date | — |
//!
//! Only the last one can produce a `Trusted` or an `Untrusted` verdict. The
//! other three all produce `NotChecked`, and each of them words *why* rather
//! than sharing one sentence, because *"you turned it off"* and *"your Acrobat
//! store is corrupt"* are opposite calls to action.
//!
//! ## 5. What is deliberately NOT here
//!
//! **Revocation.** `PathChecks::revocation_checked` is `false` on every verdict
//! this build can produce, because CRL/OCSP need the network `pdfcer-core`
//! never touches (its decision 135). This shell does not fetch either, and the
//! copy in [`crate::text::trust`] says so on every `Trusted` verdict rather than
//! once in a footnote — a disclosure attached to the claim it qualifies is one
//! an operator reads.
//!
//! **A trust decision of our own.** Nothing here interprets the `/Trust`
//! bitfield, promotes an anchor, or has any notion of "trusted enough". The
//! anchors go in, the engine's verdict comes out, and this shell's entire
//! contribution is *which file the anchors came from* and *saying what happened*.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use pdfcer_core::graph::ObjectGraph;
use pdfcer_core::settings::AcrobatTrustStore;
use pdfcer_core::signature::SignatureVerdict;
use pdfcer_core::trust_store::{self, SourceCounts, TrustAnchorSet};

#[cfg(test)]
mod tests;

/// The Acrobat/Reader release tracks whose `Security` directory may hold a
/// downloaded trust list.
///
/// ★ **The same four, in the same order, as `pdfcer-cli`'s
/// `acrobat_trust_store_paths`.** Deliberately mirrored rather than reasoned
/// out again: two front ends of one program that look in different places
/// produce the single most confusing support conversation available — *"the
/// command line finds my store and the window does not"* — and neither answer
/// is wrong on its own terms.
///
/// `DC` first because it is the only track Adobe still ships to; the three
/// older ones are there because an install that stopped being updated still has
/// a store, and a stale store is a thing this module can disclose rather than a
/// thing it must refuse.
const TRACKS: &[&str] = &["DC", "2020", "2017", "11.0"];

/// The file name Acrobat writes its address book into.
const ADDRESS_BOOK: &str = "addressbook.acrodata"; // ui-text-exempt: a file name on the operator's disk, matched literally.

/// Every place an installed Acrobat/Reader may have put its downloaded trust
/// list on this machine, most likely first.
///
/// **Platform-neutral by construction, with no `cfg(windows)`.** `%APPDATA%` is
/// a Windows variable, so on any other target `env::var` simply returns `Err`
/// and this returns an empty list — which every caller already handles, because
/// "no store on this machine" is a real state on Windows too. The CLI takes the
/// identical approach and states the identical reason.
///
/// ★ It reports **candidates**, not findings: nothing here touches the disk.
/// [`locate`] is what asks whether any of them exists, and keeping the two
/// apart is what lets [`Located::None`] report *what was looked at*, which is
/// the only actionable half of "nothing was found".
#[must_use]
pub fn candidate_paths() -> Vec<PathBuf> {
    let Ok(appdata) = std::env::var("APPDATA") else {
        return Vec::new();
    };
    TRACKS
        .iter()
        .map(|track| {
            PathBuf::from(&appdata)
                .join("Adobe")
                .join("Acrobat")
                .join(track)
                .join("Security")
                .join(ADDRESS_BOOK)
        })
        .collect()
}

/// What locating a trust store produced.
///
/// Five states rather than `Option<PathBuf>`, and every extra one exists
/// because it is a different sentence to an operator. In particular
/// [`Self::ConfiguredMissing`] must never be rendered as [`Self::None`]: a
/// person who typed a path and got *"no trust store was found"* is being told
/// their machine has no store, when what actually happened is that they made a
/// typo — and the field they would fix is the one they are looking at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Located {
    /// The operator named a path in Settings and there is a file there.
    ///
    /// A configured path **wins over discovery unconditionally**, exactly as
    /// `crate::acrobat`'s does. Somebody who has taken the trouble to type a
    /// path has said which store they mean.
    Configured(PathBuf),
    /// The operator named a path and nothing is there.
    ///
    /// Not a fallback to discovery. Falling back would make a typo behave like
    /// a correct entry pointing somewhere else, and the operator would have no
    /// way to tell which store was actually being read.
    ConfiguredMissing(PathBuf),
    /// Nothing was configured, and discovery found this one.
    Discovered(PathBuf),
    /// Nothing was configured and none of the candidates exists.
    ///
    /// Carries what was looked at, because *"pdfcer found nothing"* is not
    /// actionable and *"pdfcer looked in these four places"* is.
    None {
        /// The paths [`candidate_paths`] offered, in order.
        looked_in: Vec<PathBuf>,
    },
}

impl Located {
    /// The path to read, when there is one.
    #[must_use]
    pub fn usable(&self) -> Option<&Path> {
        match self {
            Self::Configured(p) | Self::Discovered(p) => Some(p),
            Self::ConfiguredMissing(_) | Self::None { .. } => None,
        }
    }

    /// The path this state is *about*, whether or not it can be read.
    ///
    /// Distinct from [`Self::usable`] on purpose: a missing configured path is
    /// still the path the operator needs to see printed back, and a control
    /// that showed nothing there would be answering a typo with silence.
    #[must_use]
    pub fn named(&self) -> Option<&Path> {
        match self {
            Self::Configured(p) | Self::ConfiguredMissing(p) | Self::Discovered(p) => Some(p),
            Self::None { .. } => None,
        }
    }
}

/// Find the trust store, preferring what the operator configured.
///
/// `configured` is [`crate::app::prefs::Prefs::acrobat_trust_store_path`] as
/// typed. It is trimmed here as well as on the way in, because this file is not
/// the only route a value takes — the same argument `prefs` makes about
/// `acrobat_path`, and the same reason: a trailing space is a path that does
/// not exist and the failure presents as *"the setting does nothing"*.
///
/// **An empty field means "ask this machine", not "no store".** Clearing a text
/// box is how a person un-sets it, and reading a cleared box as a positive
/// choice would suppress the feature with no way back except editing a file by
/// hand.
#[must_use]
pub fn locate(configured: &str) -> Located {
    let configured = configured.trim();
    if !configured.is_empty() {
        let path = PathBuf::from(configured);
        return if path.is_file() {
            Located::Configured(path)
        } else {
            Located::ConfiguredMissing(path)
        };
    }
    let candidates = candidate_paths();
    match candidates.iter().find(|p| p.is_file()) {
        Some(found) => Located::Discovered(found.clone()),
        None => Located::None {
            looked_in: candidates,
        },
    }
}

/// A trust store as read, with the provenance every surface must show beside
/// it.
///
/// ★ [`Self::modified`] is carried in the same struct as the anchors rather
/// than fetched where it is displayed. That is deliberate: the count and the
/// date are one fact — *"1,780 anchors, as Adobe last downloaded them on this
/// date"* — and a surface that could obtain one without the other would
/// eventually show the count alone. The whole argument for reading the store
/// live rather than snapshotting it is that its age stays visible.
#[derive(Debug, Clone)]
pub struct Store {
    /// The file that was read.
    pub path: PathBuf,
    /// Its modification time — Adobe's own record of the last AATL/EUTL
    /// refresh. `None` when the filesystem would not say.
    pub modified: Option<SystemTime>,
    /// How many anchors, by `/Source` provenance.
    pub counts: SourceCounts,
    /// Entries whose certificate the X.509 decoder refused.
    ///
    /// Surfaced rather than swallowed. A store that mostly decoded is still a
    /// usable anchor pool, and an operator whose signer happens to be one of
    /// the refused entries would otherwise see an inexplicable `Untrusted`.
    pub undecodable: usize,
    /// The anchors themselves, for [`pdfcer_core::signature::verify_all_with_trust`].
    pub anchors: TrustAnchorSet,
}

/// Read a trust store from `path`.
///
/// # Errors
///
/// Returns the engine's own error text. It is not re-worded here: the engine
/// names its refusals precisely (`NotAnAddressBook` explains that
/// `directories.acrodata` and `security-policy.acrodata` carry no anchors), and
/// a shell that paraphrased would produce a second, vaguer vocabulary for the
/// same faults.
pub fn load(path: &Path) -> Result<Store, String> {
    // The stat is taken FIRST, before the read, so the date reported belongs to
    // the bytes that were parsed rather than to whatever the file became while
    // a 3 MB parse was running. The window is tiny and the ordering costs
    // nothing; a date that describes different bytes from the counts beside it
    // is exactly the kind of quiet inconsistency this module exists to avoid.
    let modified = std::fs::metadata(path).ok().and_then(|m| m.modified().ok());
    let set = trust_store::load_from_path(path).map_err(|e| e.to_string())?;
    Ok(Store {
        path: path.to_path_buf(),
        modified,
        counts: set.counts(),
        undecodable: set.undecodable,
        anchors: set,
    })
}

/// What anchor pool a report was produced against — and, when there is none,
/// which of the three reasons applies.
///
/// See the module header's table for why there are four variants and what
/// collapsing any two of them would claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Anchors {
    /// `acrobat_trust_store = off`. pdfcer did not look and did not check.
    OptedOut,
    /// The setting is on and this machine has no store.
    NoStore {
        /// The paths that were tried, so the sentence can be actionable.
        looked_in: Vec<PathBuf>,
        /// The operator's configured path, when they set one that is not there.
        ///
        /// `Some` distinguishes *"you pointed at a file that is not there"*
        /// from *"this machine has no Acrobat store"*. They call for opposite
        /// actions and must not share a sentence.
        configured_missing: Option<PathBuf>,
    },
    /// The setting is on, a store was found, and reading it failed.
    Unreadable {
        /// The file that could not be read.
        path: PathBuf,
        /// The engine's own words for why.
        reason: String,
    },
    /// The setting is on and these anchors were used.
    Used {
        /// The file they came from.
        path: PathBuf,
        /// When Adobe last wrote it.
        modified: Option<SystemTime>,
        /// How many, by provenance.
        counts: SourceCounts,
        /// Entries the X.509 decoder refused.
        undecodable: usize,
    },
}

impl Anchors {
    /// Whether trust was actually evaluated.
    ///
    /// ★ Used only to decide which sentence to draw, **never** to decide what a
    /// verdict means. The verdict is the engine's; this predicate says which
    /// explanation of `NotChecked` belongs beside it.
    #[must_use]
    pub const fn evaluated(&self) -> bool {
        matches!(self, Self::Used { .. })
    }
}

/// Everything one examination of a file produced.
///
/// Deliberately a value with no methods that judge. It carries the engine's
/// verdicts verbatim and the provenance of the anchors, and every reading of it
/// happens in [`crate::text::trust`] where the words live.
#[derive(Debug, Clone)]
pub struct Report {
    /// What pool the verdicts were evaluated against.
    pub anchors: Anchors,
    /// One entry per signature field, in `byte_range_coverage` order.
    pub verdicts: Vec<SignatureVerdict>,
    /// The length of the bytes that were verified.
    ///
    /// Kept so a surface can say **which** state of the file it measured. The
    /// Signatures panel already makes this distinction about coverage; a
    /// verdict computed from a file that has since been appended to is a
    /// verdict about a document that no longer exists.
    pub file_len: u64,
}

/// Read a file, resolve the anchor pool, and verify every signature in it.
///
/// # Why this takes bytes AND a graph
///
/// Because `/ByteRange` is a claim about **bytes**, and the object model cannot
/// check a claim about bytes against itself — the engine's own reason for
/// `byte_range_coverage` taking a length rather than deriving one. Verification
/// needs the real file, digested; the graph is only how the signature
/// dictionaries are found.
///
/// ⚠ The bytes must be **the file on disk**, not the session's rendering of it.
/// A signature covers what was written, and an unsaved edit is not in the file.
/// [`examine_path`] is the route that guarantees this; this function is split
/// out so the whole decision table is testable without a filesystem.
#[must_use]
pub fn examine<G: ObjectGraph + ?Sized>(
    graph: &G,
    bytes: &[u8],
    setting: AcrobatTrustStore,
    configured_path: &str,
) -> Report {
    let anchors = resolve_anchors(setting, configured_path);
    // ★ The pool is threaded straight into the engine and never consulted here.
    // `verify_all_with_trust(.., None)` is by the engine's own documentation
    // identical to `verify_all`, so the opted-out path is not a second code
    // path with its own chance of disagreeing — it is the same call with an
    // empty hand.
    let pool = anchors.as_ref().map(|(_, store)| &store.anchors);
    let verdicts = pdfcer_core::signature::verify_all_with_trust(graph, bytes, pool);
    Report {
        anchors: anchors.map_or_else(
            || describe_absence(setting, configured_path),
            |(_, store)| Anchors::Used {
                path: store.path,
                modified: store.modified,
                counts: store.counts,
                undecodable: store.undecodable,
            },
        ),
        verdicts,
        file_len: bytes.len() as u64,
    }
}

/// Load the anchor pool, or nothing.
///
/// Returns the [`Located`] alongside the [`Store`] so the caller does not have
/// to locate twice; the absence path re-locates because it needs the *failed*
/// state, which by definition produced no store.
fn resolve_anchors(setting: AcrobatTrustStore, configured: &str) -> Option<(Located, Store)> {
    if setting != AcrobatTrustStore::AtOwnRisk {
        return None;
    }
    let located = locate(configured);
    let path = located.usable()?;
    let store = load(path).ok()?;
    Some((located, store))
}

/// Which of the three no-anchors states applies.
///
/// Separated from [`resolve_anchors`] because the happy path must not pay for
/// building an explanation, and because an explanation assembled from the same
/// inputs twice is one that can be tested on its own.
fn describe_absence(setting: AcrobatTrustStore, configured: &str) -> Anchors {
    if setting != AcrobatTrustStore::AtOwnRisk {
        return Anchors::OptedOut;
    }
    match locate(configured) {
        Located::None { looked_in } => Anchors::NoStore {
            looked_in,
            configured_missing: None,
        },
        Located::ConfiguredMissing(path) => Anchors::NoStore {
            looked_in: Vec::new(),
            configured_missing: Some(path),
        },
        Located::Configured(path) | Located::Discovered(path) => match load(&path) {
            // Unreachable in practice — `resolve_anchors` only falls through to
            // here when the load failed — but written as the honest answer
            // rather than as an `unreachable!`, because the two calls are
            // separated by a filesystem and a file can be replaced between
            // them. A panic here would be a crash caused by somebody else's
            // antivirus quarantining a file mid-frame.
            Ok(store) => Anchors::Used {
                path: store.path,
                modified: store.modified,
                counts: store.counts,
                undecodable: store.undecodable,
            },
            Err(reason) => Anchors::Unreadable { path, reason },
        },
    }
}

/// [`examine`], reading the file from disk.
///
/// # Errors
///
/// The `std::io::Error` text, when the file cannot be read. There is no
/// verdict in that case and none is invented: a document whose file pdfcer
/// cannot read is not a document whose signatures failed.
pub fn examine_path<G: ObjectGraph + ?Sized>(
    graph: &G,
    path: &Path,
    setting: AcrobatTrustStore,
    configured_path: &str,
) -> Result<Report, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    Ok(examine(graph, &bytes, setting, configured_path))
}

/// A modification time as `YYYY-MM-DD`, UTC.
///
/// ★ Date only, no clock time. The question an operator is answering is *"is
/// this anchor set current?"*, which is a question about weeks and months —
/// AATL refreshes are not a daily event — and a timestamp to the second would
/// invite the reading that the number is precise about something it is not.
///
/// Returns `None` for a time the filesystem could not give, or one before the
/// Unix epoch, rather than substituting today. A store with no readable date is
/// a store whose staleness is unknown, and saying so is the whole point.
#[must_use]
pub fn modified_date(at: SystemTime) -> Option<String> {
    let secs = at.duration_since(UNIX_EPOCH).ok()?.as_secs();
    Some(crate::app::clock::iso_date_utc(secs))
}

// ---------------------------------------------------------------------------
// The frame cache
// ---------------------------------------------------------------------------

/// What a cached [`Report`] was computed from.
///
/// ★★★ **Every input is in the key, and that is the whole safety property.**
///
/// A panel redraws sixty times a second and verification is a SHA-256 over the
/// whole file plus an RSA or ECDSA verify per signature, on top of a 3 MB COS
/// parse of the anchor store. Doing that per frame is not an option; caching it
/// against a key that misses an input is worse than not caching, because the
/// panel then shows a verdict about a file, a setting or an anchor set that is
/// no longer the one in force — and it looks exactly like a correct answer.
///
/// So the key is the file's identity **and** its length **and** its
/// modification time (the two cheap facts that change when a file is written
/// to), plus both halves of the trust configuration. Change any of them and the
/// verdict is recomputed.
///
/// ★ `len` and `modified` together rather than either alone: an incremental
/// save that appends always changes the length, and a same-length rewrite
/// always changes the time. Neither is sufficient on its own and both are one
/// `stat`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CacheKey {
    path: PathBuf,
    len: u64,
    modified: Option<SystemTime>,
    setting: AcrobatTrustStore,
    configured_path: String,
}

/// The `egui` memory slot the cached report lives in.
///
/// # Why `egui::Memory` rather than a field on `PanelsState`
///
/// Because the cache is an implementation detail of **one panel** and of the
/// Settings group beside it, and a field on the shared panel state would make
/// it part of every panel's contract. `dialogs::settings::widgets::text_value`
/// already keeps its per-control edit buffer here for the same reason and says
/// so.
///
/// ★ It is `insert_temp`, so it is never serialised into the layout file. A
/// signature verdict is a measurement of a file at a moment; persisting one
/// across restarts would produce a verdict about a file that may have been
/// replaced while pdfcer was not running, which is precisely the failure the
/// key above exists to prevent — reintroduced by a different door.
fn slot() -> egui::Id {
    egui::Id::new("pdfcer.trust.report") // ui-text-exempt: an egui memory key, never displayed.
}

/// The verdicts for `path`, computed at most once per distinct [`CacheKey`].
///
/// Returns `Err` with the reason the file could not be read — which is a
/// different statement from any verdict and must not be rendered as one.
///
/// ★ **It computes on the first frame it is called on, without being asked.**
/// The alternative considered was a *Check signatures* button. It was refused:
/// an operator who has opened a panel called Signatures has already asked, and
/// a button would leave the panel's default state showing coverage numbers with
/// no integrity beside them — which is the state this whole feature exists to
/// end. The cost is one verification the first time the panel is drawn for a
/// given file; the cache above is what stops it being sixty.
pub fn cached_report(
    ctx: &egui::Context,
    graph: &(impl ObjectGraph + ?Sized),
    path: &Path,
    setting: AcrobatTrustStore,
    configured_path: &str,
) -> Result<Arc<Report>, String> {
    let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
    let key = CacheKey {
        path: path.to_path_buf(),
        len: meta.len(),
        modified: meta.modified().ok(),
        setting,
        configured_path: configured_path.to_owned(),
    };
    let id = slot();
    if let Some((cached_key, report)) = ctx.data(|d| d.get_temp::<(CacheKey, Arc<Report>)>(id))
        && cached_key == key
    {
        return Ok(report);
    }
    let report = Arc::new(examine_path(graph, path, setting, configured_path)?);
    crate::diag::trace(|| {
        format!(
            "trust-report path={:?} len={} signatures={} anchors={}",
            key.path,
            key.len,
            report.verdicts.len(),
            anchor_trace(&report.anchors),
        )
    });
    ctx.data_mut(|d| d.insert_temp(id, (key, Arc::clone(&report))));
    Ok(report)
}

/// The anchor state as one trace token.
///
/// ★ Not an operator string and not in the catalog: it is a diagnostic word a
/// driven check matches on. `ui-verify` asserts on `anchors=off` /
/// `anchors=none` / `anchors=used:N`, and a check that matched translated prose
/// would break the day the prose improved.
fn anchor_trace(anchors: &Anchors) -> String {
    match anchors {
        Anchors::OptedOut => "off".to_owned(),
        Anchors::NoStore { .. } => "none".to_owned(),
        Anchors::Unreadable { .. } => "unreadable".to_owned(),
        Anchors::Used { counts, .. } => format!(
            "used:{} aatl={} eutl={} adbe={} other={}",
            counts.total, counts.aatl, counts.eutl, counts.adbe, counts.other
        ),
    }
}
