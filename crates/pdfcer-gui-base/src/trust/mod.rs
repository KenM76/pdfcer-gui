//! # `trust` — where the anchors come from, and the three facts they let this
//! shell state about a signature
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/trust/mod.md`.

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
/// It reports **candidates**, not findings: nothing here touches the disk.
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
/// `configured` is `pdfcer-gui`'s `app::prefs::Prefs::acrobat_trust_store_path` as
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
/// [`Self::modified`] is carried in the same struct as the anchors rather
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
    /// Used only to decide which sentence to draw, **never** to decide what a
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
/// happens in `pdfcer-gui`'s `text::trust` where the words live.
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
    // The pool is threaded straight into the engine and never consulted here.
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
/// Date only, no clock time. The question an operator is answering is *"is
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
    Some(crate::clock::iso_date_utc(secs))
}

// ---------------------------------------------------------------------------
// The frame cache
// ---------------------------------------------------------------------------

/// What a cached [`Report`] was computed from.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CacheKey {
    path: PathBuf,
    len: u64,
    modified: Option<SystemTime>,
    setting: AcrobatTrustStore,
    configured_path: String,
}

/// The `egui` memory slot the cached report lives in.
fn slot() -> egui::Id {
    egui::Id::new("pdfcer.trust.report") // ui-text-exempt: an egui memory key, never displayed.
}

/// The verdicts for `path`, computed at most once per distinct [`CacheKey`].
///
/// Returns `Err` with the reason the file could not be read — which is a
/// different statement from any verdict and must not be rendered as one.
///
/// **It computes on the first frame it is called on, without being asked.**
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
fn anchor_trace(anchors: &Anchors) -> String {
    match anchors {
        Anchors::OptedOut => "off".to_owned(),
        Anchors::NoStore { .. } => "none".to_owned(),
        Anchors::Unreadable { .. } => "unreadable".to_owned(),
        Anchors::Used { counts, .. } => format!(
            // ui-text-exempt: a trace token a driven check matches on, as the
            // header above argues; translating it would break `ui-verify`.
            "used:{} aatl={} eutl={} adbe={} other={}",
            counts.total, counts.aatl, counts.eutl, counts.adbe, counts.other
        ),
    }
}
