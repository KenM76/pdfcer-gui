//! # `sign` — putting the operator's own digital signature on a document
//!
//! The answer to the request this shell filed on **2026-09-03**: *"a document
//! cannot be signed."* `pdfcer-core` answered it on 2026-09-05 with
//! `pdfcer_core::sign` — 101 public items across `Pass 10.7` (PKCS#12 identity
//! loading), `10.8` (the CAdES `SignedData`, PAdES B-B) and `10.9`
//! (`EditSession::sign`) — whose own module header says it is *"the family the
//! `pdfcer-gui` request of 2026-09-03 asked for."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/sign/mod.md`.

use std::path::{Path, PathBuf};

use pdfcer_core::edit::EditSession;
use pdfcer_core::page_tree::{Page, Rect};
use pdfcer_core::sign::apply::{MdpPermission, SignApplyError, SignReport, SignRequest};
use pdfcer_core::sign::pkcs12::{Pkcs12Error, Pkcs12Report, Pkcs12Signer};
use pdfcer_core::writer::SaveOptions;

use crate::secret::Secret;

// ---------------------------------------------------------------------------
// What the document says today
// ---------------------------------------------------------------------------

/// **The document's signing situation, read before anything is offered.**
#[derive(Debug, Clone)]
pub struct Standing {
    /// Whether the document carries an `/Encrypt` dictionary.
    pub encrypted: bool,
    /// Whether a deferred redaction is staged and not yet applied or cancelled.
    pub redaction_pending: bool,
    /// Whether the base loaded through cross-reference recovery, which makes
    /// an incremental update impossible (engine decision 013).
    pub recovered: bool,
    /// How many signatures the document already carries. Not a refusal —
    /// PDF allows many — but the operator should be told they are adding to a
    /// set rather than starting one.
    pub prior_signatures: usize,
    /// The `/DocMDP` `/P` of a certification signature, if there is one.
    ///
    /// **`None` means "no certification signature", not "no `/P`"** — the
    /// engine's census documents that trap, and `/P` absent on a present
    /// certification reports `Some(2)`, because Table 254's default is
    /// permissive.
    pub certification_permission: Option<u8>,
    /// How many pages, for the visible-signature page chooser.
    pub pages: usize,
    /// Whether [`OpenDoc::path`] names a file that exists.
    ///
    /// Asked of the **file system** rather than carried as a flag, exactly as
    /// `crate::app::save::has_a_file` asks it: a second source of truth drifts,
    /// and the failure when it does is writing over the wrong file.
    pub on_disk: bool,
    /// **The empty signature fields somebody already placed in this document.**
    ///
    /// `Pass 10.13`'s whole subject: the *"sign here"* boxes a sender puts on a
    /// drawing before mailing it out. Read once, when the window opens, for
    /// [`Self`]'s stated reason — a list re-read per frame could change under
    /// the choice seeded from it.
    ///
    /// Empty in the ordinary case, and that is the point of listing rather
    /// than assuming: most documents carry none, and offering *"sign into an
    /// existing box"* on one of them would be an option whose only outcome is a
    /// question.
    pub empty_fields: Vec<SigField>,
    /// Whether the document already carries a `/DocMDP` certification.
    ///
    /// Distinct from [`Self::certification_permission`] only in what an absent
    /// `/P` means: the census reports `Some(2)` for a certification with no
    /// `/P` because Table 254's default is permissive, so the permission is
    /// never `None` on a certified document — but reading *"is it certified?"*
    /// off a permission is the kind of derivation that survives one refactor.
    pub certified: bool,
}

/// **One pre-placed, empty signature field — a box the document's author put
/// there for somebody to sign in.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SigField {
    /// The fully qualified field name (`/T`, dotted through its ancestors).
    /// This is what `SignRequest::field_name` takes.
    pub name: String,
    /// The 0-based page the widget sits on, when it could be resolved.
    ///
    /// `None` rather than a guess. `/P` is optional on a widget (§12.5.2), and
    /// a field whose widget is in no page's `/Annots` either has no page or has
    /// one this shell could not find; saying *"page 1"* in either case would be
    /// a sentence about the document that the document does not support.
    pub page: Option<usize>,
    /// Whether the field's own rectangle has no area — the author's own choice
    /// of an **invisible** signature (§12.7.4.5), honoured rather than
    /// corrected.
    pub invisible: bool,
    /// **The field carries a `/Lock` (Table 233): signing it FREEZES fields the
    /// author nominated.**
    ///
    /// The engine honours it as a `/FieldMDP` signature reference (§12.8.2.4),
    /// copying Action and Fields from the lock. `Some` carries the lock's own
    /// `/Action` name — `All`, `Include`, `Exclude` — so the sentence beside the
    /// field can say *which* freeze it is.
    ///
    /// Disclosed **before** the press, not in the summary afterwards. The
    /// engine reports it on `SignReport::field_lock` and this shell shows that
    /// too, but a consequence an operator learns about after the file is written
    /// is a consequence he did not consent to.
    pub locks: Option<String>,
    /// Whether the field carries an `/SV` seed-value dictionary (Table 234) —
    /// conditions the author attached to signing it.
    pub constrained: bool,
    /// Why this field cannot be signed into, when it cannot.
    ///
    /// `None` means it can. Listed **with the reason** rather than filtered out,
    /// on this project's standing rule: an operator looking for the box the
    /// sender told him about needs to find it and be told why it is not
    /// offered — an absent row is indistinguishable from a document that never
    /// had one.
    pub unusable: Option<FieldBar>,
}

/// Why a pre-placed signature field cannot be signed into.
///
/// A closed set with one sentence each in [`crate::text::sign`], each mirroring
/// a `SignApplyError` the engine would raise if it were chosen anyway.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldBar {
    /// The field's widgets are under `/Kids` rather than merged into the field
    /// dictionary. `SignApplyError::FieldHasKids` — the first cut signs into a
    /// merged field-widget only.
    HasKids,
}

impl SigField {
    /// Whether this field may be chosen.
    #[must_use]
    pub const fn selectable(&self) -> bool {
        self.unusable.is_none()
    }
}

/// **Read every empty `/FT /Sig` field out of the open document.**
#[must_use]
pub fn read_empty_signature_fields(session: &EditSession, pages: &[Page]) -> Vec<SigField> {
    use pdfcer_core::forms::{FieldType, FieldValue};
    use pdfcer_core::graph::ObjectGraph;
    use pdfcer_core::object::Object;

    let graph = session.graph();
    let Some(form) = pdfcer_core::forms::parse_acroform(&graph) else {
        return Vec::new();
    };
    form.fields
        .iter()
        .filter(|f| f.field_type == Some(FieldType::Signature))
        // `/V` present is a SIGNED field. `FieldValue::Signature` is the
        // projection's word for "a signature dictionary is this field's value";
        // anything else on a `/Sig` field is `Absent`.
        .filter(|f| f.value == FieldValue::Absent)
        .map(|f| {
            let dict = graph.resolved(f.id).as_dict();
            let locks = dict
                .and_then(|d| d.get(b"Lock"))
                .map(|o| graph.resolve(o))
                .and_then(Object::as_dict)
                .map(|lock| {
                    lock.get(b"Action")
                        .map(|o| graph.resolve(o))
                        .and_then(Object::as_name)
                        .map_or_else(
                            // ui-text-exempt: a PDF name from the operator's
                            // own file, echoed for the disclosure — not copy.
                            || String::from("All"),
                            |n| String::from_utf8_lossy(n.as_bytes()).into_owned(),
                        )
                });
            let constrained = dict.is_some_and(|d| d.contains_key(b"SV"));
            // `merged` is the projection's own answer to the same question
            // `/Kids` asks, and it is the one the engine's refusal keys on.
            let unusable = (!f.merged).then_some(FieldBar::HasKids);
            let widget = f.widgets.first();
            let invisible = widget.and_then(|w| w.rect).is_none_or(|r| {
                (r.urx - r.llx).abs() < f64::EPSILON || (r.ury - r.lly).abs() < f64::EPSILON
            });
            let page = widget
                .and_then(|w| w.page)
                .and_then(|id| pages.iter().position(|p| p.id == id));
            SigField {
                name: f.fully_qualified_name.clone(),
                page,
                invisible,
                locks,
                constrained,
                unusable,
            }
        })
        .collect()
}

impl Standing {
    /// Read it off the open document.
    #[must_use]
    pub fn read(session: &EditSession, path: &Path, pages: &[Page]) -> Self {
        let base = session.document();
        let census = session.signature_census();
        Self {
            encrypted: base.encryption().is_some(),
            redaction_pending: session.has_pending_redaction(),
            recovered: base.loaded_via_recovery(),
            prior_signatures: census.signatures,
            certification_permission: census.certification_permission,
            pages: pages.len(),
            on_disk: path.is_file(),
            empty_fields: read_empty_signature_fields(session, pages),
            certified: census.certifications > 0,
        }
    }

    /// **Whether a CERTIFYING signature may be offered at all, and if not, why.**
    pub const fn may_certify(&self) -> Result<(), CertifyBar> {
        if self.certified {
            return Err(CertifyBar::AlreadyCertified);
        }
        if self.prior_signatures > 0 {
            return Err(CertifyBar::NotFirst {
                existing: self.prior_signatures,
            });
        }
        Ok(())
    }

    /// **Whether this surface may offer anything at all, and if not, why.**
    #[must_use]
    pub fn refusal(&self) -> Option<Refusal> {
        if self.redaction_pending {
            return Some(Refusal::RedactionPending);
        }
        if self.encrypted {
            return Some(Refusal::Encrypted);
        }
        if self.certification_permission == Some(1) {
            return Some(Refusal::CertificationForbids { permission: 1 });
        }
        if self.recovered {
            return Some(Refusal::RecoveredBase);
        }
        if !self.on_disk {
            return Some(Refusal::NotOnDisk);
        }
        None
    }
}

/// Why this document cannot be signed at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// A deferred redaction is staged. Apply or cancel it first.
    RedactionPending,
    /// The document is encrypted, and pdfcer's incremental writer cannot
    /// append to an encrypted base.
    Encrypted,
    /// A certification signature's `/DocMDP` `/P` forbids adding another.
    CertificationForbids {
        /// The `/P` value. Only `1` reaches here — Table 254's `2` is exactly
        /// the permission that allows signing.
        permission: u8,
    },
    /// The base loaded through cross-reference recovery; nothing can be
    /// appended to it.
    RecoveredBase,
    /// The document has never been written to disk. See §4.
    NotOnDisk,
}

/// **Why this document cannot be CERTIFIED**, though it can still be signed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertifyBar {
    /// `SignApplyError::AlreadyCertified` — §12.8.2.2.1 permits one `/DocMDP`
    /// per document.
    AlreadyCertified,
    /// `SignApplyError::CertificationNotFirst` — the certifier is the author,
    /// *"the person applying the first signature"*, and a later certification
    /// could not govern the changes made before it.
    NotFirst {
        /// How many signatures are already there.
        existing: usize,
    },
}

// ---------------------------------------------------------------------------
// The identity
// ---------------------------------------------------------------------------

/// **A loaded signing identity — a private key and its certificate chain.**
pub struct Identity {
    signer: Pkcs12Signer,
    /// Where it came from, for the window's summary line. A path, never a
    /// preference — see §5.
    source: PathBuf,
}

impl Identity {
    /// **Open a `.pfx`/`.p12` with `passphrase`.**
    ///
    /// The file is read here rather than by the caller so that the bytes have
    /// exactly one owner and one lifetime: they hold an encrypted private key,
    /// and a `Vec<u8>` of them passed around the dialog is a copy nobody is
    /// tracking.
    ///
    /// A read failure and a parse failure are **different** answers, because
    /// they send the operator to different places — one to the file picker, one
    /// to the passphrase field. Flattening them into "could not open the
    /// certificate" is the shape of an afternoon spent retyping a correct
    /// passphrase.
    ///
    /// # Errors
    ///
    /// [`IdentityFailure`]: the file system refused, or the container did.
    pub fn open(path: &Path, passphrase: &Secret) -> Result<Self, IdentityFailure> {
        let bytes = std::fs::read(path).map_err(|e| IdentityFailure::Unreadable(e.to_string()))?;
        let signer = Pkcs12Signer::from_der(&bytes, passphrase.expose_str())
            .map_err(IdentityFailure::Import)?;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // WHAT IS ABSENT HERE IS THE DESIGN. No passphrase, no length
            // of one, no path, no serial, no key bytes. §5 of this module's
            // header argues the length; the PATH is left out because a trace
            // file is kept as evidence and a durable pointer at where somebody
            // stores their digital ID is not ours to publish.
            //
            // What IS here is what a diagnosis needs: the container verified,
            // what protected it, what kind of key came out, and how long the
            // chain is. `subject` is deliberately NOT traced — it is the
            // operator's own name.
            let r = signer.report();
            format!(
                "sign-identity mac={} key={} chain={} unrelated={} scheme={}",
                r.mac.as_deref().unwrap_or("none"),
                r.key,
                r.chain_length,
                r.unrelated_certificates,
                r.key_scheme,
            )
        });
        Ok(Self {
            signer,
            source: path.to_path_buf(),
        })
    }

    /// What the container was made of — the engine's rule-4 disclosure.
    #[must_use]
    pub fn report(&self) -> &Pkcs12Report {
        self.signer.report()
    }

    /// The file it was loaded from.
    #[must_use]
    pub fn source(&self) -> &Path {
        &self.source
    }
}

/// Why an identity could not be loaded.
#[derive(Debug, Clone)]
pub enum IdentityFailure {
    /// The file system refused, already formatted.
    Unreadable(String),
    /// The container refused, by name. Every [`Pkcs12Error`] variant is a
    /// refusal that names its own cause; the window prints it verbatim rather
    /// than re-wording it, because the engine distinguishes *wrong passphrase*
    /// from *a scheme pdfcer does not implement* and flattening the two sends
    /// an operator to re-type a passphrase that was correct.
    Import(Pkcs12Error),
}

// ---------------------------------------------------------------------------
// Placement
// ---------------------------------------------------------------------------

/// **Whether the signature is drawn on a page, and where.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Placement {
    /// `/Rect [0 0 0 0]` on the first page; nothing drawn. The default.
    Invisible,
    /// A widget on `page` (0-based), at the default box — see [`default_rect`].
    Visible {
        /// 0-based page index.
        page: usize,
    },
    /// **Sign INTO a pre-placed empty signature field.** `Pass 10.13`.
    ///
    /// The field's own `/Rect` and page place the appearance; nothing is
    /// appended to `/Annots` or `/Fields` and the author's dictionary is
    /// otherwise untouched. A field whose rectangle is zero-area is the
    /// author's own choice of an invisible signature and is honoured as one.
    ExistingField {
        /// The field's fully qualified name (`/T`, dotted through its
        /// ancestors). Passed to `SignRequest::field_name`.
        name: String,
    },
}

/// The visible signature's box, in PDF points: **180 × 60, inset 36 pt from
/// the page's bottom-right corner.**
#[must_use]
pub fn default_rect(media: Rect) -> Rect {
    /// The box, in PDF points.
    const W: f64 = 180.0;
    /// See [`W`].
    const H: f64 = 60.0;
    /// The inset from the page edge, in PDF points. Half an inch.
    const INSET: f64 = 36.0;

    let page_w = (media.urx - media.llx).abs();
    let page_h = (media.ury - media.lly).abs();
    let w = W.min(page_w);
    let h = H.min(page_h);
    // The inset shrinks rather than pushing the box off the page when there is
    // not room for both it and the box.
    let inset_x = INSET.min((page_w - w).max(0.0) / 2.0);
    let inset_y = INSET.min((page_h - h).max(0.0) / 2.0);
    let llx = media.llx.min(media.urx);
    let lly = media.lly.min(media.ury);
    let right = llx + page_w - inset_x;
    let bottom = lly + inset_y;
    Rect {
        llx: right - w,
        lly: bottom,
        urx: right,
        ury: bottom + h,
    }
}

// ---------------------------------------------------------------------------
// Preparation
// ---------------------------------------------------------------------------

/// **What the operator authored, ready for the engine.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Authored {
    /// `/Reason`, free text. Empty means the key is omitted.
    pub reason: String,
    /// `/Location`, free text — the operator's words, not a resolved place.
    /// Empty means the key is omitted.
    pub location: String,
    /// Where the widget goes.
    pub placement: Placement,
    /// `/M`, the claimed signing time as a PDF date string. Captured when the
    /// window opened and **shown on screen**, so what is written is the time
    /// the operator was told — the engine reads no clock and says a GUI should
    /// pass *"the time it showed the operator."*
    pub signing_time: String,
    /// **Make this a certifying (author) signature, at this `/DocMDP` level.**
    ///
    /// `Pass 10.12`; `None` is an ordinary approval signature and is the
    /// default. §2d argues why it lives in this window rather than on a command
    /// of its own, and [`Standing::may_certify`] why it is sometimes absent.
    ///
    /// The engine's own type, `MdpPermission`, rather than a local mirror.
    /// The three levels ARE Table 254's three values, their meanings are the
    /// standard's, and `MdpPermission::meaning` already renders each in plain
    /// words — a parallel enum here would be a second spelling of a fixed list
    /// whose only possible divergence is a bug.
    pub certify: Option<MdpPermission>,
}

/// Why a signing did not produce bytes.
#[derive(Debug)]
pub enum PrepareFailure {
    /// The document itself is out of scope. Reachable from [`prepare`] as well
    /// as from [`Standing::refusal`] because the two are asked at different
    /// moments and the document can change between them.
    Refused(Refusal),
    /// The engine refused, by name.
    Engine(SignApplyError),
}

/// **Finished bytes, waiting for a destination.**
pub struct Prepared {
    /// The signed document, in memory.
    bytes: Vec<u8>,
    /// What the engine says it wrote.
    report: SignReport,
}

impl Prepared {
    /// The engine's own account of the signature it wrote.
    #[must_use]
    pub const fn report(&self) -> &SignReport {
        &self.report
    }

    /// How many bytes, for the trace and the outcome sentence.
    #[must_use]
    pub const fn byte_len(&self) -> usize {
        self.bytes.len()
    }

    /// Whether the engine re-read its own output and found the signature
    /// intact before returning it. See §2 — always `true` on `Ok`, carried so
    /// the fact is stated rather than assumed, and **not** a substitute for an
    /// independent read.
    #[must_use]
    pub const fn self_verified(&self) -> bool {
        self.report.self_verified
    }

    /// **Write them to `target`, atomically.**
    pub fn write_to(&self, target: &Path) -> Result<usize, WriteFailure> {
        let temporary = target.with_extension("pdfcer-tmp");
        std::fs::write(&temporary, &self.bytes).map_err(|e| WriteFailure(e.to_string()))?;
        if let Err(err) = std::fs::rename(&temporary, target) {
            let _ = std::fs::remove_file(&temporary);
            return Err(WriteFailure(err.to_string()));
        }
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // `path` is Debug-quoted for `protect-written`'s reason: a
            // Windows path routinely contains a space, and a consumer splitting
            // the line into `key=value` pairs would lose every field after it.
            //
            // ⚠ `subject` and `serial` are NOT here. They are on screen, in the
            // report the operator reads, which is where a disclosure about
            // whose key was used belongs. A trace file is kept and shared.
            //
            // `field_reused=` is here as well as on `sign-prepared`, and the
            // duplication is deliberate: this is the line that says a FILE
            // exists, and *"the signature went into the box the sender placed"*
            // is a claim about that file. A check reading only the written line
            // would otherwise have to correlate two events to learn the one
            // thing the feature is about.
            format!(
                "sign-written path={:?} bytes={} field={} prior={} self_verified={} \
                 field_reused={} certified={}",
                target,
                self.bytes.len(),
                self.report.field_name,
                self.report.prior_signatures,
                u8::from(self.report.self_verified),
                u8::from(self.report.field_reused),
                self.report.certification.map_or_else(
                    // ui-text-exempt: trace token, never displayed.
                    || "none".to_owned(),
                    |p| p.p().to_string()
                ),
            )
        });
        Ok(self.bytes.len())
    }
}

/// **What became of one signing, as the dialog needs to hear it.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The bytes reached a path.
    Written {
        /// Where they went.
        path: PathBuf,
        /// Whether that path was the document the operator has open.
        replaced: bool,
        /// The rule-4 disclosure: what the engine says it wrote, already
        /// worded. See [`crate::text::sign::written_details`].
        details: String,
    },
    /// Nothing was written, and this is why — already an operator-facing
    /// sentence.
    Failed(String),
}

/// The file system refused, already formatted.
#[derive(Debug, Clone)]
pub struct WriteFailure(pub String);

impl std::fmt::Display for WriteFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// **Sign the document and hand back the bytes.**
pub fn prepare(
    session: &mut EditSession,
    pages: &[Page],
    identity: &Identity,
    authored: &Authored,
    options: &SaveOptions,
) -> Result<Prepared, PrepareFailure> {
    let mut request = SignRequest::at(authored.signing_time.clone());
    // Empty is omitted rather than written as an empty string: `/Reason ()` in
    // a signature dictionary is a claim that the operator gave a reason and it
    // was nothing, which is not what an untouched field means.
    request.reason = non_empty(&authored.reason);
    request.location = non_empty(&authored.location);
    // `/Name` is deliberately NEVER set — see `crate::text::sign`'s header.
    // The engine: "`None` omits the key and a verifier falls back to the
    // certificate subject (Table 252 says it should anyway)." A free-text name
    // beside a certificate is a second, unverifiable claim about who signed.
    // `Pass 10.12`. Written before the placement, so the request is
    // assembled in the order the engine guards it: certification is refused
    // before any field is resolved.
    request.certify = authored.certify;
    request.visible = match &authored.placement {
        Placement::Invisible => None,
        // `Pass 10.13`: the field's OWN `/Rect` and page place the
        // appearance, so `visible` stays `None`. Setting both is
        // `SignApplyError::RectRefusedForExistingField` — which cannot be
        // reached from here, because `Placement`'s three arms are exclusive.
        // That is the enum earning its shape: the refusal is unrepresentable
        // rather than reachable-and-handled.
        Placement::ExistingField { name } => {
            request.field_name = Some(name.clone());
            None
        }
        Placement::Visible { page } => {
            let page = *page;
            // The CROP box, not the media box, and the difference is what
            // the operator sees. Content is clipped to `/CropBox` at display
            // time (Table 30), so a box placed against a larger `/MediaBox` on
            // a trimmed sheet would be laid partly or wholly outside the
            // visible page — present in the file, and invisible in every
            // reader. `Page::crop_box` defaults to `media_box`, so the two
            // agree on every document that does not trim.
            //
            // A page index the vector does not hold cannot arise from the
            // chooser, which is built from this same vector; US Letter is the
            // fallback rather than a panic, on this project's standing
            // preference against panicking on a branch a guard excluded.
            let crop = pages
                .get(page)
                .map_or(Rect::from_corners(0.0, 0.0, 612.0, 792.0), |p| p.crop_box);
            Some((page, default_rect(crop)))
        }
    };

    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        // `into_field=` is a BIT, not the field's name. A field name is text
        // out of the operator's own document — a title block's wording, a
        // customer's name — and this line goes into a file the harness keeps as
        // evidence. `sign-prepared` below carries what the engine wrote, which
        // is the answer a diagnosis needs; what this line owes is *which shape
        // of request was built*.
        //
        // ⚠ `certify=` is the `/P` NUMBER or `none`, never `{:?}` of
        // `MdpPermission`: a check parses this line, and a Debug rendering is a
        // spelling nobody chose.
        format!(
            "sign-requested visible={} page={} into_field={} certify={} reason={} location={} \
             time_len={}",
            u8::from(request.visible.is_some()),
            request.visible.map_or(usize::MAX, |(p, _)| p),
            u8::from(request.field_name.is_some()),
            request.certify.map_or_else(
                // ui-text-exempt: trace token, never displayed.
                || "none".to_owned(),
                |p| p.p().to_string()
            ),
            u8::from(request.reason.is_some()),
            u8::from(request.location.is_some()),
            authored.signing_time.len(),
        )
    });

    let (bytes, report) = session
        .sign(identity.signer_ref(), &request, options)
        .map_err(PrepareFailure::Engine)?;

    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        // `field_reused=`, `locked=`, `notes=` and `certified=` are what
        // `Pass 10.12`–`10.14` added, and each is the ONE fact a check needs to
        // tell "the signature went into the sender's box" from "a new box was
        // created beside it" — two outcomes whose byte counts and field names
        // can be identical.
        //
        // ⚠ `locked=` is a BIT and `notes=` a COUNT. Both of their contents are
        // field names and sentences out of the operator's document; the engine's
        // own wording of them is on screen, where he can act on it.
        format!(
            "sign-prepared bytes={} field={} algorithm={:?} certificates={} cms={} reserved={} \
             prior={} level={} self_verified={} field_reused={} locked={} notes={} \
             appearance_lines={} certified={}",
            bytes.len(),
            report.field_name,
            report.algorithm,
            report.certificates,
            report.cms_bytes,
            report.reserved_bytes,
            report.prior_signatures,
            report.pades_level,
            u8::from(report.self_verified),
            u8::from(report.field_reused),
            u8::from(report.field_lock.is_some()),
            report.notes.len(),
            report.appearance_lines.len(),
            report.certification.map_or_else(
                // ui-text-exempt: trace token, never displayed.
                || "none".to_owned(),
                |p| p.p().to_string()
            ),
        )
    });
    Ok(Prepared { bytes, report })
}

impl Identity {
    /// The engine's signer, as the trait object `EditSession::sign` takes.
    pub(crate) fn signer_ref(&self) -> &dyn pdfcer_core::sign::Signer {
        &self.signer
    }
}

/// `None` for a field the operator left alone.
fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

/// **The name to suggest in the save picker — never the source file.**
///
/// The standing rule for every write that produces a second document.
#[must_use]
pub fn suggested_path(source: &Path, suffix: &str) -> PathBuf {
    let stem = source.file_stem().map_or_else(
        // ui-text-exempt: a filename fallback for a path with no stem, not
        // operator copy. Every sibling suggestion function makes the same one.
        || String::from("document"),
        |s| s.to_string_lossy().into_owned(),
    );
    let named = format!("{stem}{suffix}.pdf");
    source
        .parent()
        .map_or_else(|| PathBuf::from(&named), |parent| parent.join(&named))
}

#[cfg(test)]
mod tests;
