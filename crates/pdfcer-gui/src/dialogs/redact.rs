//! # `dialogs::redact` — the Apply-redactions transaction
//!
//! The body of `edit.redact_apply`, and the **irreversible** half of the
//! redaction feature. Its reversible twin is [`crate::panels::redact`], and the
//! split between them is the distinction
//! `crate::text::commands::edit_redact`'s shipped tooltip already draws:
//! *"Marking is reversible; applying is not."*
//!
//! This is the only surface in pdfcer-gui that commits an operation nothing can
//! take back, and its whole shape follows from that.
//!
//! ## The five states
//!
//! | state | what the operator sees | what exists |
//! |---|---|---|
//! | **prepared** | the measured report, a destination choice, up to three checkboxes, and a control whose label is the consequence | the finished redacted bytes, **in memory** |
//! | **staged** | that a removal is already armed, what that means, and one control that calls it off | a flag on the session, and nothing else |
//! | **refused** | a named refusal, and nothing to confirm | nothing |
//! | **written** | where the file went, whether it replaced the open one, and what is still in it | a file |
//! | **write failed** | why no file appeared | nothing |
//!
//! ★★★ **`staged` is new on 2026-09-05** and it is the state a document is in
//! after the default destination has been confirmed (`Pass 250.2`). Its body is
//! [`staged`]'s, and that module's header carries the seam and the one decision
//! worth reading — that it quotes **no numbers**, because the removal re-runs at
//! save and a measurement taken at the moment of consent is stale by the time
//! this phase is drawn.
//!
//! There is deliberately no *ready* state. Opening this dialog **runs the whole
//! removal** — see §2 — so by the time anything is drawn the numbers on screen
//! are measurements of the exact bytes that will be written, not predictions
//! about bytes that do not exist yet.
//!
//! ## ★ 1. Why the report comes BEFORE the write, and the operator chooses the
//! destination
//!
//! `crate::dialogs::ocr`'s argument, one operation further along the scale of
//! consequence. That dialog recognises, discloses what it inferred, and only
//! then offers to save — so *"the operator reads the disclosure while holding
//! the one thing that gives it force: the ability to not save."*
//!
//! Here the disclosure is not about inference, it is about **what will be
//! destroyed and what pdfcer could not destroy**, and the residual half of it is
//! the whole reason the feature can be trusted. A surface that redacted and
//! dropped a file picker in front of the operator would be technically
//! disclosive and practically a program that quietly shipped a partially
//! redacted document.
//!
//! ★★★ **2026-09-04 — the destination is the operator's, not this dialog's.**
//! This section used to end by arguing that the write must always be to a new
//! file. The operator overruled that: *"why does it have to save to a new file
//! right away? Why can't it just wait on saving until I choose to save over the
//! existing file or save as a new file?"* [`Destination`] carries the whole
//! argument and what survives of the old ruling (the safe default, and
//! [`suggested_path`] never proposing the source).
//!
//! ★★★ **CORRECTED the same evening.** That paragraph ended, at midday, by
//! naming *"the one half of his request the engine cannot express — deferring
//! the write to a later Save, which would need a redaction that mutates an
//! `EditSession` and there is no such verb."* **There is now.** `Pass 250.1`
//! shipped `EditSession::apply_redactions` the same afternoon, in answer to
//! this shell's filing, and [`Destination::OpenDocument`] is the deferred
//! destination it makes possible — **and it is the default**. There are three
//! destinations now, not two, and the write-now pair is what is left of the
//! original design rather than the whole of it.
//!
//! ## ★ 2. Why the removal runs synchronously, on open
//!
//! It is the salvage source's shape and it is kept, with the trade stated
//! rather than inherited.
//!
//! The alternative is `crate::ocr::Job`'s: a worker thread, a spinner and a
//! poll. Everything needed for it is available — `OpenDoc::session` is an
//! `Arc<EditSession>` and every field of [`crate::redact::PreparedRedaction`]
//! is `Send`. It is not done, for one reason that decides it: **a report
//! computed on another thread is a report about a document that may have
//! changed by the time it is read.** OCR can tolerate that because it refuses
//! outright when `edit_epoch != 0`; a redaction cannot, because the marks the
//! operator is applying are the ones they have just made and the epoch is
//! moving by construction.
//!
//! Running it inside the dispatch that opens the dialog gives the report and
//! the bytes one consistent snapshot, taken at a moment the operator caused. The
//! cost is a frame that takes as long as a full rewrite of the document —
//! visible on a large sheet, and paid once, on a deliberate click, for the one
//! operation in the program where a stale answer would be a security defect.
//!
//! ## ★ 3. What confirmation actually consists of, and why it is not one click
//!
//! Four gates, and each closes a different failure:
//!
//! 1. **[`crate::text::redact::confirm_checkbox`]** — always present. Its
//!    wording targets the exact misunderstanding the feature exists to prevent:
//!    that applying removes the *marks* rather than the *content*.
//! 2. **[`crate::text::redact::residual_acknowledgement_checkbox`]** — present
//!    **only when the report has residuals**. Showing it always would make it a
//!    box operators tick without reading, which is how every acknowledgement in
//!    a program becomes worthless. It is also enforced below the UI, at
//!    [`crate::redact::PreparedRedaction::write_to`], because a greyed control
//!    is a drawing decision and not a mechanism.
//! 3. ★ **[`crate::text::redact::overwrite_acknowledgement_checkbox`]** —
//!    present **only when the operator has chosen to replace the open file**
//!    (2026-09-04). A different fact from gate 1: that one is about the
//!    *content*, this one is about the *document*. Somebody can have taken in
//!    that the text is going for good without noticing that the file they
//!    opened is going with it. Conditional for gate 2's reason — a box that is
//!    always there is a box that is always ticked.
//! 4. **A control whose label is the consequence** — never "OK", never
//!    "Apply". One label per destination, and the punctuation is part of the
//!    claim: *"Permanently remove & save as…"* on the new-file destination,
//!    where the ellipsis promises the picker that really is coming;
//!    *"… & replace `<name>` now"* on the replace destination, which names the
//!    file and drops the ellipsis because no further question follows; and
//!    *"Set up the removal — it happens when I save"* on the default, which
//!    promises nothing further because no file is involved and **claims no
//!    immediate removal, because there is none**. An ellipsis on a control that
//!    asks nothing more is a lie the operator acts on, and so is a label
//!    claiming a removal that has not happened.
//!
//! ★★★ …and, between the destination choice and the button, **a disclosure
//! rather than a gate**: [`crate::text::redact::removal_happens_at_save`], drawn
//! only on the deferred destination. It is deliberately NOT a fourth checkbox —
//! §3's own argument about conditional boxes applies to their multiplication
//! too, and four acknowledgements is a form, which is filled in rather than
//! read. What the operator is owed here is the FACT, before the click.
//!
//! ★★★ **CORRECTED 2026-09-05.** That sentence used to be
//! `undo_will_be_cleared`, naming the number of undo steps the click would
//! destroy, and the paragraph argued at length that he was owed the count
//! before he committed. He was. `Pass 250.2` preserves the whole undo log, so
//! there is no count and the sentence would be false — and what has replaced it
//! is a *more* surprising fact rather than a lesser one: **the page does not
//! change.** The disclosure's argument is unchanged; only its subject moved.
//!
//! And a fourth thing that is an absence: **no keyboard shortcut, and no Enter
//! binding.** The footer says so in words rather than leaving it to be noticed.
//! Every other destructive verb in this shell is chorded and reversible; this
//! one is neither, and the asymmetry is deliberate.
//!
//! ## ★ 4. The `ready` flag is read one frame late, on purpose
//!
//! [`RedactDialog::show`] computes whether the confirm control may be enabled
//! **before** the checkboxes are drawn, so a checkbox ticked on this frame does
//! not enable the button until the next one. A fast double-click on the box
//! would otherwise land its second press on a control that became enabled
//! between the two — which on this dialog means an irreversible operation
//! reached by a gesture the operator made at a disabled control.
//!
//! ## 5. ★★★ CORRECTED 2026-09-04 (evening) — this dialog pushes an `Action`
//! on exactly one of its three destinations
//!
//! What stood here at midday, and it was right about the world it described:
//!
//! > *"[`super`]'s rule: a dialog uses the action funnel when it edits **this**
//! > document, and this one never does. Applying produces *bytes on disk*; the
//! > open document keeps its marks, its undo log and its epoch whichever
//! > destination was chosen."*
//!
//! [`Destination::OpenDocument`] edits **this** document, so it takes the
//! funnel, and by [`super`]'s own rule rather than despite it. The two
//! write-now destinations are unchanged and still push nothing: they produce
//! bytes on disk and leave the session alone.
//!
//! ★★ **2026-09-05: it is now two of five presses, not one of three.** The
//! deferred destination arms rather than removes (`Pass 250.2`), and the
//! [`Phase::Staged`] phase's *call the removal off* control disarms — and both
//! reach `EditSession` through the same funnel for the same reason, which is
//! that the engine's verbs take `&mut EditSession` and `Arc::get_mut` is the
//! funnel's second step.
//!
//! | press | what it changes | route |
//! |---|---|---|
//! | confirm on [`Destination::OpenDocument`] | the session's pending-redaction flag | `RedactAction::Pending(Staging::Stage)` → `crate::app::actions::redact` → `vector_edit` |
//! | *call the removal off*, in [`Phase::Staged`] | the same flag, back off | `RedactAction::Pending(Staging::Cancel)` → the same arm |
//! | confirm on [`Destination::NewFile`] | a file | [`crate::redact::PreparedRedaction::write_to`], here |
//! | confirm on [`Destination::ReplaceOriginal`] | the source file | the same, atomically |
//!
//! What the funnel's reasoning demanded and still demands is that irreversible
//! work not run part-way through a layout pass — and it does not, on any of
//! them: every control sets a flag, and the push, the picker and the write all
//! happen after the window's closure returns.
//!
//! ★★ **After a replace, the open document is deliberately STALE, and the
//! outcome sentence says so.** The session was not touched, so the canvas goes
//! on drawing the marks and the content underneath them while the file those
//! bytes came from contains neither.
//!
//! ★★★ The reason that used to be given for it — *"`EditSession` has no verb
//! that could"* — is no longer true, and the staleness is now a **consequence
//! of the destination the operator chose** rather than a limit of the program.
//! It is still not tidied away by swapping the session underneath, and the old
//! argument for refusing that manoeuvre stands untouched: a swap discards the
//! whole undo log without saying so, and `crate::app::save::save_as` refuses it
//! for the same reason. An operator who wants the open document to change now
//! has a control that says so. One who chose to write a file gets a file, and
//! the divergence is **disclosed** rather than hidden, in
//! `crate::text::redact::applied_clean`'s replace form, which tells him by name
//! which file to reopen. Rule 4: report separately, and do not pretend.
//!
//! ## 6. It is document-scoped, and closing the document discards the bytes
//!
//! `crate::dialogs::ocr`'s ruling, and it matters more here: a redaction is of
//! *these marks* on *this file*, and writing prepared bytes after the operator
//! has put the document away would produce a redacted file derived from a
//! document nobody is looking at any more.

mod staged;

use std::path::{Path, PathBuf};

use egui_shell::theme::Theme;

use crate::app::state::{OpenDoc, Status};
use crate::redact::{
    PreparedRedaction, RedactApplyRefusal, ResidualAcknowledgement, WriteRefusal,
    prepare_redaction_apply,
};
use crate::text::redact as t;

// ---------------------------------------------------------------------------
// Named regions
//
// Matched LITERALLY by `tools/ui-verify/src/checks/redaction.rs`, so renaming
// one silently un-aims the check that measures it. See `crate::dialogs::ocr`'s
// equivalent block for why a dialog needs these when a ribbon control gets its
// rect for free.
// ---------------------------------------------------------------------------

/// The whole window.
const REGION_DIALOG: &str = "redact-apply-dialog"; // ui-text-exempt: trace region name, never displayed

/// The mandatory confirmation checkbox.
const REGION_ACK: &str = "redact-apply-ack"; // ui-text-exempt: trace region name, never displayed

/// The extra acknowledgement, declared **only while it exists** — which is
/// itself the assertion a harness wants, since its presence is evidence that
/// the report disclosed a residual.
const REGION_RESIDUAL_ACK: &str = "redact-apply-residual-ack"; // ui-text-exempt: trace region name, never displayed

/// The control that commits.
const REGION_CONFIRM: &str = "redact-apply-confirm"; // ui-text-exempt: trace region name, never displayed

/// The *replace the original* destination choice, declared **only while the
/// document has an original to replace** — so its absence from a trace is
/// evidence about the document rather than about the build.
const REGION_DESTINATION_REPLACE: &str = "redact-apply-destination-replace"; // ui-text-exempt: trace region name, never displayed

/// The third acknowledgement, declared **only while it is being asked for** —
/// i.e. only while the operator has chosen to replace the original.
const REGION_OVERWRITE_ACK: &str = "redact-apply-overwrite-ack"; // ui-text-exempt: trace region name, never displayed

/// The *this document* destination choice — the default since 2026-09-04.
///
/// Declared **unconditionally**, unlike its two siblings, and that asymmetry is
/// the assertion: this destination is available on every document, including
/// one created in this session that has no file to replace, so its ABSENCE
/// from a trace is evidence about the build rather than about the document.
const REGION_DESTINATION_INTO_DOCUMENT: &str = "redact-apply-destination-into-document"; // ui-text-exempt: trace region name, never displayed

/// The *a new file* destination choice, also declared unconditionally.
///
/// ★ Published so `tools/ui-verify` can **click** it. Its redaction check
/// drives the whole feature to a file — that the source was not touched, that
/// the output lacks the secret, that a second process extracts nothing from it
/// — and the default destination produces no file at all, so the harness has to
/// move off the default deliberately and needs a rect to move to.
const REGION_DESTINATION_NEW_FILE: &str = "redact-apply-destination-new-file"; // ui-text-exempt: trace region name, never displayed

/// The staging disclosure, declared **only while it is on screen** — i.e. only
/// while the deferred destination is selected.
///
/// ★ It is a region rather than only a string so a harness can assert that the
/// sentence is *above the confirm control*, which is the whole of its value:
/// `tools/ui-verify`'s redaction check can compare this rect's bottom against
/// [`REGION_CONFIRM`]'s top and fail if the disclosure ever moves below the
/// button it is meant to precede.
///
/// ★★ **Renamed from `redact-apply-undo-note` on 2026-09-05**, with
/// `tools/ui-verify/src/checks/redaction.rs` in the same commit. The old name
/// described the sentence that used to live here — *"this clears your undo
/// history"* — and `Pass 250.2` made that false; a region name that still said
/// `undo` would have aimed a harness at a sentence about undo and found one
/// about staging, which is the shape of a check that passes while measuring
/// something else. The geometry assertion it carries is unchanged.
const REGION_STAGING_NOTE: &str = "redact-apply-staging-note"; // ui-text-exempt: trace region name, never displayed

/// Height kept clear below the report for the checkbox and button rows.
const FOOTER_RESERVE: f32 = 150.0;

/// The least height the report may be given.
///
/// Without a floor, a small window produces a scroll area that draws **nothing
/// at all** — `available_height()` minus a reservation goes negative, and a
/// negative `max_height` is a silently empty area rather than an error. On this
/// dialog that would be a confirmation with no report above it, which is the
/// one shape it must never take. The About and OCR dialogs record the same
/// trap.
const REPORT_FLOOR: f32 = 120.0;

/// Where one apply transaction has got to.
///
/// A state machine rather than several `Option`s, because the states are
/// mutually exclusive and an `Option` quadruple has combinations that would all
/// compile and none of which means anything.
#[derive(Debug)]
enum Phase {
    /// The removal ran, the proof passed, and the bytes are waiting for a
    /// confirmation. `Box`ed because this variant is far larger than its
    /// siblings and a `match` on the enum would otherwise move the whole
    /// document around.
    Prepared(Box<PreparedRedaction>),
    /// The apply was refused before anything was written.
    Refused(RedactApplyRefusal),
    /// ★★★ **A removal is already armed on this document** (`Pass 250.2`, new
    /// 2026-09-05).
    ///
    /// A phase of its own rather than a [`Self::Refused`] carrying
    /// [`RedactApplyRefusal::AlreadyStaged`], and the two questions are why: an
    /// un-staged document asks *"shall I?"* and a staged one asks *"what did I
    /// already decide, and can I change my mind?"*. A refusal answers the first
    /// question badly instead of the second one well, and — critically — a
    /// refusal has no control on it, which would leave the operator staring at
    /// the reason he cannot save with nothing to press.
    ///
    /// It carries no data. Everything the phase says is true of any staged
    /// document, and the one thing it might have carried — the preview report —
    /// is a **stale measurement** by the time this phase is drawn: the removal
    /// re-runs at the save over whatever the document says then. Quoting it
    /// here would present yesterday's numbers as today's, which is precisely
    /// what `crate::redact::StagedRedaction`'s own doc comment warns about.
    Staged,
    /// The bytes reached this path.
    ///
    /// ★ It carries the three numbers the outcome sentence needs rather than
    /// the [`PreparedRedaction`] they came from. Keeping the prepared value
    /// alive after the write would mean holding a second copy of a redacted
    /// document in memory for as long as the operator leaves the window open,
    /// for no purpose — the bytes are on disk and cannot be written twice from
    /// here. The counts are what the sentence is about.
    ///
    /// `residuals` is the field that decides **which** sentence: the catalog's
    /// rule 1 is that a leftover is named in the same sentence as the success,
    /// so a zero and a non-zero here are two different pieces of copy rather
    /// than one with a number in it.
    Written {
        /// Where the operator put it.
        path: PathBuf,
        /// Whether that path was the document that is open — i.e. whether the
        /// source file was replaced rather than a copy written beside it.
        ///
        /// Carried rather than re-derived by comparing `path` to `source`,
        /// because the outcome sentence must describe **what happened**, and a
        /// comparison performed later answers a question about the paths as
        /// they are now. It is also the difference between two sentences that
        /// say opposite things about the window the operator is looking at.
        replaced: bool,
        /// `RedactionReport::marks_applied`.
        regions: u64,
        /// `RedactionReport::pages_redacted`.
        pages: usize,
        /// How many items the report disclosed as NOT removed.
        residuals: usize,
    },
    /// A destination was named and no file appeared.
    WriteFailed(WriteRefusal),
}

/// **Where the redacted document goes.**
///
/// ★★★ Added 2026-09-04, on the operator's explicit instruction, and it
/// reverses a ruling this file used to state as settled. His words:
///
/// > *"why does it have to save to a new file right away? Why can't it just
/// > wait on saving until I choose to save over the existing file or save as a
/// > new file?"*
///
/// # What this file used to say, and why it was wrong
///
/// [`RedactDialog::commit`] read, verbatim: *"There is no 'save over the
/// original' branch to find, because there is none to write, and on this
/// operation that is the difference between a copy and the destruction of the
/// only remaining source of the content being removed."*
///
/// The premise is true and the conclusion did not follow. Overwriting the
/// source **is** the destruction of the only remaining copy — but the person
/// entitled to decide that is the person who marked the content for
/// destruction in the first place, and forcing a copy does not protect him from
/// the decision, it only makes him perform it in two steps with a stray file
/// left over. Every other edit in this shell trusts him with Save and Save As
/// on exactly this reasoning; the redaction had quietly taken the decision away
/// on his behalf.
///
/// ★ What the old ruling was *actually* protecting is kept, and kept in the
/// form it belongs in: [`Self::NewFile`] is still the **default**, and
/// `crate::dialogs::redact::suggested_path` still never suggests the source. A
/// safe default is a mechanism; a warning is something to click past. The
/// change is that the safe default is now a default rather than the only
/// option.
///
/// # ★★★ CORRECTED the same evening — the deferred half SHIPPED, and this
/// section used to say it could not
///
/// What stood here, verbatim, written at about midday:
///
/// > *"⚠ What this deliberately does NOT do, and why. He asked for the write to
/// > be deferred — applied into the session, saved later by Save or Save As
/// > like any other edit. **The engine cannot express that**, and this dialog
/// > does not fake it. [`pdfcer_core::redact::apply_redactions`] takes a
/// > `&Document` and returns `Vec<u8>`; `EditSession`'s only constructor is
/// > `new(Document)` and it has no `replace_document`, no `rebase` and no
/// > `reload`."*
///
/// Every clause of that was true when it was written and was filed as an engine
/// request the same morning. **The engine answered it that afternoon**:
/// `EditSession::apply_redactions` (`Pass 250.1`, `225db51`) applies the
/// removal into the session and leaves the write to the ordinary save verbs. So
/// the paragraph is not softened, it is **replaced** — [`Self::OpenDocument`]
/// is the destination it said was impossible, and it is now the default.
///
/// ★ What the old paragraph got right and is worth keeping: the manoeuvre it
/// refused — *"building a second `EditSession` and swapping it under the open
/// document"* — is still refused, and the engine did not ship that either. Its
/// verb collapses the session in place, keeps the document identity, and clears
/// the undo log **by name** rather than by accident, which is the difference
/// between a disclosed consequence and a silent data loss. The refusal was
/// right; only its conclusion about what could exist was wrong.
///
/// The request is at `D:\Dev\FeatureRequests\pdfce_FeatureRequests\
/// open\request_apply_redactions_into_the_session.md` and the reply beside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Destination {
    /// ★★★ **The open document, with nothing written — the default since
    /// 2026-09-04 (evening), and the thing he actually asked for.**
    ///
    /// `crate::redact::stage_into_session`: the removal is armed, and
    /// `file.save` / `file.save_as` / `file.save_copy` carry it out, exactly as
    /// they carry every other edit to a file.
    ///
    /// It is the **default** because it is the only one of the three that
    /// writes nothing. The old default ([`Self::NewFile`]) was safe because it
    /// never overwrote; this is safer still, because it never writes.
    ///
    /// ★★★ **Its price was inverted on 2026-09-05, and the old price is worth
    /// recording because it is what the operator agreed to.** Under
    /// `Pass 250.1` this destination **finalized**: the removal happened at the
    /// click and the whole undo log went with it, disclosed above the confirm
    /// control as a step count, on his ruling *"finalizing the document and
    /// can't be undone is ok **for now**"*. `Pass 250.2` charges nothing —
    /// base, overlay and the entire undo/redo stack survive — and what is
    /// disclosed in the same place is the surprise that replaced the price:
    /// [`crate::text::redact::removal_happens_at_save`], because **the page
    /// does not change**.
    OpenDocument,
    /// A new file, chosen in the save picker.
    NewFile,
    /// The document that is open, replaced in place.
    ///
    /// Offered only when the source is a real file on disk — a document
    /// created in this session has no original to replace, and a control
    /// meaning "replace nothing" is worse than an absent one.
    ReplaceOriginal,
}

/// ★★★ **The destination a freshly-opened dialog starts on.**
///
/// A named constant rather than a literal inside [`RedactDialog::open`], so the
/// property that actually matters — *the default writes nothing* — can be
/// asserted without constructing a document, and so that changing it is a
/// visible edit rather than one word in a struct literal.
///
/// It moved on 2026-09-04 from [`Destination::NewFile`] to
/// [`Destination::OpenDocument`]. Both are safe defaults and for different
/// reasons: the old one never *overwrote*, the new one never *writes*.
const DEFAULT_DESTINATION: Destination = Destination::OpenDocument;

impl Destination {
    /// Whether this destination writes a file **now**, rather than leaving the
    /// write to a later Save.
    ///
    /// A method rather than three `== ` comparisons scattered through
    /// [`RedactDialog`], because five separate places ask the same question —
    /// which permanence sentence, which button label, which acknowledgements
    /// are owed, whether the picker opens, and whether an `Action` is pushed —
    /// and a fourth destination added later must be answered once rather than
    /// found five times.
    const fn writes_now(self) -> bool {
        matches!(self, Self::NewFile | Self::ReplaceOriginal)
    }
}

/// The Apply-redactions dialog.
#[derive(Debug)]
pub struct RedactDialog {
    /// The document's own path, for suggesting a name to save under.
    ///
    /// Captured on construction rather than read per frame, for
    /// `crate::dialogs::ocr`'s reason applied to the file rather than to the
    /// page: nothing can change it while the dialog is open, and reading it
    /// from a `&OpenDoc` at save time would make the suggestion depend on a
    /// borrow the write path does not otherwise need.
    source: PathBuf,
    /// The transaction's state.
    phase: Phase,
    /// The mandatory acknowledgement.
    acknowledged: bool,
    /// The extra acknowledgement, meaningful only when the report has
    /// residuals.
    ///
    /// Two flags rather than one, deliberately: they answer different
    /// questions, and a single flag would let an operator who understood the
    /// permanence be treated as having read a residual list they were never
    /// shown.
    residuals_acknowledged: bool,
    /// Where the redacted document goes. [`Destination::OpenDocument`] until
    /// the operator says otherwise — see that type for the whole argument, and
    /// for why the default moved on 2026-09-04.
    destination: Destination,
    /// The **third** acknowledgement: that replacing the original destroys the
    /// last copy of the content being removed.
    ///
    /// ★ A third flag rather than folding it into
    /// [`Self::acknowledged`], on this dialog's own standing reason for keeping
    /// the first two apart: they answer different questions, and a shared flag
    /// would let an operator who ticked one be treated as having read the
    /// other. Here the asymmetry is sharper still — the permanence box is about
    /// the *content*, and this one is about the *file*. A person can perfectly
    /// well understand that the text is going for good and not have noticed
    /// that the document they opened is going with it.
    ///
    /// It is only *asked for* while [`Destination::ReplaceOriginal`] is
    /// selected, and only *required* then. Left ticked from an earlier
    /// selection it is harmless, because the destination it applies to is read
    /// at the same instant.
    overwrite_acknowledged: bool,
    /// Set by the confirm control, consumed by [`Self::show`] after the
    /// window's closure returns.
    ///
    /// The two-step every dialog here uses, for a stronger reason than most:
    /// this is the irreversible half, and an `rfd` modal opened from inside an
    /// `egui::Window` closure blocks the frame it is being drawn in.
    confirm_requested: bool,
    /// Set by the *Call the removal off* control in [`Phase::Staged`], consumed
    /// by [`Self::show`] after the window's closure returns.
    ///
    /// A second flag rather than a reuse of [`Self::confirm_requested`], for
    /// the reason this file keeps its acknowledgement flags apart: the two
    /// presses are opposite acts on the one operation that cannot be undone
    /// once it reaches a file, and a shared flag with a phase test would make
    /// *arm* and *disarm* one code path distinguished by state.
    cancel_requested: bool,
    /// Set by the Close control; same two-step, because a widget drawn from the
    /// state cannot drop the state it is being drawn from.
    close_requested: bool,
}

impl RedactDialog {
    /// **Prepare the redaction and build the dialog around the answer.**
    ///
    /// The whole removal runs here — see §2 — so this call is as expensive as a
    /// full rewrite of the document, once, on a deliberate click.
    fn open(doc: &OpenDoc) -> Self {
        let phase = match prepare_redaction_apply(&doc.session) {
            Ok(prepared) => {
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        //
                        // Emitted at PREPARE rather than only at write, so a
                        // harness can tell "the removal ran and the operator
                        // did not confirm" from "the removal never ran". The
                        // two look identical from the file system.
                        "redact-prepared marks={} pages={} glyphs={} streams={} checked={} \
                         short={} residuals={} verified={} bytes={}",
                        prepared.report.marks_applied,
                        prepared.report.pages_redacted,
                        prepared.report.glyphs_removed,
                        prepared.report.content_streams_rewritten,
                        prepared.verification.strings_checked,
                        prepared.verification.strings_too_short_for_raw_check,
                        prepared.verification.residuals.len(),
                        prepared.verification.is_clean(),
                        prepared.byte_len(),
                    )
                });
                Phase::Prepared(Box::new(prepared))
            }
            // ★★★ The staged state arrives as a REFUSAL from the pipeline and
            // is turned into a phase here, rather than being detected by asking
            // `doc.session.has_pending_redaction()` before the call.
            //
            // That is deliberate and it is this project's standing preference
            // for a mechanism over a condition. `prepare_redaction_apply`
            // refuses `AlreadyStaged` by name because it must — while a removal
            // is armed the engine declines `to_full_bytes`, and without the
            // named refusal the operator would read *"this document cannot be
            // rewritten in full"* — so the fact is established in the pipeline
            // whatever this dialog does. Asking the session again here would be
            // a second, independent test of the same condition, and the day the
            // two disagreed the dialog would be the one that was wrong.
            Err(RedactApplyRefusal::AlreadyStaged) => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "redact-already-staged".to_owned()
                });
                Phase::Staged
            }
            Err(refusal) => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("redact-refused reason={refusal:?}")
                });
                Phase::Refused(refusal)
            }
        };
        Self {
            source: doc.path.clone(),
            phase,
            acknowledged: false,
            residuals_acknowledged: false,
            destination: DEFAULT_DESTINATION,
            overwrite_acknowledged: false,
            confirm_requested: false,
            cancel_requested: false,
            close_requested: false,
        }
    }

    /// Draw one frame. Returns `false` when the dialog should close.
    pub(super) fn show(
        &mut self,
        ctx: &egui::Context,
        _doc: &OpenDoc,
        actions: &mut Vec<crate::app::actions::Action>,
    ) -> bool {
        // ★ §4 — read BEFORE the body draws its checkboxes, so a box ticked on
        // this frame does not enable the confirm control until the next one.
        let ready = self.ready_to_confirm();

        // ★ ITS OWN OS WINDOW as of 2026-08-21, and of every dialog in this
        // directory this is the one where being able to move it off the
        // document matters most: the report lists what will be REMOVED, and
        // checking it against the page underneath was impossible while the
        // window covered that page.
        //
        // ★ The dialog region is published from inside the callback now — the
        // window response it used to come from no longer exists, and
        // `dialogs::host` tags what is published with this viewport so the
        // harness can convert it.
        let (frame, ()) = crate::dialogs::host::Host::new(
            "redact-apply", // ui-text-exempt: a viewport key, never displayed.
            t::apply_title(),
            egui::vec2(760.0, 560.0),
            egui::vec2(480.0, 320.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_DIALOG, ui.max_rect());
            self.body(ui, ready);
        });
        let open = !frame.closed;

        // The irreversible half, after the closure. See `confirm_requested`.
        if std::mem::take(&mut self.confirm_requested) {
            self.commit(actions);
        }
        // ★ …and the reversible one, which still waits for the closure to
        // return. See [`Self::take_cancel`].
        self.take_cancel(actions);
        open && !std::mem::take(&mut self.close_requested)
    }

    /// **Consume the *call the removal off* press, if there was one.**
    ///
    /// A method rather than four lines inside [`Self::show`], and the reason is
    /// this suite's standing one: [`Self::show`] needs an `egui::Context` and a
    /// real viewport, so nothing inside it can be asserted headlessly, and the
    /// one thing worth asserting about this press is **which action it
    /// raises**. A build that raised [`crate::redact::Staging::Stage`] here
    /// would re-arm the removal the operator just asked to call off, silently,
    /// on a control whose label says the opposite.
    ///
    /// ★ It pushes an `Action` rather than touching the session. The engine's
    /// `cancel_pending_redaction` takes `&mut EditSession`, `Arc::get_mut` is
    /// the funnel's second step, and performing that from inside a dialog's
    /// draw is exactly what the funnel exists to prevent.
    ///
    /// ★ It closes the window. The outcome is reported by the funnel's edit
    /// disclosure like any other edit, and a window left open beside it would
    /// be a second account of one event — and, worse, an account of a state the
    /// document is no longer in, since this phase exists only while a removal
    /// is armed.
    fn take_cancel(&mut self, actions: &mut Vec<crate::app::actions::Action>) {
        if !std::mem::take(&mut self.cancel_requested) {
            return;
        }
        actions.push(crate::app::actions::Action::Redact(
            crate::app::actions::RedactAction::Pending(crate::redact::Staging::Cancel),
        ));
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            "redact-cancel-requested".to_owned()
        });
        self.close_requested = true;
    }

    /// Whether the confirm control may be enabled.
    ///
    /// Pure, and the whole of the gate's rule — so every property of it is
    /// asserted headlessly, which is `crate::viewer`'s standing split applied
    /// to the one control in the program that must not be enabled early.
    fn ready_to_confirm(&self) -> bool {
        let Phase::Prepared(prepared) = &self.phase else {
            return false;
        };
        self.acknowledged
            && (residual_lines(prepared).is_empty() || self.residuals_acknowledged)
            // ★ The overwrite acknowledgement is owed by ONE destination, and
            // it is spelled as that destination rather than as "not NewFile".
            // The negative form was correct while there were two choices and
            // became wrong the moment there were three — it would have demanded
            // an overwrite acknowledgement from the destination that overwrites
            // nothing, which is a control the operator cannot satisfy because it
            // is not on screen.
            && (self.destination != Destination::ReplaceOriginal || self.overwrite_acknowledged)
    }

    /// ★★★ **The staging disclosure, or nothing** — the sentence drawn between
    /// the destination choice and the confirm control.
    ///
    /// Pure, and a method rather than three lines inside [`Self::gates`], for
    /// [`Self::choose_destination`]'s reason: this is the disclosure that
    /// stands between a button labelled *"the removal happens when I save"* and
    /// an operator who has never used a redaction tool that did not blacken the
    /// page instantly. A property that load-bearing is asserted headlessly
    /// rather than left to a reading of the draw order.
    ///
    /// ★★★ **What it says was inverted on 2026-09-05.** It used to be
    /// `undo_will_be_cleared(self.undo_depth)` — the price of `Pass 250.1`'s
    /// collapsing verb, the number of undo steps the click would destroy — and
    /// there is no such price any more. What replaced it is not a smaller
    /// version of the same warning: it is the opposite fact, that **nothing
    /// happens on screen**, which is more surprising than the loss it replaces
    /// and is the one thing an operator cannot work out by looking. See
    /// [`t::removal_happens_at_save`].
    ///
    /// `None` on the two write-now destinations, and that is a claim rather
    /// than an omission: those routes do produce a file at the click, so a
    /// sentence saying nothing is written would be false there.
    fn staging_disclosure(&self) -> Option<&'static str> {
        // ★ Asked as *"does this write a file?"* rather than by naming the
        // variant: the disclosure belongs to the destination that defers, which
        // is precisely the one that does not write, and a fourth deferred
        // destination would inherit it rather than have to be remembered here.
        (!self.destination.writes_now()).then(t::removal_happens_at_save)
    }

    /// Whether replacing the open document is an option at all.
    ///
    /// `is_file` rather than a flag, and the question is asked of the **file
    /// system**, exactly as `crate::app::save::has_a_file` asks it and for the
    /// reason recorded there: *"a `created_here: bool` flag is a second source
    /// of truth… and the failure mode when it drifts is writing over the wrong
    /// file."*
    ///
    /// A document created in this session has a bare name rather than a path,
    /// so there is nothing to replace and the choice is not drawn — an inert or
    /// meaningless control being worse than an absent one (the no-inert-controls
    /// rule).
    fn can_replace_original(&self) -> bool {
        self.source.is_file()
    }

    /// **Take the destination choice, and retire the acknowledgement that was
    /// given about the previous one.**
    ///
    /// ★★ Pure, and a method rather than four lines inside [`Self::gates`], so
    /// the rule can be asserted headlessly — `crate::viewer`'s standing split
    /// applied to the one flag that stands between a click and the deletion of
    /// the source document.
    ///
    /// The rule: **changing the destination un-ticks
    /// [`Self::overwrite_acknowledged`].** Without it, an operator could tick
    /// the box, think better of it, select *a new file*, change their mind
    /// again, and arrive back at *replace* with the button already live — the
    /// consent standing from a decision they had explicitly withdrawn in
    /// between. That is not a hypothetical sequence; it is what "I'll just look
    /// at what the other option says" looks like from the program's side.
    ///
    /// It fires on **any** change of destination rather than only on leaving
    /// the replace choice. Retiring a tick that was not needed costs nothing;
    /// deciding *which* changes matter is where a future edit gets it wrong.
    fn choose_destination(&mut self, choice: Destination) {
        if choice != self.destination {
            self.overwrite_acknowledged = false;
            self.destination = choice;
        }
    }

    /// Everything inside the window.
    fn body(&mut self, ui: &mut egui::Ui, ready: bool) {
        let theme = Theme::of(ui.ctx());
        match &self.phase {
            Phase::Prepared(prepared) => {
                let residuals = residual_lines(prepared);
                Self::report(ui, &theme, prepared, &residuals, self.destination);
                ui.add_space(8.0);
                ui.separator();
                self.gates(ui, &residuals, ready);
            }
            Phase::Refused(refusal) => {
                ui.label(t::report_heading());
                ui.add_space(6.0);
                ui.label(t::refusal_message(refusal));
            }
            // ★★★ The armed-removal phase. Its whole body is in `staged`, which
            // owns the sentences and the one control, so that this `match`
            // stays a list of states rather than becoming a place where one of
            // them is drawn and the others are dispatched.
            Phase::Staged => {
                self.cancel_requested |= staged::body(ui, &theme);
            }
            Phase::Written {
                path,
                replaced,
                regions,
                pages,
                residuals,
            } => {
                ui.label(outcome_line(path, *regions, *pages, *residuals, *replaced));
            }
            Phase::WriteFailed(reason) => {
                ui.label(t::write_failed(reason));
            }
        }

        ui.add_space(10.0);
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(t::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }

    /// The measured report: what will be removed, what was verified, and what
    /// could not be.
    ///
    /// Every optional line is drawn **only when its count is non-zero**. A
    /// report that listed "0 annotations removed" beside four real findings
    /// would train the operator to skim it, and the skim is what this whole
    /// surface exists to prevent.
    fn report(
        ui: &mut egui::Ui,
        theme: &Theme,
        prepared: &PreparedRedaction,
        residuals: &[String],
        destination: Destination,
    ) {
        ui.label(t::report_heading());
        ui.add_space(6.0);
        // ★ The permanence statement is FIRST in the body and in the warning
        // role — never fine print, never below the counts. It is the one
        // sentence a reader who takes in nothing else must take in.
        //
        // ★★ Three forms since 2026-09-04 (evening), one per destination, and
        // the match is exhaustive rather than an `if replacing` with an else:
        // this is the sentence that says what happens to the operator's file,
        // and a fourth destination that fell through to the wrong arm here
        // would be a false claim in the one place a false claim is worst.
        let permanence = match destination {
            Destination::OpenDocument => t::permanence_statement_deferred(),
            Destination::NewFile => t::permanence_statement(false),
            Destination::ReplaceOriginal => t::permanence_statement(true),
        };
        ui.label(egui::RichText::new(permanence).color(theme.palette.danger));
        ui.add_space(6.0);
        ui.separator();

        egui::ScrollArea::vertical()
            .id_salt(REGION_DIALOG)
            .auto_shrink([false, true])
            .max_height((ui.available_height() - FOOTER_RESERVE).max(REPORT_FLOOR))
            .show(ui, |ui| {
                let report = &prepared.report;
                ui.label(t::will_remove_heading());
                ui.add_space(4.0);
                ui.label(t::removal_summary(
                    report.marks_applied,
                    report.pages_redacted,
                    report.glyphs_removed,
                    report.content_streams_rewritten,
                ));
                if report.annotations_removed > 0 {
                    ui.add_space(4.0);
                    ui.label(t::annotations_removed(report.annotations_removed));
                }
                if report.info_strings_scrubbed > 0 {
                    ui.add_space(4.0);
                    ui.label(t::info_scrubbed(report.info_strings_scrubbed));
                }
                // ★★★ What happened to the raster images, stated even though it
                // is a success. `pdfcer-core` v0.26.0 destroys the covered
                // samples and re-encodes; before 2026-09-03 it refused the
                // document instead. A report that lists glyphs removed and says
                // nothing about an overwritten photograph has quietly picked
                // which irreversible act is worth mentioning.
                if report.images_cleared > 0 || report.images_removed > 0 {
                    ui.add_space(4.0);
                    ui.label(t::images_destroyed(
                        report.images_cleared,
                        report.images_removed,
                        report.images_overcovered,
                    ));
                }
                // ★ Separate, because it is a different claim: the same picture
                // is still on the other pages, and "I redacted the logo" and
                // "the logo is gone from this file" are not the same sentence.
                if report.images_cloned_shared > 0 {
                    ui.add_space(4.0);
                    ui.label(t::images_shared_copied(report.images_cloned_shared));
                }
                // ★★ The drawn geometry that was cut out. New in `pdfcer-core`
                // v0.27.0 and worth a line of its own on a CAD sheet: before
                // it, lines ran straight through a redacted rectangle and
                // nothing said so. This is the count that makes "the drawing
                // under the box is gone" a statement rather than an assumption.
                if report.vector_paths_cut > 0 {
                    ui.add_space(4.0);
                    ui.label(t::vector_paths_cut_line(
                        report.vector_paths_cut,
                        report.vector_paths_dropped,
                    ));
                }
                if report.containers_decomposed > 0 {
                    ui.add_space(4.0);
                    ui.label(t::containers_decomposed(
                        report.containers_decomposed,
                        report.objects_promoted,
                    ));
                }
                ui.add_space(4.0);
                ui.label(t::single_revision_note());

                // --- the proof -------------------------------------------
                //
                // "Verified" only from a clean verification that actually
                // checked something — the catalog's rule 2, enforced at the
                // one call site entitled to the word.
                let verification = &prepared.verification;
                if verification.is_clean() && verification.strings_checked > 0 {
                    ui.add_space(8.0);
                    ui.label(t::verified_line(verification.strings_checked));
                }
                if verification.strings_too_short_for_raw_check > 0 {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(t::verification_limit_line(
                            verification.strings_too_short_for_raw_check,
                        ))
                        .color(theme.palette.text_muted),
                    );
                }

                // --- what could not be removed ----------------------------
                if !residuals.is_empty() {
                    ui.add_space(10.0);
                    ui.separator();
                    ui.label(
                        egui::RichText::new(t::residual_heading()).color(theme.palette.danger),
                    );
                    ui.add_space(4.0);
                    for line in residuals {
                        ui.label(egui::RichText::new(line).color(theme.palette.danger));
                        ui.add_space(4.0);
                    }
                }

                ui.add_space(10.0);
                ui.separator();
                ui.label(egui::RichText::new(t::scope_reminder()).color(theme.palette.text_muted));
            });
    }

    /// The two checkboxes, the confirm control, and the no-shortcut note.
    fn gates(&mut self, ui: &mut egui::Ui, residuals: &[String], ready: bool) {
        // ★★★ The destination, ABOVE the acknowledgements and above the
        // confirm control, because it changes what two of them say. An
        // operator who ticked "I understand this is permanent" and then chose
        // to replace the original would have acknowledged a sentence that was
        // not yet about the thing they went on to do.
        //
        // Radio buttons rather than two confirm controls: the choice is one
        // state with two values, it is read back at the write, and a pair of
        // buttons would put two irreversible verbs side by side where a
        // mis-aimed click lands on the wrong one. Drawn only when there is an
        // original to replace — see `can_replace_original`.
        //
        // ★★★ THREE choices since 2026-09-04 (evening), and the first of them
        // is drawn UNCONDITIONALLY — which is the change that made this block
        // stop being wrapped in `can_replace_original()`. A document created in
        // this session has no file to replace, but it certainly has a session
        // to redact into, and the old shape hid the whole destination group
        // from it and silently forced a save-as. The *replace* row is what
        // depends on there being a file; the group is not.
        ui.label(t::destination_heading());
        ui.add_space(2.0);
        let mut choice = self.destination;
        let into = ui.radio_value(
            &mut choice,
            Destination::OpenDocument,
            t::destination_open_document(),
        );
        crate::diag::ui_rect(REGION_DESTINATION_INTO_DOCUMENT, into.rect);
        into.on_hover_text(t::destination_open_document_tooltip());
        let new_file = ui.radio_value(&mut choice, Destination::NewFile, t::destination_new_file());
        crate::diag::ui_rect(REGION_DESTINATION_NEW_FILE, new_file.rect);
        new_file.on_hover_text(t::destination_new_file_tooltip());
        if self.can_replace_original() {
            let name = file_name_of(&self.source);
            let replace = ui.radio_value(
                &mut choice,
                Destination::ReplaceOriginal,
                t::destination_replace(&name),
            );
            crate::diag::ui_rect(REGION_DESTINATION_REPLACE, replace.rect);
            replace.on_hover_text(t::destination_replace_tooltip());
        }
        self.choose_destination(choice);
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        // ★★★ **The staging disclosure, ABOVE the confirm control.**
        //
        // ★★★ CORRECTED 2026-09-05. This block used to draw
        // `undo_will_be_cleared(depth)` — the price of `Pass 250.1`'s
        // collapsing verb — and the comment that stood here argued at length
        // that the operator was owed the step count before he committed. He was.
        // There is no step count any more: `Pass 250.2` preserves the whole
        // undo log, so the sentence would be false and the argument for showing
        // it has become an argument for showing something else.
        //
        // What is drawn instead is the fact that replaces it, and it is more
        // surprising rather than less: **the page does not change**. He presses
        // a control about permanent removal and the marks and the content stay
        // exactly where they are, because the removal happens at the save. The
        // two readings available to him without this sentence are *"it did not
        // work"* and *"it worked and the marks are just still drawn"*, and the
        // second is the one that ships a marked file.
        //
        // ★ A sentence and not a fourth checkbox, deliberately, and that part
        // of the old argument is untouched. This dialog's §3 argues that a box
        // which is always there is a box that is always ticked, and the same
        // erosion applies to boxes that multiply: four acknowledgements is a
        // form, and a form is filled in rather than read. The operator is
        // already gated on the permanence box; what he is owed here is the
        // FACT, before the click, which is what rule 4 asks for and what
        // "told, not asked" means.
        //
        // Drawn only for the destination it is true of. The two write-now
        // destinations really do produce a file at the click.
        if let Some(sentence) = self.staging_disclosure() {
            // `danger`, not `notice`, and the palette's own split decides it:
            // notice is *"worth knowing and nothing is broken"*, and a
            // permanent removal an operator has just armed without seeing
            // anything happen is the sharpest end of this dialog.
            let danger = Theme::of(ui.ctx()).palette.danger;
            let note = ui.label(egui::RichText::new(sentence).color(danger));
            crate::diag::ui_rect(REGION_STAGING_NOTE, note.rect);
            ui.add_space(6.0);
        }
        // Shown only when the operator has actually asked to replace the
        // original, for the same reason the residual box is conditional: a
        // permanent checkbox is a permanent reflex.
        if self.destination == Destination::ReplaceOriginal {
            let name = file_name_of(&self.source);
            let box_ = ui.checkbox(
                &mut self.overwrite_acknowledged,
                t::overwrite_acknowledgement_checkbox(&name),
            );
            crate::diag::ui_rect(REGION_OVERWRITE_ACK, box_.rect);
            ui.add_space(4.0);
        }
        // Shown only when there is something to acknowledge — §3 item 2.
        if !residuals.is_empty() {
            let box_ = ui.checkbox(
                &mut self.residuals_acknowledged,
                t::residual_acknowledgement_checkbox(),
            );
            crate::diag::ui_rect(REGION_RESIDUAL_ACK, box_.rect);
            ui.add_space(4.0);
        }
        let ack = ui.checkbox(&mut self.acknowledged, t::confirm_checkbox());
        crate::diag::ui_rect(REGION_ACK, ack.rect);
        ui.add_space(8.0);

        // ★ The label IS the consequence, and the consequence now depends on
        // the destination: an ellipsis promises the picker, and naming the file
        // promises there will be no further question before it is replaced.
        let label = match self.destination {
            // No ellipsis and no file name: nothing is written, so there is no
            // further question and no file to name.
            Destination::OpenDocument => t::confirm_button_into_document().to_owned(),
            Destination::NewFile => t::confirm_button().to_owned(),
            Destination::ReplaceOriginal => t::confirm_button_replace(&file_name_of(&self.source)),
        };
        let confirm = ui.add_enabled(ready, egui::Button::new(label));
        // Declared only while it is live, so its absence from a trace is
        // evidence the gates are closed rather than evidence a click missed.
        if ready {
            crate::diag::ui_rect(REGION_CONFIRM, confirm.rect);
        }
        let clicked = confirm.clicked();
        // ★★★ **A greyed Confirm with no explanation at all** — O77's sweep,
        // and the most consequential of the seven: this is the last control
        // before content is destroyed, and an operator who cannot press it had
        // no way to find out why.
        //
        // ★ It names WHICH box is unticked rather than refusing generically.
        // Two checkboxes gate this button and they appear at different times —
        // the residual one only when the engine reported residuals — so
        // *"tick the box"* would be ambiguous exactly when it matters.
        //
        // ★★ The `if !ready` shape, and the borrow order, are copied from
        // `dialogs::formfield` and `dialogs::textannot`:
        // `on_disabled_hover_text` CONSUMES the response, so `.rect` and
        // `.clicked()` are read first.
        if !ready {
            // ★ Three OUTSTANDING flags, not three "acknowledged" ones. A box
            // that was never drawn is not owed, and sending the operator to
            // look for it would be the vague refusal this sentence exists to
            // prevent — so the conditions that decide whether each box appears
            // are the same expressions used here.
            confirm.on_disabled_hover_text(t::confirm_disabled(
                !self.acknowledged,
                !residuals.is_empty() && !self.residuals_acknowledged,
                self.destination == Destination::ReplaceOriginal && !self.overwrite_acknowledged,
            ));
        }
        if clicked {
            self.confirm_requested = true;
        }
        ui.add_space(6.0);
        ui.label(egui::RichText::new(t::no_shortcut_note()).small().weak());
    }

    /// **Send the redacted bytes to the destination the operator chose.**
    ///
    /// ★★★ CORRECTED 2026-09-04. This method's doc comment used to read:
    ///
    /// > *"It asks, every time, and the suggestion is never the file that was
    /// > opened — see [`suggested_path`]. There is no 'save over the original'
    /// > branch to find, because there is none to write, and on this operation
    /// > that is the difference between a copy and the destruction of the only
    /// > remaining source of the content being removed."*
    ///
    /// The operator overruled it, and the reasoning is in [`Destination`]. What
    /// survives of the old ruling is the part that was a *mechanism* rather than
    /// a *prohibition*: [`suggested_path`] still never proposes the source file.
    /// What is gone is the refusal to write the branch at all.
    ///
    /// ⚠ **Corrected 2026-09-05.** This paragraph said *"[`Destination::NewFile`]
    /// is still the default"* — and by then [`DEFAULT_DESTINATION`] two hundred
    /// lines above it read [`Destination::OpenDocument`], moved on 2026-09-04.
    /// **One file asserting two different defaults about itself**, which is
    /// worse than a stale sentence in a document nobody reads: this is the
    /// paragraph a future session consults *before* changing the default.
    ///
    /// ★ The claim it was making is still true of the mechanism, and that is
    /// why it survived a rewrite of the surrounding argument: both defaults are
    /// safe, for **different reasons** — `NewFile` never *overwrote*,
    /// `OpenDocument` never *writes*. A sentence that is right about the
    /// principle and wrong about the value is the hardest kind to notice.
    ///
    /// So there are now two paths, and the asymmetry between them is the whole
    /// safety argument:
    ///
    /// | destination | how the path is obtained | what stands between the click and the write |
    /// |---|---|---|
    /// | [`Destination::NewFile`] | the save picker, suggesting `-redacted` | the picker itself, plus the OS's own overwrite prompt if the operator navigates onto an existing file |
    /// | [`Destination::ReplaceOriginal`] | [`Self::source`], with no picker | a **third** checkbox naming the file, and a confirm button whose label names it too |
    ///
    /// ★ Replacing takes no picker **deliberately**. A picker pre-filled with
    /// the source would be a dialog whose safe answer is to change the field,
    /// which is the shape of every accidental overwrite there has ever been.
    /// The consent is taken before the click, in words, at a control the
    /// operator had to select; once taken, the program does what it said.
    ///
    /// ★★ The write itself is atomic — temp file, then rename — because on this
    /// path a torn write destroys the last remaining copy of the content being
    /// removed. See [`crate::redact::PreparedRedaction::write_to`].
    fn commit(&mut self, actions: &mut Vec<crate::app::actions::Action>) {
        let Phase::Prepared(prepared) = &self.phase else {
            return;
        };
        // ★★★ THE DEFERRED ROUTE, 2026-09-04 (evening). Nothing is written and
        // nothing here touches a file system.
        //
        // It leaves through the ACTION FUNNEL rather than being performed here,
        // and that reverses §5 of this file's header for one destination — the
        // section is corrected in place. The reason §5 gave for staying out of
        // the funnel was that applying *"changes no document, so it has nothing
        // to order against and no epoch to bump"*. On this destination it
        // changes the open document, so it has both: `vector_edit` cancels the
        // render worker, takes the session, bumps `edit_epoch`, invalidates the
        // page textures and resyncs the page set. Performing that from inside a
        // dialog's draw is exactly what the funnel exists to prevent.
        //
        // ★ The dialog CLOSES rather than moving to an outcome phase, and the
        // outcome is reported by the funnel's edit disclosure like any other
        // edit. Two accounts of one event is worse than one: the action runs
        // after this frame, so a sentence written here would be a prediction
        // — and on the one path where the action failed, a false one.
        if !self.destination.writes_now() {
            actions.push(crate::app::actions::Action::Redact(
                crate::app::actions::RedactAction::Pending(crate::redact::Staging::Stage),
            ));
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // `marks=` is the number the operator was shown at the moment
                // of consent. The funnel's own `redact-staged` line records
                // what the engine's staging preview then measured; a build in
                // which the report on screen and the removal that gets armed
                // had drifted apart would show the two disagreeing.
                format!(
                    "redact-apply-deferred marks={}",
                    prepared.report.marks_applied
                )
            });
            self.close_requested = true;
            return;
        }
        let acknowledgement = if self.residuals_acknowledged {
            ResidualAcknowledgement::Given
        } else {
            ResidualAcknowledgement::Withheld
        };
        let residuals = residual_lines(prepared).len();
        let regions = prepared.report.marks_applied;
        let pages = prepared.report.pages_redacted;
        let target = match self.destination {
            // Answered above and returned; spelled rather than left to a `_`
            // arm so that a future fourth destination is a compile error here
            // rather than a file written to the wrong place.
            Destination::OpenDocument => return,
            // No picker: the consent for this path was taken in words, at the
            // radio and the third checkbox, before the click. See the table
            // above for why a pre-filled picker would be worse rather than
            // safer.
            Destination::ReplaceOriginal => self.source.clone(),
            Destination::NewFile => {
                let suggested = suggested_path(&self.source);
                let crate::app::files::Picked::Path(chosen) =
                    crate::app::files::pick_save_path(&suggested, t::save_dialog_title())
                else {
                    // Cancelled, or a build with no picker. The prepared bytes
                    // are still in hand and the control is still there: nothing
                    // is lost and nothing is said, because a cancelled save is a
                    // complete and uninteresting outcome. The marks are
                    // untouched either way.
                    return;
                };
                chosen
            }
        };
        self.phase = match prepared.write_to(&target, acknowledgement) {
            Ok(_) => Phase::Written {
                path: target,
                replaced: self.destination == Destination::ReplaceOriginal,
                regions,
                pages,
                residuals,
            },
            Err(refusal) => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("redact-write-failed path={target:?} detail={refusal}")
                });
                Phase::WriteFailed(refusal)
            }
        };
    }
}

/// **The sentence shown once bytes are on disk.**
///
/// Free rather than a method so the catalog's rule 1 — *a residual is named in
/// the same sentence as the success* — is decided by a pure function a test can
/// drive, rather than inside a `match` on a window's state.
///
/// The branch is on `residuals`, and the two sentences are genuinely different
/// copy rather than one with a number in it. An operator who acknowledged a
/// residual in this dialog and then closed it is owed a standing record of what
/// remains, and *"…and verified absent from the saved file"* would be a lie in
/// that case rather than merely an omission.
///
/// The **file name** rather than the whole path, because the sentence is read
/// in a window that is about 700 pt wide and a Windows path is routinely longer
/// than that. The full destination is on the trace line
/// `PreparedRedaction::write_to` emits, which is where a reader who needs it
/// will look.
/// **The file name a sentence should use for `path`.**
///
/// The name rather than the whole path, because every sentence that needs one
/// is read in a window about 700 pt wide and a Windows path is routinely longer
/// than that. Falls back to the whole path when there is no final component,
/// which is the only case in which the longer string is the more informative
/// one.
///
/// Shared by [`outcome_line`] and by the destination controls so the file is
/// spelled the same way in the choice, in the acknowledgement, on the button
/// and in the outcome. Four different spellings of one file name on one screen
/// is how an operator ends up unsure which file the sentence is about.
#[must_use]
fn file_name_of(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}

#[must_use]
fn outcome_line(
    path: &Path,
    regions: u64,
    pages: usize,
    residuals: usize,
    replaced: bool,
) -> String {
    let name = file_name_of(path);
    if residuals == 0 {
        t::applied_clean(&name, regions, pages, replaced)
    } else {
        t::applied_with_residuals(&name, regions, residuals, replaced)
    }
}

/// **Every item the report discloses as NOT removed.**
///
/// The single expression that both gates the extra acknowledgement and prints
/// the section — one derivation, so a residual can never be listed without
/// being acknowledgeable or acknowledged without being listed. Three sources,
/// in this order:
///
/// 1. **carriers the engine could not scrub** — `CarrierAction::
///    DisclosedNotScrubbed`, the cardinal-rule-honest outcome for a carrier
///    this build cannot fully redact;
/// 2. **raw-byte residuals** — [`crate::redact::proof`]'s middle verdict, a
///    byte run that survives outside every decoded stream and that pdfcer
///    genuinely cannot classify;
/// 3. **retained marks** — regions where nothing was removed because the image
///    under them could not be decoded. The engine names this as the number to
///    read before saying "redacted", and it is the strongest kind of residual
///    on this list: the content is still there, under a rectangle that says it
///    is not;
/// 4. **vector geometry that could not be cut**, and **clips whose outline had
///    to be kept** — an outline on a drawing can be as identifying as the text
///    it surrounded;
/// 5. **objects promoted out of a compressed container** by materialising the
///    operator's unsaved edits (engine rule R38).
///
/// The last is the mildest and is listed anyway. Page content cannot live in
/// an object stream at all (ISO 32000-1 §7.5.7), so it cannot hold redacted
/// text — but it is a leftover of the operator's own edits, and a report that
/// silently drops the findings it judges harmless is a report whose judgement
/// the operator has no way to audit.
#[must_use]
fn residual_lines(prepared: &PreparedRedaction) -> Vec<String> {
    use pdfcer_core::redact::CarrierAction;
    let mut out: Vec<String> = prepared
        .report
        .carriers
        .iter()
        .filter(|c| c.action == CarrierAction::DisclosedNotScrubbed)
        .map(|c| t::residual_carrier_line(c.carrier))
        .collect();
    // ★★★ RETAINED MARKS, and the engine names this as the one number to read
    // before the word "redacted" is used. A retained mark is a region where
    // NOTHING was removed — the image under it could not be decoded, so the
    // engine applied every other mark and left that one standing rather than
    // refusing the document. The result is a half-redacted file that looks
    // finished, which is precisely what this list exists to prevent.
    if prepared.report.marks_retained > 0 {
        out.push(t::marks_retained_line(prepared.report.marks_retained));
    }
    // ★★ Vector geometry crossing a region that could NOT be cut — a malformed
    // path object the engine cannot rewrite as a unit. Zero on every
    // well-formed page since `pdfcer-core` v0.27.0, which cuts paths at the
    // region boundary; a non-zero value here is therefore rare and is a real
    // residual, not the ordinary case.
    //
    // On a drawing this is the residual that matters most and the one nobody
    // asks about: a title-block border or a view's geometry running through a
    // redacted rectangle is a shape, and a shape can be as identifying as the
    // text it surrounded.
    if prepared.report.vector_paths_intersecting > 0 {
        out.push(t::vector_paths_residual_line(
            prepared.report.vector_paths_intersecting,
        ));
    }
    // ★ A clip whose ink was cut and whose ORIGINAL outline had to stay: ISO
    // 32000-1 §8.5.4 applies a clip after painting, so shrinking it would hide
    // later, unmarked content. Nothing of it is visible and it is still a shape
    // in the file — exactly the finding rule 1 forbids judging harmless on the
    // operator's behalf.
    if prepared.report.vector_clips_kept > 0 {
        out.push(t::vector_clips_kept_line(prepared.report.vector_clips_kept));
    }
    out.extend(
        prepared
            .verification
            .residuals
            .iter()
            .map(|r| t::raw_residual_line(&r.text, r.site)),
    );
    if !prepared.promoted_by_materialisation.is_empty() {
        out.push(t::promotion_line(
            prepared.promoted_by_materialisation.len(),
        ));
    }
    out
}

/// **The name to suggest for the redacted copy.**
///
/// ★ **Never the file that was opened.** The suffix is what makes the default
/// answer a new document, so an operator who accepts the suggestion without
/// reading it cannot overwrite the one file that still contains the content
/// they are removing. That is the standing rule expressed as a default rather
/// than as a warning — a warning is something to click past.
///
/// The same shape and the same argument as `crate::app::save::suggested_path`
/// and `crate::dialogs::ocr::suggested_path`, with a different suffix, and the
/// extension is forced to `.pdf` for their reason: the bytes are a PDF whatever
/// the source was called.
#[must_use]
pub fn suggested_path(source: &Path) -> PathBuf {
    let stem = source.file_stem().map_or_else(
        // ui-text-exempt: a filename fallback for a path with no stem, not
        // operator copy. Both sibling suggestion functions make the same one.
        || String::from("document"),
        |s| s.to_string_lossy().into_owned(),
    );
    let name = format!("{stem}{}.pdf", t::suggested_suffix());
    source
        .parent()
        .map_or_else(|| PathBuf::from(&name), |dir| dir.join(&name))
}

/// Open the dialog for the document in `status`, if there is one.
///
/// The dispatch target for `edit.redact_apply`. Lives here rather than in
/// [`super::DialogsState`] only because it needs [`RedactDialog::open`]'s
/// private constructor; the guard it applies is the one `open_print` documents
/// — the ribbon control is gated on `doc.pages`, a chord bound to the same id is
/// not, and both are fixed by refusing here at the one place the dialog is
/// built.
pub(super) fn open_for(status: &Status) -> Option<RedactDialog> {
    let Status::Open(doc) = status else {
        return None;
    };
    Some(RedactDialog::open(doc))
}

/// The headless assertions for this dialog's state machine, in their own file
/// since 2026-09-04 — see [`tests`]'s header for the seam.
#[cfg(test)]
mod tests;
