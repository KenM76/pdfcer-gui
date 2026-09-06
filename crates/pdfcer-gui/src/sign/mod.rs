//! # `sign` — putting the operator's own digital signature on a document
//!
//! The answer to the request this shell filed on **2026-09-03**: *"a document
//! cannot be signed."* `pdfcer-core` answered it on 2026-09-05 with
//! `pdfcer_core::sign` — 101 public items across `Pass 10.7` (PKCS#12 identity
//! loading), `10.8` (the CAdES `SignedData`, PAdES B-B) and `10.9`
//! (`EditSession::sign`) — whose own module header says it is *"the family the
//! `pdfcer-gui` request of 2026-09-03 asked for."*
//!
//! The window is [`crate::dialogs::sign`]; this module is everything that can
//! be decided **without a `Ui`** — what the document says about itself, what
//! may be offered, which identity is loaded, and the atomic write at the end.
//! The split is [`crate::protect`]'s and [`crate::redact`]'s, taken rather than
//! re-argued: every rule on this surface is a rule about **the operator's
//! file**, and a rule that can only be exercised by driving a window is a rule
//! that gets asserted once, by hand, and then drifts.
//!
//! ---
//!
//! # 1. ★★★ THE SUBSYSTEM WAS SHIPPED, ANSWERED OUR OWN REQUEST, AND WAS NOT
//! IN THE BINARY FOR THREE DAYS
//!
//! This module exists because of a defect worth stating before any of its
//! design, since the defect is the more transferable half.
//!
//! `crates/pdfcer-gui/Cargo.toml` took `pdfcer-core` with
//! `default-features = false` and forwarded `jpx` and `ocrs`. The engine's
//! `signing` feature is **default on**; a `default-features = false` dependency
//! that does not re-name it strips it. So `pdfcer_core::sign` did not exist in
//! this build at all. **Nothing failed to compile. No test went red.**
//!
//! ★★ That is the JPX incident, repeating, three days after the warning about
//! it was written into the very manifest that repeated it — the comment at
//! `Cargo.toml`'s feature block records the day the GUI silently lost JPEG 2000
//! decoding to the identical omission and says, in as many words, *"forgetting
//! to forward does not fail to compile."*
//!
//! ⇒ **A warning does not protect a code path written after it.** The
//! mechanism that does is `tools/gates/check-forwarded-features.sh`, which
//! reads the engine's own `default = [...]` and fails the build when a name in
//! it is neither forwarded nor listed with a reason.
//!
//! ★ And it went unnoticed for two of those days because
//! `tools/gates/check-verb-coverage.sh` scored `EditSession::sign` as
//! **consumed** on the bare word `sign` appearing in `app/actions/bookmarks.rs`
//! — in a documentation table about the arithmetic **sign of `/Count`**. That
//! gate now matches call shape. Two independent instruments, both green, both
//! about nothing.
//!
//! ---
//!
//! # 2. The engine's verbs, verified against the source at the pinned revision
//!
//! `D:\Dev\pdfcer\crates\pdfcer-core\src\`, at the revision `Cargo.lock` pins
//! (`pdfcer-core 0.42.0`, `d6b998f`). Checked at that commit rather than at
//! `main`, on [`crate::protect`]'s standing rule: what compiles here is the
//! pin, and a sentence about the engine has a shelf life measured in hours.
//!
//! ```text
//! sign::pkcs12::Pkcs12Signer::from_der(&[u8], &str) -> Result<Pkcs12Signer, Pkcs12Error>
//! Pkcs12Signer::report()                            -> &Pkcs12Report
//! EditSession::sign(&dyn Signer, &SignRequest, &SaveOptions)
//!                                                   -> Result<(Vec<u8>, SignReport), SignApplyError>
//! forms::parse_acroform(&SessionGraph)              -> Option<AcroForm>
//! ```
//!
//! ---
//!
//! # 2b. ★★★ WHAT THE PIN MOVE OF 2026-09-06 CHANGED, AND THE COPY IT MADE FALSE
//!
//! The lock went `f9bc7c8` (v0.41.0) → `d6b998f` (v0.42.0), thirteen commits,
//! carrying three signing Passes. **Nothing failed to compile** — every one of
//! them is additive on a `#[non_exhaustive]` struct — which is precisely why
//! the interesting half is the prose:
//!
//! | Pass | what arrived | what it falsified here |
//! |---|---|---|
//! | `10.12` (`02bb1ba`) | `SignRequest::certify` — a **certifying** (`/DocMDP`) signature | nothing; the capability was simply absent |
//! | `10.13` (`ab40127`) | `SignRequest::field_name` resolving an EXISTING empty `/FT /Sig` field — signing **into** a box somebody else placed | nothing; absent |
//! | `10.14` (`187fa09`) | a **composed** visible appearance: signer CN, date, reason, location, Helvetica, shrink-to-fit | ★★★ [`crate::text::sign::placement_note`], which told the operator in as many words that *"the box is an empty frame"* |
//!
//! ★★ That third row is the one worth carrying, because the falsehood was
//! **under-promising** and therefore unreportable: an operator told the box
//! would be empty, who then finds his own name in it, has been pleasantly
//! surprised and will never file a defect. Nothing on the screen, in a test, or
//! in a gate could have gone red. What caught it was that the string's own doc
//! comment carried the date, the engine commit and the instruction *"when the
//! pin moves past `f9bc7c8`, re-read this string first"* — a dated citation
//! that names its own expiry.
//!
//! ⇒ **A claim about the engine is a citation, and a citation gets a date and a
//! successor.** The corrected string carries the same apparatus.
//!
//! ---
//!
//! # 2c. ★★★ SIGNING INTO A BOX THE SENDER PLACED — the half that matters
//!
//! What shipped on 2026-09-06 signs by **creating** a signature field. That is
//! the wrong half for this operator's ordinary day: a drawing goes out for
//! approval, the sender places a *"sign here"* box on the title block, and it
//! comes back needing a signature **in that box**. `Pass 10.13` is the other
//! half, and it changes three things here.
//!
//! **1. The field is chosen, not created.** [`Standing::empty_fields`] lists
//! every `/FT /Sig` field in the document that has no `/V` — read once, when the
//! window opens, out of `forms::parse_acroform`. [`Placement::ExistingField`]
//! names one.
//!
//! **2. ★★★ Placement becomes a THREE-way choice, and the combination the
//! engine refuses is made unrepresentable.** `SignRequest::visible` beside a
//! `field_name` that resolves to an existing field is
//! `SignApplyError::RectRefusedForExistingField` — *"the existing field already
//! has a rectangle; --visible/--page do not apply."* This shell could have sent
//! both and shown the refusal. It does not: [`Placement`] is one enum with three
//! arms, so *"draw the box here"* and *"use the sender's box"* cannot both be
//! true, and the window **retires** the page chooser rather than greying it —
//! R9's *absent* branch, with [`crate::text::sign::placement_field_note`]
//! saying why it went.
//!
//! **3. ★★★ TWO ENFORCEMENT FAMILIES ARRIVE WITH IT, AND BOTH ARE THE AUTHOR'S
//! RULES RATHER THAN PDFCER'S.** This is the design decision the wording has to
//! carry, and getting it wrong makes a working feature read as a defect.
//!
//! * **`/Lock` (Table 233)** on the chosen field is *honoured*, as a
//!   `/FieldMDP` signature reference (§12.8.2.4) whose Action and Fields are
//!   copied from the lock. So signing that box can legitimately **freeze other
//!   fields the author nominated** — a real consequence for a form the operator
//!   may still have to fill in. [`SigField::locks`] carries it, and the window
//!   says so **beside the field, before the press**, not in the summary
//!   afterwards.
//! * **`/SV` (Table 234)** is enforced **in full**: a required constraint the
//!   request does not meet is refused by name with the satisfying values
//!   (`SeedValueViolated`); a recommended one unmet is disclosed on
//!   `SignReport::notes`; and anything pdfcer does not evaluate — `/Cert`, a
//!   required timestamp, a legal attestation, revocation info, an unknown key —
//!   is **refused rather than skipped** (`SeedValueUnevaluable`).
//!
//! ★★★ **The engine is therefore deliberately stricter than Acrobat, and the
//! operator will meet refusals on documents Acrobat would sign.** A sentence
//! that reads *"pdfcer could not sign this"* would be true and would be taken as
//! a defect in pdfcer. [`crate::text::sign::author_imposed`] is the one wording
//! for all of them and it says whose rule it is: **the person who prepared the
//! document wrote the condition**, pdfcer will not sign around it, and the
//! remedy is another box or a word with the sender. The strictness is stated as
//! a choice, because an operator comparing two programs deserves to know which
//! one is doing something unusual and why.
//!
//! ---
//!
//! # 2d. Certifying — an option in this window, not a second command
//!
//! `Pass 10.12`'s `SignRequest::certify` writes the `/DocMDP` transform and the
//! catalog `/Perms`, which is *"the author's signature"*: it says what may be
//! changed afterwards without invalidating it (Table 254 — `P` 1, 2 or 3).
//!
//! ★ It is a **radio pair inside the Sign window**, not `file.certify` on the
//! ribbon, and that is a design decision rather than an economy. The two acts
//! share every field on this form — the same identity, the same reason, the same
//! placement, the same destination, the same private key handled the same way —
//! and differ in one value. A second command would put all of that in a second
//! file, which is where a disclosure goes missing; and it would ask the operator
//! to know the word *certify* before he could find out what it means. Here the
//! choice is beside its explanation.
//!
//! ⚠ Both of the engine's certification refusals are **states of the document**,
//! not of the request: a certification must be the document's FIRST signature
//! (`CertificationNotFirst`) and there is at most one per document
//! (`AlreadyCertified`). Both are knowable when the window opens, so
//! [`Standing::may_certify`] answers them there and the option is **absent with
//! a sentence** rather than offered and then refused.
//!
//! ## ★★ What the engine does that this module must NOT duplicate
//!
//! `EditSession::sign` **self-verifies**: step 5 of its own documentation
//! re-parses the bytes it is about to return and runs `signature_verify` over
//! them, and *"anything but `Integrity::Verified` with full coverage is a
//! refusal, and no bytes are returned. pdfcer does not hand out a signature it
//! cannot itself verify."*
//!
//! So there is no verification step in [`prepare`], and adding one would be a
//! second derivation of one fact — which is how two surfaces come to disagree.
//! What this module does instead is **state** it: [`Prepared::self_verified`]
//! carries the engine's own `SignReport::self_verified`, which that struct's
//! documentation says is *"always `true` on `Ok`; present so the fact is
//! stated, not assumed."*
//!
//! ⚠ That is emphatically **not** the same as this project's R1 bar. A field
//! saying the engine checked its own output is still the engine's word inside
//! one process. The independent read is `tools/ui-verify`'s
//! `a_document_can_be_signed_and_the_signature_is_in_the_file`, which reopens
//! the written file **in a fresh process** and reads it through the Signatures
//! panel — the verification side that shipped as `Pass 10.5` — so the oracle is
//! a different subsystem from the one under test.
//!
//! ---
//!
//! # 3. ★★★ Why the session is handed to the verb, and why the file is reopened
//! afterwards
//!
//! The opposite of [`crate::protect`]'s answer, and the asymmetry is the
//! engine's rather than a preference here.
//!
//! `EditSession::sign` takes **`&mut self`** and stages a real, undoable
//! `CommandKind::AddSignatureField` — the signature field, its widget and the
//! signature dictionary with its zero-filled `/Contents` hole — then serialises
//! the session as an **incremental update**. It must be the open session: the
//! operator's unsaved edits are in it, and a signature that did not cover them
//! would cover a document they are not looking at.
//!
//! ★★ But the session is **left holding the placeholder**, not the signature.
//! The engine says so outright: *"the session still holds the staged
//! placeholder objects (zeros in `/Contents`) … a caller that wants to keep
//! editing must re-open the returned bytes. A CLI writes the bytes and is done;
//! a GUI reloads."*
//!
//! ⇒ So after a successful write this surface offers, and does not perform,
//! **[`crate::dialogs::sign`]'s *Open the signed document*** — a second
//! document tab on the file that was just written. Offering rather than
//! performing, because replacing the operator's open document out from under
//! them would discard an undo history they can still see; and offered rather
//! than omitted, because the open document is now in a state whose only honest
//! description is *"this is not the file you signed"*, and an operator left to
//! discover that by pressing `Ctrl+S` would append a second revision on top of
//! a stale base.
//!
//! ⚠ **What is deliberately NOT done: no `Ctrl+Z` is pushed and no state is
//! rewound.** Undoing the staged command in the old session is, in the
//! engine's own words, *"harmless and pointless"*. Doing it would look like
//! tidying up and would put a spurious entry on the operator's undo stack for
//! an act that produced a file.
//!
//! ---
//!
//! # 4. The five refusals, all of which are STATED rather than discovered
//!
//! R9: *the control is absent or explained, never a button that fails on
//! press.* Whether **this** document can be signed is not knowable when the
//! command registry is built, so `file.sign` stays on the ribbon and the window
//! opens and says why instead of drawing a form whose only possible outcome is
//! a failure. [`Standing::refusal`] is the whole of that decision, as a pure
//! function.
//!
//! | [`Refusal`] | engine variant | why this shell can reach it |
//! |---|---|---|
//! | [`Refusal::Encrypted`] | `SignApplyError::Encrypted` | File ▸ Security ▸ Encrypt… ships (`O119`), so this shell can *make* an encrypted document and then be asked to sign it |
//! | [`Refusal::RedactionPending`] | `SignApplyError::RedactionPending` | deferred redaction ships (`Pass 250.2`), and a pending removal is a normal mid-session state |
//! | [`Refusal::CertificationForbids`] | `SignApplyError::CertificationForbids { permission: 1 }` | a certified document opened from disk |
//! | [`Refusal::RecoveredBase`] | `SignApplyError::RecoveredBase` | a damaged file that loaded through cross-reference recovery |
//! | [`Refusal::NotOnDisk`] | *(none — see below)* | File ▸ New makes a document that has never been written |
//!
//! ★★ The first two are the ones the build brief names, and they are the two
//! this shell can produce **in one session without leaving the application**,
//! which is what makes them reachable rather than theoretical.
//!
//! ★★★ The fifth has **no engine counterpart**, and that is the interesting
//! one. `EditSession::sign` would not refuse a document that was never on
//! disk — it would sign it, incrementally, over whatever base the session
//! holds. The refusal is this shell's, and the reason is that an incremental
//! update is *an appendix to a specific file*: signing a document that has
//! never been saved produces bytes whose base revision exists nowhere, so the
//! operator's next ordinary Save would write a different file that the
//! signature does not describe. Saying *"save it first"* is one sentence; the
//! alternative is a signed file with no ancestor.
//!
//! ---
//!
//! # 5. ★★★ THE PASSPHRASE, AND A RULE STRICTER THAN THE ONE BESIDE IT
//!
//! This module handles a **private key**, and that changes the standard from
//! the one [`crate::protect`] works to.
//!
//! * The passphrase becomes a [`Secret`] the instant it leaves the text field,
//!   for `crate::secret`'s reason: `Action` derives `Debug`, this crate traces
//!   to stderr under `PDFCER_DIAG`, and **`tools/ui-verify` captures that
//!   stderr to a file it keeps as evidence**. A single `{:?}` on the path would
//!   write it to disk in plain text.
//! * ⚠ **No trace line here carries the passphrase's LENGTH either**, and that
//!   is a deliberate departure from `protect::prepare`, which traces
//!   `user_chars=` and `owner_chars=`. There, the length plus "is it ASCII" is
//!   what explains a SASLprep normalisation refusal completely, and a document
//!   password is the operator's own gate on their own file. Here the secret
//!   guards a **private key** — the material an impersonation needs — and a
//!   length is a search-space reduction written into a file that outlives the
//!   session. `passphrase=set|empty` answers the only diagnostic question
//!   ("was one supplied at all?") and reduces nothing.
//! * [`Identity`] holds the engine's `Pkcs12Signer`, whose own `Debug` impl
//!   prints the report and never the key. This type does not derive `Debug` at
//!   all; see its note.
//! * Nothing writes the `.pfx` path into any persisted preference. A path is
//!   not key material, but a file picker that remembered where the operator
//!   keeps their identity would be a durable pointer at it, written by a
//!   convenience nobody asked for.
//!
//! ---
//!
//! # 6. Where the bytes go — [`crate::redact`]'s shape, part for part
//!
//! Settled by the operator hours before the redaction work started, in his own
//! words: *"why does it have to save to a new file right away? Why can't it
//! just wait on saving until I choose to save over the existing file or save as
//! a new file?"* So, unchanged here:
//!
//! 1. **A new file is the default**, and [`suggested_path`] never proposes the
//!    source. A safe default is a mechanism; a warning is something to click
//!    past.
//! 2. **Replacing the original is offered**, behind one extra acknowledgement
//!    that names the file, and it takes **no picker** — a picker pre-filled
//!    with the source is the shape of every accidental overwrite there has ever
//!    been.
//! 3. **The write is atomic** — temp file, then rename ([`Prepared::write_to`]).
//!
//! ★ Replacing is genuinely reasonable here in a way it is not for a redaction:
//! a signature is an *incremental update*, so the replaced file still contains
//! every byte it had. Nothing is lost by signing in place. It is still not the
//! default, because "the file I sent out" and "the file I signed" being one
//! keystroke apart is worth one deliberate act.

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
///
/// [`crate::protect::Standing`]'s twin, and it exists for that type's stated
/// reason applied to this surface: a form that opened offering to sign a
/// document the engine will refuse has told the operator a falsehood before
/// they touched anything.
///
/// Read **once**, when the window opens, and never re-read per frame. The
/// sentence under the heading must describe the document the operator chose to
/// act on; a value re-read every frame could change out from under the choices
/// seeded from it.
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
    /// ★ Empty in the ordinary case, and that is the point of listing rather
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
///
/// `Pass 10.13`. Everything on this type is read out of the document and
/// nothing is inferred; see [`read_empty_signature_fields`] for where each
/// value comes from and for the two keys the engine models and
/// `pdfcer_core::forms::Field` does not.
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
    /// ★★★ Disclosed **before** the press, not in the summary afterwards. The
    /// engine reports it on `SignReport::field_lock` and this shell shows that
    /// too, but a consequence an operator learns about after the file is written
    /// is a consequence he did not consent to.
    pub locks: Option<String>,
    /// Whether the field carries an `/SV` seed-value dictionary (Table 234) —
    /// conditions the author attached to signing it.
    ///
    /// A boolean rather than the parsed constraints, deliberately. `/SV` is
    /// seven `/Ff` bits over five entry families and the engine evaluates all of
    /// them; re-deriving that here would be a **second** answer to a question
    /// with one answer, and the two would disagree the first time either
    /// changed. What this shell owes the operator before the press is *"the
    /// sender attached conditions to this box"*; what the conditions ARE is the
    /// engine's sentence, arriving by name if one is unmet.
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
///
/// `Pass 10.13`'s input. Signed fields are excluded — the engine refuses to
/// re-sign one (`SignApplyError::FieldAlreadySigned`) and offering it would be
/// an option whose only outcome is a refusal — and so is anything that is not a
/// signature field, which is a different refusal
/// (`SignApplyError::FieldNotSignature`) and equally not worth offering.
///
/// # ★★ Two of the five values are read from the raw dictionary, and that is
/// not a shortcut
///
/// `pdfcer_core::forms::Field` models `/FT`, `/T`, `/V`, `/Kids` and the
/// widgets' rectangles, and it models **neither `/Lock` nor `/SV`** — the engine
/// reads both directly off the field dictionary inside its own signing path
/// (`EditSession::reusable_sig_field`), where they are consumed rather than
/// projected. So there is no projection to read them from, and this function
/// asks the object graph the same question the engine asks.
///
/// ★ It asks only whether they are **present**, never what they say. Parsing
/// `/SV`'s seven `/Ff` bits here would be a second implementation of a rule the
/// engine enforces in full — see [`SigField::constrained`].
///
/// `pages` is the flattened page vector, passed rather than re-walked, so a
/// widget's `/P` can be turned into the page number an operator counts.
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
            // ★ `merged` is the projection's own answer to the same question
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
    ///
    /// `pages` is passed rather than re-derived, because `OpenDoc::pages` is
    /// *"the flattened page vector, resolved once at open"* and re-walking the
    /// tree here would be a second answer to a question the document already
    /// has one answer to.
    ///
    /// ⚠ `pages` changed shape on 2026-09-06 from a count to the vector:
    /// [`read_empty_signature_fields`] needs the page **identities** to turn a
    /// widget's `/P` into the number an operator counts, and re-walking the tree
    /// here would be a second answer to a question the document has one answer
    /// to. The count is taken from it.
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
    ///
    /// Pure, so both arms are asserted headlessly. §2d: both of the engine's
    /// certification refusals are states of the **document**, knowable when the
    /// window opens, so the option is absent with a sentence rather than offered
    /// and then refused.
    ///
    /// ★ The order is the engine's own guard order — `AlreadyCertified` is
    /// checked before `CertificationNotFirst` — so a document that is both
    /// gets the same sentence here that it would get from the engine. Two
    /// surfaces disagreeing about which of two true things to say is how an
    /// operator learns to distrust both.
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
    ///
    /// Pure, so every arm is asserted headlessly rather than by driving a
    /// window. See §4 of this module's header for the table and for the one
    /// refusal that is this shell's rather than the engine's.
    ///
    /// # Order
    ///
    /// The order is *how early the operator can act on it*, not severity.
    /// A pending redaction is one press away from being applied or cancelled,
    /// so it is named first even though encryption is the harder wall: telling
    /// somebody about the wall when the gate beside it is merely latched wastes
    /// the one sentence they will read.
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
///
/// A closed set with one sentence each in [`crate::text::sign`]. Every variant
/// but [`Self::NotOnDisk`] mirrors a `SignApplyError` the engine would raise;
/// stating them here means the operator meets the refusal **instead of** a
/// form, rather than after filling one in.
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
///
/// Distinct from [`Refusal`] and it must stay distinct: every [`Refusal`]
/// closes the window, and each of these closes exactly one option on a window
/// that still works. Flattening them would turn *"you cannot be the author of
/// this document, but you can approve it"* into *"this document cannot be
/// signed"*, which is false and is the more expensive direction to be wrong in.
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
///
/// ⚠ **No `Debug`, derived or hand-written, and that is the point.** The
/// engine's [`Pkcs12Signer`] has a careful hand-written one that prints the
/// report and never the key, so a derive here would in fact be safe today. It
/// is still absent, because the thing that protects `crate::secret::Secret` is
/// that *the value cannot be formatted* — a property that survives somebody
/// adding a field. A `Debug` on the container is one refactor away from
/// printing whatever is put next to the signer.
///
/// Use [`Self::report`] for anything a human or a trace needs.
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
    /// ★ A read failure and a parse failure are **different** answers, because
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
            // ★★★ WHAT IS ABSENT HERE IS THE DESIGN. No passphrase, no length
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
///
/// ★★★ THE DEFAULT IS INVISIBLE, AND IT IS A DECISION RATHER THAN A COPY OF
/// THE ENGINE'S.
///
/// `SignRequest::visible`'s own documentation says invisible *"is the default
/// for batch/CLI signing"*, which is an argument about batches. The argument
/// here is about what would be drawn on a CAD sheet the operator is about to
/// send out: **a box is applied content**, it renders exactly as the saved file
/// renders, and there is nothing provisional about it. So the default draws
/// nothing, the box is offered, and the copy on the control says what will be
/// inside it before it is chosen.
///
/// ★★ **What is inside it changed under this shell on 2026-09-06.** At the old
/// pin the appearance was *"a thin frame only — no text"*, and this type's
/// documentation and [`crate::text::sign::placement_note`] both said so. Engine
/// `Pass 10.14` (`187fa09`, in the pin since `d6b998f`) **composes** the signer
/// CN, the date, and the reason and location when given, in Helvetica, shrunk to
/// fit, and refuses a rectangle too small for them by name
/// (`SignApplyError::AppearanceOverflow`) before anything is staged. See §2b.
///
/// ⇒ The default is still invisible, and the argument for that survives the
/// correction intact but is now a **different** argument: not *"the box would be
/// empty and read as a defect"* but *"a signature the reader shows in its own
/// panel does not need a stamp on the drawing, and a stamp is content the
/// operator did not draw."* An operator who wants the box now gets a box with
/// his name in it.
///
/// ★★★ **The third arm is not a placement at all, and that is the point.**
/// [`Self::ExistingField`] names a box **somebody else already placed**; its own
/// `/Rect` and page decide where the appearance goes, and the engine refuses a
/// `visible` rectangle beside it by name. Modelling all three as one enum makes
/// the refused combination unrepresentable rather than reachable-and-explained.
///
/// ⚠ **Not `Copy`**, because [`Self::ExistingField`] owns the field's name. The
/// name is carried rather than an index into [`Standing::empty_fields`] for the
/// reason every stale-index bug has: the vector is read once when the window
/// opens and the request is built later, and an index that survives into a list
/// that changed points at the wrong field silently, while a name that no longer
/// exists is refused by the engine, by name.
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
///
/// ★★ Every number here is stated rather than tuned, because this is content
/// written into the operator's file and *"about a third of the way up"* is not
/// a specification anyone can check. 36 pt is a half-inch margin — the same
/// inset a title block leaves and the value ISO 32000-1's own examples use;
/// 180 × 60 is the box Acrobat's own signature appearance defaults to at 100 %,
/// which is the size an operator's eye already expects.
///
/// ★ Bottom-**right** rather than bottom-left because a CAD sheet's title block
/// is bottom-right and a signature belongs beside it — and because the
/// alternative, bottom-left, is where every drawing frame in this operator's
/// own files puts its revision table.
///
/// ⚠ It is clamped to the page: on a page smaller than 252 × 132 pt the box
/// would otherwise be placed partly or wholly outside the media box, which the
/// engine would accept and no reader would draw. The clamp is
/// [`Self`]-contained arithmetic on `media` rather than a refusal, because a
/// small page is not an error and a signature on it is still wanted.
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
///
/// Everything on this struct is the operator's own words or the operator's own
/// choice. pdfcer infers nothing into a signature dictionary — the engine's
/// rule-4 note says the reason: *"the signing time, name, reason, location and
/// contact are the caller's words, written verbatim."*
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
    /// ★ The engine's own type, `MdpPermission`, rather than a local mirror.
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
    ///
    /// Temp file, then rename — `crate::protect::Prepared::write_to`'s
    /// mechanism, taken deliberately. The destination may be the file the
    /// operator has open, and a torn write there leaves them with neither the
    /// signed document nor the one they started with.
    ///
    /// # Errors
    ///
    /// [`WriteFailure`] — the file system refused.
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
            // ★ `path` is Debug-quoted for `protect-written`'s reason: a
            // Windows path routinely contains a space, and a consumer splitting
            // the line into `key=value` pairs would lose every field after it.
            //
            // ⚠ `subject` and `serial` are NOT here. They are on screen, in the
            // report the operator reads, which is where a disclosure about
            // whose key was used belongs. A trace file is kept and shared.
            //
            // ★★ `field_reused=` is here as well as on `sign-prepared`, and the
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
///
/// The handler produces this and hands it back through
/// [`crate::dialogs::DialogsState::sign_outcome`]. A single type with two
/// variants rather than a `Result`, because the *failure* side here is already
/// a finished operator-facing sentence — every producer of one has more context
/// than the dialog does about which of five things went wrong — and a `Result`
/// whose error is a `String` invites a caller to add its own wording on top,
/// which is how one event comes to be described twice.
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
///
/// The one place `EditSession::sign` is called. See §3 for why it is the open
/// session and not a throwaway, and why nothing is undone afterwards.
///
/// # ★ The reservation is the engine's default and is not offered as a control
///
/// `SignRequest::reserve` defaults to 12 KiB, which the engine's own note says
/// *"fits a SHA-256/RSA-4096 CAdES signature with a three-certificate chain
/// about three times over"*. It is not a question an operator can answer — the
/// number is a property of their certificate chain, which pdfcer has just read
/// and they have not — so asking would be handing them arithmetic. If it is
/// ever too small the engine refuses by name with **both** numbers, and
/// [`crate::text::sign::reservation_too_small`] states that the reservation is
/// fixed, so the refusal is a fact rather than an instruction the operator
/// cannot follow.
///
/// ⚠ The alternative considered and rejected was **retrying automatically at a
/// larger reserve**. It would work, and it would mean the size of the hole in
/// the operator's file depended on a retry they were never told about. R8b
/// Rule 4: what is written is disclosed.
///
/// # Errors
///
/// [`PrepareFailure`] — the document is out of scope, or the engine refused.
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
    // ★ `/Name` is deliberately NEVER set — see `crate::text::sign`'s header.
    // The engine: "`None` omits the key and a verifier falls back to the
    // certificate subject (Table 252 says it should anyway)." A free-text name
    // beside a certificate is a second, unverifiable claim about who signed.
    // ★★★ `Pass 10.12`. Written before the placement, so the request is
    // assembled in the order the engine guards it: certification is refused
    // before any field is resolved.
    request.certify = authored.certify;
    request.visible = match &authored.placement {
        Placement::Invisible => None,
        // ★★★ `Pass 10.13`: the field's OWN `/Rect` and page place the
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
            // ★ The CROP box, not the media box, and the difference is what
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
        // ★ `into_field=` is a BIT, not the field's name. A field name is text
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
        // ★★ `field_reused=`, `locked=`, `notes=` and `certified=` are what
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
    ///
    /// Private-in-spirit: it is `pub(crate)` rather than `pub` so that the key
    /// operation is reachable from [`prepare`] and from nowhere a future module
    /// might casually put it.
    pub(crate) fn signer_ref(&self) -> &dyn pdfcer_core::sign::Signer {
        &self.signer
    }
}

/// `None` for a field the operator left alone.
///
/// ★ Trims first. A field holding one space is an untouched field as far as
/// anybody looking at the screen is concerned, and writing `/Reason ( )` into a
/// legal document because of a stray keystroke is the kind of thing nobody ever
/// finds.
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
pub fn suggested_path(source: &Path) -> PathBuf {
    let stem = source.file_stem().map_or_else(
        // ui-text-exempt: a filename fallback for a path with no stem, not
        // operator copy. Every sibling suggestion function makes the same one.
        || String::from("document"),
        |s| s.to_string_lossy().into_owned(),
    );
    let named = format!("{stem}{}.pdf", crate::text::sign::suggested_suffix());
    source
        .parent()
        .map_or_else(|| PathBuf::from(&named), |parent| parent.join(&named))
}

#[cfg(test)]
mod tests;
