//! # `dialogs` — the shell's stationary, screen-anchored surfaces
//!
//! ## What belongs here, and what does not
//!
//! A **dialog** is a single transaction with a start and an end: it is opened
//! deliberately, it holds one job's worth of answers, and closing it forgets
//! them. A **panel** is somewhere an operator dips in and out of while
//! working, and it keeps its state across documents. The distinction decides
//! where a surface lives, and getting it wrong is not cosmetic — a print
//! configuration that persisted across documents would let a range typed for
//! one file silently apply to another.
//!
//! `SALVAGE.md`'s redistribution table names the tenants of this directory:
//! *"Dialogs — properties, print, export, reset, settings host — ~1,500 lines
//! — `dialogs/`."* [`print`] is the first of them.
//!
//! ## ★ Every dialog here is screen-anchored, never page-anchored
//!
//! A decision inherited from the old shell, where it was made in response to a
//! specific operator objection: **controls whose position is derived from the
//! page move on every zoom and scroll.** A surface an operator is reading and
//! typing into must stay where they put their eyes. Each dialog therefore
//! anchors to the viewport rather than being positioned relative to the
//! canvas, and none of them is drawn inside the canvas's coordinate space.
//!
//! ## ★ Where dialog state lives, and why it is one field
//!
//! [`DialogsState`] is the whole dock-side surface of this module: one field
//! on `PdfcerApp`, one `open_*` call per dialog from the command dispatcher,
//! and one [`DialogsState::show`] call per frame. It follows
//! `crate::panels::PanelsState` exactly — same idiom, second instance, not a
//! new convention — and the reason it is a struct rather than a bare
//! `Option<PrintDialog>` is that the *next* dialog is then a change to this
//! file rather than to `app/mod.rs`, which is the file every parallel task
//! already contends over.
//!
//! ## Why a dialog does not push an `Action`
//!
//! `crate::app::actions`' invariant is that **no code path runs from a widget
//! to a document**, and the four things it buys are all about *document*
//! state: a coherent undo log, an aliasing problem turned into a queue,
//! explicit ordering between changes, and a greppable answer to "what can
//! change this?".
//!
//! A print changes no document state. It reads the document — the pages, the
//! edited view — and writes to a spooler, so it contributes nothing to the
//! undo log and has nothing to order against. Routing it through the funnel
//! would add an `Action` variant that `apply` could only answer by reaching
//! back into a dialog for the state it needs, which is the funnel pointing the
//! wrong way.
//!
//! What the funnel's *reason* does still demand is that the irreversible work
//! not happen part-way through a layout pass, and [`print::PrintDialog`]
//! honours that in its own scope: the button sets a flag, and the spool runs
//! after the window's closure returns. See that field's documentation.
//!
//! **A dialog that edits the document is a different case and must use the
//! funnel.** The properties dialog and the settings host will both raise
//! `Action`s; this note is about printing specifically, not about dialogs in
//! general.

pub mod about;
/// The *What pressing it does* chooser, drawn into the form-field placement
/// dialog for a push button. Its own module because it is the only row group
/// there that carries a disclosure obligation.
pub mod buttonaction;
/// The Embed-fonts confirmation - everything `embed_fonts` would do to the
/// document, computed by the verb's own planner and shown before any of it
/// happens.
///
/// Its header carries the reason it has no settings: the only configuration an
/// embed has is *which folders*, and that lives in Settings.
/// The window before a full rewrite. Its header carries the reason it writes
/// the file BEFORE it opens: when a window asks somebody to trade something
/// irreversible for a benefit, the benefit must be measured, not predicted.
pub mod compact;
/// The render report `tools.render_diagnostics` opens — what the renderer did
/// with the page currently on the canvas, with the room the status bar's one
/// elided line does not have.
pub mod diagnostics;
pub mod embed;
/// ★ The Insert-image window — a picture placed on the page as content, by a
/// rectangle in millimetres.
///
/// Its header carries the decision a reader will question first: why placement
/// is numeric rather than a drag, and why a drag is a second **route** to the
/// same action rather than the one that should have shipped.
/// ★ The Export-DXF window — the page's vector geometry, at a scale somebody
/// can defend.
///
/// Its header carries the sentence the whole feature turns on, quoted from
/// `pdfcer-core`: every generic PDF-to-DXF converter exports at paper scale and
/// says nothing, so a 1:2 detail arrives at half size **looking plausible**.
pub mod export_dxf;
/// ★★★ The Export-image window — a picture of the page in a format that can
/// actually hold what is on it. `OPERATOR_REQUESTS.md` O120.
pub mod export_image;
/// ★★★ The Export-text window. Its header carries the half of the operator's
/// ask that does not exist: **no route from a text file back into a PDF**.
pub mod export_text;
pub mod formfield;
/// ★★ **A dialog is an OS window** — the operator's report of 2026-08-20, and
/// `ui-conventions/dialogs.md` G1. One host, so the path of least resistance
/// and the right answer are the same call; its header carries what an OS
/// window actually buys, how it degrades on the web target, and the two rows
/// (G3 ownership, G5 focus trapping) that eframe 0.35 cannot express.
pub mod host;
pub mod insert_image;
pub mod insert_pages;
pub mod new_document;
pub mod ocr;
/// ★ How a window offers to step aside so the operator can point at the page —
/// `OPERATOR_REQUESTS.md` O66. The dialog half of `canvas::placing`.
/// The box that lets an encrypted document be opened. Its header records the
/// defect it closes: the shell detected `NeedsPassword` perfectly and had no way
/// to supply one, for as long as it has been able to detect it.
/// **How each dialog is built** — every `open_*` constructor, split out of this
/// file on 2026-09-05 when it reached R2's 1,500-line ceiling. The seam is
/// argued in that module's own header: an opener decides *whether* a dialog may
/// exist and gathers what it needs, while this file owns *what dialogs exist*
/// and *how their answers reach the app*. Two different callers, two different
/// sets of invariants, one receiver.
pub mod open;
/// ★★★ **The question that comes before pdfcer lets go of the file** —
/// `OPERATOR_REQUESTS.md` O122. Three shapes of one window: save-then-hand-over,
/// confirm-and-hand-over, and the refusal for a document that has never been
/// written anywhere.
///
/// Its header carries why there is no third *"open without saving"* button,
/// which is the one place it deliberately departs from [`unsaved`]'s shape.
pub mod open_in_acrobat;

pub mod password;
pub mod placing;
pub mod print;
/// **Encrypt… and Permissions…** — `OPERATOR_REQUESTS.md` O119. Its header
/// carries the three disclosures O119 named and where each is drawn, why the
/// form is two sections in a fixed order, and why the saving mechanism is
/// [`redact`]'s part for part rather than a second answer to a settled question.
pub mod protect;
/// The Apply-redactions transaction — the report, the two acknowledgements, and
/// the write. The **irreversible** half of the redaction feature; its
/// reversible twin is `crate::panels::redact`. See its header for why the
/// removal runs on open, why confirmation is three gates rather than one click,
/// and why the destination is asked for every time.
pub mod redact;
/// ★★ The question Save has never asked about a **signed** document — the
/// warning that stands between a structural edit and a revision written over a
/// legal artifact.
///
/// Its header carries the gap it closes, why the engine says the question can
/// only be asked at save time, the table of which impact earns which surface,
/// and why the compacted-save path needed nothing from it.
pub mod signature;
/// The Remove-fonts confirmation - the destructive twin of `embed`, and the
/// disclosure surface its own scaffold entry named as the thing blocking it.
pub mod unembed;
/// ★★ The question `file.close` promised in its own tooltip and never asked —
/// the confirmation that stands between an operator's afternoon of markup and
/// `Status::Empty`.
///
/// Its header carries the defect it closes, why `save_pending` was NOT the bug
/// and must not become the fix, and why the first button says *Save a copy…*
/// rather than *Save*.
pub mod unsaved;
/// ★ Raising and draining the unsaved question — split out of this file on
/// 2026-09-02 under R2. Its header carries the three-answer shape that makes it
/// a real seam, and the defect the middle answer cost.
mod unsaved_host;

/// ★ The Set-scale dialog — what a dimension's number *means*.
///
/// Phase 7 shipped three tools that place dimensions and no way to say what
/// scale they are at, so every label read in PDF points: a measurement of the
/// **paper** rather than of the thing drawn on it. A plausible answer to a
/// question nobody asked, which is worse than a missing feature.
pub mod scale;
/// The words half of a text-bearing annotation — the second half of the
/// place-then-type gesture. Its header argues why it is a dialog.
pub mod textannot;

/// ★ The Settings window — the thirteen questions the PDF standard declines to
/// answer, and the operator's answers to them.
///
/// **Application-scoped and not held in [`DialogsState`]**, which is the one
/// departure in this directory and is forced rather than chosen. Its draft has
/// to be readable at the *top* of the frame, before any widget is built,
/// because the theme is installed there and a draft theme must take effect
/// immediately — you cannot judge a theme from a radio label. So the draft
/// lives on `PdfcerApp` as `settings_draft`, and this module is a renderer with
/// no state of its own.
/// ★ The keyboard reference, **derived from the keymap that dispatches**.
///
/// Application-scoped, beside [`about`]: a keyboard reference is meaningful
/// with nothing open, and is one of the two things a new operator reaches for
/// before opening a file.
///
/// Its header carries why it holds no list — `DEFECTS.md` D5 is not fixed
/// there, it is made unrepresentable.
pub mod shortcuts;

pub mod settings;

use crate::app::state::{OpenDoc, Status};

/// **Whether a dialog that has just drawn must be dropped out of its slot.**
///
/// `open` is what the dialog's own `show` returned — *"should I still be on
/// screen?"* — and `answered` is whether it is holding a decision its owner has
/// not collected yet. A dialog is retired only when **both** say no: it is off
/// screen *and* it has nothing left to hand over.
///
/// # ★★★ WHY THIS IS NOT `!open`, AND THE DAY THAT COST
///
/// It was `!open`, expressed at each call site as
/// `if …map(|d| d.show(ctx)) == Some(false) { self.slot = None; }`, and for
/// eleven of the thirteen dialogs that is exactly right: they act through
/// `actions` while they draw, so a closed one has nothing left in it.
///
/// The two **confirmation** windows are different in kind, and the difference
/// is the whole defect. `unsaved` and `signature` deliberately do NOT act. They
/// *park* an answer and let `crate::app::PdfcerApp` perform it —
/// `resume_after_unsaved` and `resume_after_signature`, both later in the same
/// frame — because the acts in question (closing a document, writing over the
/// operator's own file) are the two most destructive things this shell does and
/// must have exactly one route each. A window that could call `save_in_place`
/// would be a second route.
///
/// So for those two, `show` returning `false` and the dialog being *finished*
/// are different facts. Pressing the button sets the answer, which is what
/// makes `show` answer `false` — and the old branch then destroyed the dialog,
/// **and the answer inside it**, before the drain three call frames later could
/// look. `take_signature_answer` found an empty slot and returned `None`.
///
/// ⇒ The observable result, found by driving on 2026-08-29
/// (`an_invalidating_save_is_warned_about`): the signature warning opened, held
/// the save, drew its proceed button, took the click, **closed** — and traced
/// no `signature-confirmed` and wrote no file. A signed document could not be
/// saved at all by any route the guard covers. The feature had shipped the day
/// before, correctly stopping the save and never letting it through, which is
/// worse than the silence it replaced.
///
/// ★ Neither half was wrong on its own, which is why no unit test saw it. The
/// dialog returns its answer when asked; the drain performs whatever it is
/// given; the defect lives entirely in the **lifetime between them**, and a
/// lifetime is not a value any assertion over either half can name. That is the
/// same shape `PROJECT_PLAN.md` §4 built the driving harness for.
///
/// # The invariant this creates, stated where it can be checked
///
/// > **Every caller of [`DialogsState::show`] must drain the parked answers in
/// > the same frame.**
///
/// There is one caller — `crate::app::frame` — and it drains both, immediately
/// after. A retained-because-answered dialog therefore lives for zero frames:
/// it is emptied and dropped by `take_*_answer` before anything can draw it
/// again. A caller that did not drain would see the window redraw for as long
/// as it ignored it, which is a loud failure rather than a silent one, and that
/// direction was chosen deliberately over discarding the answer.
const fn retire(open: bool, answered: bool) -> bool {
    !open && !answered
}

/// Every dialog this build has, and whether each is open.
///
/// One field per dialog, each an `Option` whose `Some` *is* the "open" state —
/// there is no separate visibility flag that could disagree with whether the
/// state exists. Closing a dialog drops its state, which is what makes
/// "closing forgets the job" true by construction rather than by remembering
/// to reset fields.
///
/// ## ★ The fields are in two groups, and the split is load-bearing
///
/// A **document-scoped** dialog is about the open file: a print job is a job
/// on *these* pages. An **application-scoped** dialog is about pdfcer itself
/// and is meaningful with nothing loaded.
///
/// Until 2026-08-14 every dialog here was document-scoped and
/// [`DialogsState::show`] could take the shortcut of dropping all of them the
/// moment the document went away. [`about::AboutDialog`] broke that: an
/// operator who has just launched pdfcer and wants to know what version they
/// are running, or under what terms, has no document — and a control that did
/// nothing in that state would be the placeholder `HANDOFF.md` §6 forbids.
///
/// So the two groups are drawn separately rather than the rule being softened
/// for everything. Print still closes with its document; About does not, and
/// cannot be made to without breaking the command that opens it.
#[derive(Default)]
pub struct DialogsState {
    // --- document-scoped: closed when the document closes -----------------
    /// The print dialog, when one is open.
    print: Option<print::PrintDialog>,

    /// The Recognise-text dialog, when one is open.
    ///
    /// Document-scoped, and firmly so: a recognition is of one page of one
    /// file. ★ It is the first dialog here that can hold **unsaved bytes**, and
    /// closing the document discards them — which is the right answer rather
    /// than a loss. Writing them afterwards would produce a file derived from a
    /// document the operator has already put away, and offering to do that is
    /// how a program ends up with two ideas about what "the document" means.
    ocr: Option<ocr::OcrDialog>,

    /// The Render-diagnostics report, when one is open.
    ///
    /// Document-scoped: it describes *this page of this file*, and a window
    /// left up over a closed document would be reporting measurements of a
    /// raster that no longer exists. It holds no configuration, so closing it
    /// forgets nothing — but it must still close, for the same reason print
    /// does.
    diagnostics: Option<diagnostics::DiagnosticsDialog>,

    /// The Set-scale dialog, when one is open.
    ///
    /// Document-scoped, and the first dialog here that **edits the document
    /// through the action funnel**. Print writes to a spooler and OCR produces
    /// a new file; this one recalibrates a dimension group in the open
    /// document, which is an undoable edit — see
    /// `crate::app::actions::Action::SetGroupScale`.
    ///
    /// ★ *"and redaction produce new files"* stood here until 2026-09-04:
    /// `dialogs::redact` now edits through the funnel too, on its default
    /// destination of three.
    ///
    /// That is why [`Self::show`] takes an action queue at all: this module's
    /// header says a dialog that edits the document *"must use the funnel"*,
    /// and this is the first one that does.
    scale: Option<scale::ScaleDialog>,
    /// The open text-annotation dialog, if a text box, sticky or stamp has
    /// just been placed.
    text_annot: Option<textannot::TextAnnotDialog>,
    /// The form-field placement dialog, if one is open.
    ///
    /// `None` is both "closed" and "nothing placed"; `Some` is both "open" and
    /// "here is the draft", which is the one-field idiom every dialog in this
    /// struct follows.
    form_field: Option<formfield::FormFieldDialog>,

    /// The Apply-redactions dialog, when one is open.
    ///
    /// Document-scoped, and more emphatically than any of its neighbours. ★ It
    /// is the second dialog here that holds **unsaved bytes** and the first
    /// whose bytes are a *destructive* transformation of the open file, so
    /// closing the document discards them — which is the right answer rather
    /// than a loss, and for a sharper version of [`Self::ocr`]'s reason: a
    /// redaction is of *these marks* on *this document*, and writing prepared
    /// bytes after the operator has put the file away would produce a redacted
    /// copy of something nobody is looking at, derived from a mark census that
    /// no longer exists to be checked against.
    redact: Option<redact::RedactDialog>,

    /// The Encrypt / Permissions window, when one is open — O119.
    ///
    /// Document-scoped for [`Self::redact`]'s reason at its sharpest: it holds
    /// the document's protection **as it stood when the window opened**, and
    /// every control on it is seeded from that reading. ★ ONE field for the two
    /// ribbon controls — see [`Self::open_protect`].
    protect: Option<protect::ProtectDialog>,

    // --- application-scoped: survives an empty canvas ---------------------
    /// The About dialog, when one is open.
    ///
    /// Carries the attribution surface — see [`about`] and
    /// [`crate::text::about`] for why a shipped `LICENSE` file is not enough
    /// once a CC-BY-SA-4.0 asset is in the package.
    about: Option<about::AboutDialog>,

    /// The sized-New dialog, when one is open.
    ///
    /// **Application-scoped**, beside About and for the strongest version of
    /// its reason: an operator with nothing open is not somebody this window is
    /// *tolerated* for, they are the operator it exists for. Closing a document
    /// must therefore not close it — and, unlike About, this one would be
    /// actively harmful to close, because the document it is about to make is
    /// how the operator gets out of the empty state.
    new_document: Option<new_document::NewDocumentDialog>,

    /// The keyboard reference, when one is open.
    ///
    /// **Application-scoped**, beside [`Self::about`] and for its reason: a
    /// keyboard reference is meaningful with nothing open, and closing a
    /// document must not close it.
    shortcuts: Option<shortcuts::ShortcutsDialog>,

    /// The password prompt, when a document is waiting on one.
    ///
    /// ★★ **Application-scoped, not document-scoped**, and the distinction is
    /// the whole of why it works: the document it is about is **not open** —
    /// that is its premise — so a guard that closed it when nothing was open
    /// would close it exactly when it is needed. It sits beside About and the
    /// sized-New window for the same reason those two do.
    password: Option<password::PasswordDialog>,

    /// The insert dialog, when one is open.
    ///
    /// **Document-scoped**: it inserts into the open document, so closing that
    /// document closes it. It sits in this group rather than beside About for
    /// the reason the group exists — a dialog configuring an edit to a file
    /// that is no longer open is configuring nothing.
    insert_pages: Option<insert_pages::InsertPagesDialog>,

    /// The Insert-image window, when one is open.
    ///
    /// **Document-scoped**: it places a picture on a page of the open file, and
    /// it holds the imported bytes — so closing the document discards them,
    /// which is the right answer rather than a loss, for [`Self::ocr`]'s reason
    /// applied to an operand instead of to a result.
    insert_image: Option<insert_image::InsertImageDialog>,

    /// The Embed-fonts window, when one is open.
    ///
    /// **Document-scoped**: its whole content is a plan computed against the
    /// open document's font inventory, so it describes nothing once that
    /// document is gone.
    embed: Option<embed::EmbedDialog>,

    /// The compacted-copy confirmation, when one is open.
    ///
    /// **Document-scoped**: it holds a serialisation of the open document, which
    /// describes nothing once that document is gone — and holds it by value, so
    /// closing the document frees it.
    compact: Option<compact::CompactDialog>,

    /// The Remove-fonts window, when one is open.
    ///
    /// **Document-scoped**, for its sibling's reason: its content is a plan
    /// computed against the open document's font inventory.
    unembed: Option<unembed::UnembedDialog>,

    /// The Export-DXF window, when one is open.
    ///
    /// **Document-scoped**: it exports a page of the open file, and its scale
    /// suggestion is computed from that document's own dimension groups.
    export_dxf: Option<export_dxf::ExportDxfDialog>,

    /// The Export-image window, when one is open.
    ///
    /// **Document-scoped**, for its neighbour's reason: every control in it is
    /// a statement about the open document's pages.
    export_image: Option<export_image::ExportImageDialog>,

    /// The Export-text window, when one is open.
    ///
    /// **Document-scoped**, for its neighbour's reason: every control in it is
    /// a statement about the open document's pages.
    export_text: Option<export_text::ExportTextDialog>,

    /// The unsaved-edits confirmation, when one is open.
    ///
    /// **Document-scoped in subject and deliberately NOT closed by
    /// [`Self::close_document_scoped`]**, which is the one exception in this
    /// struct and is worth stating where the field is.
    ///
    /// Every other document-scoped dialog describes a document that is still
    /// open, so dropping it when the document goes is right. This one describes
    /// a document that is *about to* go, and the act that closes it is the act
    /// this window authorised. Clearing it there would be harmless today and
    /// would be a trap the moment anything else called `close_document` — the
    /// window would vanish mid-question and the operator would be left having
    /// been asked nothing.
    ///
    /// It is cleared by its own answer instead, in `PdfcerApp`'s drain, which is
    /// the only place that can know the question was finished with.
    unsaved: Option<unsaved::UnsavedDialog>,

    /// The Open-in-Acrobat confirmation, when one is open — O122.
    ///
    /// ★★ **Document-scoped**, and it is the one classification here worth
    /// arguing. The window is about handing *this file* to Acrobat: its
    /// sentences name the file and count its unsaved edits, and both facts stop
    /// being true the moment the document closes. A window left up over a
    /// closed document would be offering to hand over something that is no
    /// longer open.
    ///
    /// ⚠ It is nonetheless **not** cleared by [`Self::close_document_scoped`],
    /// and that is deliberate rather than an omission: the act this window
    /// authorises *is* a close, so clearing it on close would destroy the
    /// answer at the exact moment the answer is being carried out. It is
    /// cleared by its own drain, like [`Self::unsaved`] and
    /// [`Self::signature`], which is the correct owner of a window whose
    /// outcome outlives the document it was asked about.
    open_in_acrobat: Option<open_in_acrobat::OpenInAcrobatDialog>,

    /// ★ Set when the Open-in-Acrobat question was answered with Cancel.
    ///
    /// The twin of [`Self::unsaved_cancelled`] and it exists for the same
    /// reason: a Cancel parks no outcome, so a drain reports nothing, which is
    /// indistinguishable from "not answered yet". Without this the application
    /// would have no way to trace `acrobat-cancelled` — and a driven check
    /// cannot tell a cancelled question from an ignored one by looking at the
    /// screen, because both leave the document exactly where it was.
    open_in_acrobat_cancelled: bool,
    /// **The operator answered Cancel**, parked because the window is dropped
    /// on that answer and the answer must outlive it.
    ///
    /// ★★★ `UnsavedDialog::answered` reports whether an OUTCOME is parked, and
    /// a Cancel parks none — it closes the window and nothing else. So the
    /// retire rule correctly drops the dialog on a Cancel, and without this the
    /// fact that the operator said *no* went with it.
    ///
    /// ★★ Invisible until O102's quit cycle needed it. Every previous caller
    /// treats a Cancel as *"nothing happened"* — true for a tab close, where the
    /// tab simply stays. The cycle has to STOP, and *"they cancelled"* and
    /// *"they have not answered yet"* are the two states it must tell apart or
    /// it re-asks forever. The driven check saw exactly that: two
    /// `unsaved-asked` lines and no `quit-cancelled`.
    unsaved_cancelled: bool,

    /// The signature warning that stands in front of an invalidating save,
    /// when one is open.
    ///
    /// **Document-scoped, and NOT closed by [`Self::close_document_scoped`]**
    /// — the second exception in this struct, and it is the same exception as
    /// [`Self::unsaved`]'s for a slightly different reason worth stating.
    ///
    /// That window survives the close because the close is the act it
    /// authorised. This one survives because the act it authorises — a write —
    /// is performed by `crate::app::lifecycle` in the drain *after* the
    /// dialogs draw, and any path that closed the document in between would
    /// take the question away from an operator who is mid-answer. There is no
    /// such path today; the field is written so there cannot be one later.
    ///
    /// Like `unsaved`, it is cleared by its own answer, in `PdfcerApp`'s drain,
    /// which is the only place that can know the question was finished with.
    signature: Option<signature::SignatureDialog>,
}

impl DialogsState {
    /// **Open the Set-scale dialog with a reference line already measured.**
    ///
    /// The calibration path's entry point, raised by the application on the
    /// click that completes the two-point pick.
    ///
    /// # ★ It REPLACES an open dialog, where [`Self::open_scale`] refuses to
    ///
    /// That guard exists so a second press of the ribbon control does not
    /// discard what the operator has half typed. The situations are opposite
    /// here: the operator asked to measure on the drawing, the dialog closed
    /// so they could, and they have now finished. A guard that refused would
    /// leave them looking at a stale window with no measurement in it —
    /// the one outcome the whole gesture exists to avoid.
    /// **Has a window asked to step aside?** — `OPERATOR_REQUESTS.md` O66.
    ///
    /// Read-and-clear, and it answers the page the asking dialog is placing
    /// on so `canvas::placing` can record it. One arm per kind, listed rather
    /// than wildcarded so a second `PlaceKind` has to be wired rather than
    /// silently never asked.
    pub fn take_place_request(
        &mut self,
        page: usize,
    ) -> Option<(crate::canvas::placing::PlaceKind, usize)> {
        let d = self.insert_image.as_mut()?;
        d.take_place_request()
            .then_some((crate::canvas::placing::PlaceKind::Image, page))
    }

    /// **Hand a placed rectangle back to the window that asked for it.**
    ///
    /// ★ The window is not reopened, because it was never closed — see
    /// `dialogs::placing`. It simply starts drawing again on the next frame,
    /// with the numbers this writes into it.
    pub fn deliver_placement(
        &mut self,
        kind: crate::canvas::placing::PlaceKind,
        rect: pdfcer_core::page_tree::Rect,
    ) {
        match kind {
            crate::canvas::placing::PlaceKind::Image => {
                if let Some(d) = self.insert_image.as_mut() {
                    d.place(rect);
                }
            }
        }
    }

    /// **Is the window that asked for this placement still here?**
    ///
    /// ★★ The one guard that closes every exit route nobody enumerated. If the
    /// document is closed under a pending placement the dialog is dropped
    /// (`forget_document`), and without this the canvas would sit in a
    /// placement tool waiting for a window that no longer exists. `app::frame`
    /// checks it once a frame and cancels, which costs one `Option` read.
    #[must_use]
    pub fn has_requester(&self, kind: crate::canvas::placing::PlaceKind) -> bool {
        match kind {
            crate::canvas::placing::PlaceKind::Image => self.insert_image.is_some(),
        }
    }

    /// Whether the open Set-scale dialog is asking to start the two-point pick.
    ///
    /// Read-and-clear, so the caller cannot re-arm on every frame by forgetting
    /// to reset it.
    pub fn take_scale_calibrate_request(&mut self) -> bool {
        self.scale
            .as_mut()
            .is_some_and(scale::ScaleDialog::take_calibrate_request)
    }

    /// Draw every open dialog, and close the ones that asked to close.
    ///
    /// Called once per frame from frame composition, **after** the canvas and
    /// the docks: a dialog is an overlay, and egui's `Area` ordering follows
    /// the order things are added within a frame.
    ///
    /// # Why a closed document closes the DOCUMENT-SCOPED dialogs
    ///
    /// A print job is a job on this file's pages. A dialog left up over a
    /// closed document would be configuring a job against pages that no
    /// longer exist, and the honest response is to close it rather than to
    /// freeze it or to let it act on whatever is opened next.
    ///
    /// # ★ …and why About is drawn either way
    ///
    /// It is about pdfcer, not about a document. Closing it when the document
    /// closes would make `file.about` — a command every mode offers, with no
    /// `enabled_when` — open a window that vanished on the same frame
    /// whenever the canvas was empty. That is a control that does nothing,
    /// and it would look exactly like a bug in the command dispatch rather
    /// than like a rule about dialog lifetime.
    ///
    /// The early return therefore covers only the first group. Both are drawn
    /// first and closed after, rather than closed inside the borrow that drew
    /// them: a dialog decides whether it stays open *while* it draws (the
    /// title-bar cross and its own Close button are both widgets), so the
    /// answer arrives out of the same call that needs `&mut` on the state
    /// being dropped.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        status: &Status,
        actions: &mut Vec<crate::app::actions::Action>,
        window: Option<isize>,
        keymap: Option<&egui_shell::manifest::Keymap>,
        registry: &egui_shell::CommandRegistry,
    ) {
        // Application-scoped first, so that an empty canvas cannot skip it.
        // Ordering is the whole guard here: putting this after the early
        // return below is a one-line edit that would silently restore the old
        // behaviour, which is why it is above it rather than beside it.
        if self.about.as_mut().map(|d| d.show(ctx)) == Some(false) {
            self.about = None;
        }
        // Beside About, above the guard, and for a sharper version of the same
        // reason: this window's whole purpose is to produce a document, so a
        // guard that closed it when none was open would close it exactly when
        // it was needed.
        if self.new_document.as_mut().map(|d| d.show(ctx, actions)) == Some(false) {
            self.new_document = None;
        }
        // ★★★ Beside them, and ABOVE the no-document guard, for the sharpest
        // version of the same reason in this function: the document this window
        // is about is **not open**. That is its entire premise. A guard that
        // required an open document would close the password prompt in exactly
        // the state it exists for.
        if self.password.as_mut().map(|d| d.show(ctx, actions)) == Some(false) {
            self.password = None;
        }
        // Application-scoped, above the no-document guard with About and the
        // sized-New window. A keyboard reference an operator opened before
        // loading anything must not vanish because nothing is loaded.
        if self
            .shortcuts
            .as_mut()
            .map(|d| d.show(ctx, keymap, registry))
            == Some(false)
        {
            self.shortcuts = None;
        }

        let Status::Open(doc) = status else {
            self.close_document_scoped();
            return;
        };
        let doc: &OpenDoc = doc;
        if self.print.as_mut().map(|d| d.show(ctx, doc, window)) == Some(false) {
            self.print = None;
        }
        if self.ocr.as_mut().map(|d| d.show(ctx, doc, actions)) == Some(false) {
            self.ocr = None;
        }
        if self.diagnostics.as_mut().map(|d| d.show(ctx, doc)) == Some(false) {
            self.diagnostics = None;
        }
        // ★ `actions` since 2026-09-04: the apply dialog's default destination
        // now edits the OPEN document, so it pushes through the funnel — §5.
        if self.redact.as_mut().map(|d| d.show(ctx, doc, actions)) == Some(false) {
            self.redact = None;
        }
        if self.protect.as_mut().map(|d| d.show(ctx, doc)) == Some(false) {
            self.protect = None;
        }
        if self.insert_pages.as_mut().map(|d| d.show(ctx, actions)) == Some(false) {
            self.insert_pages = None;
        }
        if self.insert_image.as_mut().map(|d| d.show(ctx, actions)) == Some(false) {
            self.insert_image = None;
        }
        if self.export_dxf.as_mut().map(|d| d.show(ctx, actions)) == Some(false) {
            self.export_dxf = None;
        }
        if self.export_image.as_mut().map(|d| d.show(ctx, actions)) == Some(false) {
            self.export_image = None;
        }
        if self.export_text.as_mut().map(|d| d.show(ctx, actions)) == Some(false) {
            self.export_text = None;
        }
        if self.embed.as_mut().map(|d| d.show(ctx, actions)) == Some(false) {
            self.embed = None;
        }
        if self.compact.as_mut().map(|d| d.show(ctx, actions)) == Some(false) {
            self.compact = None;
        }
        if self.unembed.as_mut().map(|d| d.show(ctx, actions)) == Some(false) {
            self.unembed = None;
        }
        // ★ The Manage-dimension-groups WINDOW used to be drawn here, and the
        // comment that stood in its place said the order was load-bearing —
        // its *Set scale…* button parked a request drained on the next line.
        //
        // It is a dock panel as of 2026-08-19 (`crate::panels::Panel::DimensionGroups`),
        // because a window taller than the screen can push its own title bar
        // off the desktop and the operator could not close it. **The hand-over
        // survived the move and moved with it**: a panel body cannot reach
        // `DialogsState` at all, so it still parks a `GroupId` — now on
        // `crate::panels::PanelsState` — and `crate::app::PdfcerApp::docks`
        // drains it into [`Self::open_scale`] the moment the dock releases its
        // borrows. Same one-shot, same guards, one layer out.
        // ★ Takes the action queue, unlike its four neighbours. See the field.
        // It does not take `doc`: the scale it sets belongs to a *group*, which
        // is document-scoped but not page-scoped, and the entry fields need
        // nothing from the open document at all.
        if self.scale.as_mut().map(|d| d.show(ctx, actions)) == Some(false) {
            self.scale = None;
        }
        if self.form_field.as_mut().map(|d| d.show(ctx, actions)) == Some(false) {
            self.form_field = None;
        }
        if self.text_annot.as_mut().map(|d| d.show(ctx, actions)) == Some(false) {
            self.text_annot = None;
        }
        // ★ LAST, and the position is load-bearing in a way none of its
        // neighbours' are.
        //
        // This window's answer destroys or replaces the open document. Drawing
        // it before its siblings would let a frame exist in which the operator
        // has pressed *Close without saving*, `PdfcerApp` has not drained the
        // answer yet, and every dialog above is still drawing over a document
        // that is about to stop existing. Nothing would crash — the drain
        // happens between frames — but the ordering that makes that true should
        // be a statement rather than an accident, because the accident is one
        // reorder away and its failure mode is a surface describing a document
        // nobody has any more.
        //
        // ★★★ [`retire`] rather than `== Some(false)`, and the difference is a
        // defect: this window PARKS its answer for `resume_after_unsaved` and
        // pressing a button is what makes `show` say `false`. Dropping it on
        // that `false` threw the answer away with it.
        if self
            .unsaved
            .as_mut()
            .is_some_and(|d| retire(d.show(ctx), d.answered()))
        {
            // ★★★ **Park the cancellation before the window goes.** A Cancel
            // parks no outcome, so `retire` correctly drops the dialog and the
            // fact that the operator said *no* would go with it. The field's own
            // doc comment carries why that was invisible until O102's quit cycle
            // needed it, and what the driven check saw.
            if self
                .unsaved
                .as_ref()
                .is_some_and(unsaved::UnsavedDialog::was_cancelled)
            {
                self.unsaved_cancelled = true;
            }
            self.unsaved = None;
        }
        // ★★ O122 — beside the unsaved question rather than among the
        // document-scoped windows above, because it belongs to the same
        // family: it PARKS an answer that the application acts on, and the act
        // is a close. `retire` rather than `== Some(false)`, for the reason its
        // two neighbours record with receipts — pressing a button is what makes
        // `show` say `false`, and dropping the dialog on that `false` throws
        // the answer away with it.
        if self
            .open_in_acrobat
            .as_mut()
            .is_some_and(|d| retire(d.show(ctx), d.answered()))
        {
            if self
                .open_in_acrobat
                .as_ref()
                .is_some_and(open_in_acrobat::OpenInAcrobatDialog::was_cancelled)
            {
                self.open_in_acrobat_cancelled = true;
            }
            self.open_in_acrobat = None;
        }
        // ★ LAST of all, one place beyond the unsaved question, and the
        // position is argued the same way its neighbour's is.
        //
        // This window's answer WRITES — over the operator's own file, on the
        // in-place route. Drawing it before its siblings would let a frame
        // exist in which the operator has pressed *Save anyway*, `PdfcerApp`
        // has not drained the answer yet, and every dialog above is still
        // drawing over a document whose bytes are about to be replaced on
        // disk. Nothing would crash — the drain happens between frames — but
        // the ordering that makes that true should be a statement rather than
        // an accident.
        //
        // After `unsaved` rather than before it because the two can only ever
        // be raised on separate gestures (see `dialogs::signature`'s §7), and
        // if a future change ever makes both live at once the destructive-est
        // question should be the one on top.
        //
        // ★★★ [`retire`] rather than `== Some(false)`, for its neighbour's
        // reason and with the receipt: on 2026-08-29
        // `an_invalidating_save_is_warned_about` clicked *Save anyway*, the
        // window closed, and `signature-confirmed` never appeared — because
        // this line had already destroyed the dialog the answer was sitting in.
        if self
            .signature
            .as_mut()
            .is_some_and(|d| retire(d.show(ctx), d.answered()))
        {
            self.signature = None;
        }
    }

    /// **Take the operator's answer to the signature warning**, if they have
    /// given one.
    ///
    /// Drained by `crate::app::PdfcerApp` immediately after [`Self::show`], for
    /// the reason [`Self::take_unsaved_answer`] is: the act it authorises — a
    /// write — belongs to the application, not to a dialog. A window that
    /// could call `save_in_place` would be a second route to the one operation
    /// in this shell that can destroy the operator's file.
    ///
    /// ★ **It clears the window on the way out.** The answer and the window's
    /// lifetime are one fact, and separating them is how a confirmation gets
    /// answered once and acted on every frame — which here means writing the
    /// operator's file sixty times a second.
    pub fn take_signature_answer(&mut self) -> Option<signature::PendingSave> {
        let answer = self.signature.as_mut()?.take_confirmation()?;
        self.signature = None;
        Some(answer)
    }

    /// **Ask before `pending` if this save would invalidate a signature.**
    ///
    /// Returns `true` when the question was raised and the caller must
    /// **stop** — the save is now this window's to authorise. `false` means
    /// there was nothing to ask about and the caller saves as before.
    ///
    /// # ★ The return value is "did I interrupt you", exactly as
    /// [`Self::ask_unsaved`]'s is
    ///
    /// And for the identical reason, restated because it is the property that
    /// makes the guard safe to add to a third save route later: a guard read
    /// as *"may I proceed"* fails **open** when somebody inverts it or forgets
    /// it, and the unannounced write happens. Read this way it fails
    /// **closed** — a missing `if` raises the question and its answer performs
    /// the save anyway, so the operator sees one redundant window rather than
    /// a signed document silently rewritten.
    ///
    /// The already-open guard is the same one, and it matters here for the
    /// same reason it matters there: `Ctrl+S` held down while the question is
    /// on screen would otherwise replace the pending save with a second one,
    /// and an operator who asked for a copy would get an in-place write.
    pub fn ask_signature(&mut self, status: &Status, pending: signature::PendingSave) -> bool {
        if self.signature.is_some() {
            // Already asking. Swallow the second request rather than stacking
            // it — the operator is looking at a question and has not answered
            // it, and the honest reading of a second press is impatience.
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                "signature-ask-ignored reason=already-asking".to_owned()
            });
            return true;
        }
        let Some(dialog) = signature::ask_for(status, pending) else {
            return false;
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // ★ The pending save is IN the line. A reader of a trace from a
            // machine they cannot see needs to know which write was held: an
            // in-place save held at this question is the operator's own file
            // still carrying its previous bytes, and a copy held here is
            // simply a file that never appeared.
            format!("signature-asked pending={pending:?}")
        });
        self.signature = Some(dialog);
        true
    }

    /// Take the operator's answer to the unsaved-edits question, if they have
    /// given one.
    ///
    /// Drained by `crate::app::PdfcerApp` immediately after [`Self::show`], for
    /// the reason every hand-over in this module has one: the act it authorises
    /// — closing, opening, replacing — belongs to the application, not to a
    /// dialog, and a window that could call `close_document` would be a second
    /// route to the most destructive operation this shell has.
    ///
    /// ★ **It clears the window on the way out.** The answer and the window's
    /// lifetime are one fact, and separating them is how a confirmation gets
    /// asked twice — or, worse, answered once and acted on every frame.
    pub fn take_unsaved_answer(&mut self) -> Option<(unsaved::PendingIntent, unsaved::Outcome)> {
        let answer = self.unsaved.as_mut()?.take_outcome()?;
        self.unsaved = None;
        Some(answer)
    }

    /// **Ask before handing the open document to Acrobat** — O122.
    ///
    /// Returns `true` when the question was raised and the caller must
    /// **stop**: the handover is now this window's to authorise.
    ///
    /// # ★ The return value is "did I interrupt you", exactly as
    /// [`Self::ask_unsaved`]'s and [`Self::ask_signature`]'s are
    ///
    /// And for the identical reason, which is worth restating because this is
    /// the third guard to take the shape: a guard read as *"may I proceed"*
    /// fails **open** when somebody inverts it or forgets it, and here that
    /// means a document closed and handed to another program with nothing
    /// asked. Read this way it fails **closed** — a missing `if` raises the
    /// question, and the operator sees one redundant window rather than losing
    /// a document off their screen without warning.
    ///
    /// The already-asking guard is the same one its two siblings carry: a
    /// second press while the question is on screen is impatience, not a
    /// second request, and stacking it would leave a window nobody can dismiss.
    pub fn ask_open_in_acrobat(
        &mut self,
        prompt: crate::acrobat::Prompt,
        viewer: crate::acrobat::Viewer,
        edits: u64,
        file: String,
    ) -> bool {
        if self.open_in_acrobat.is_some() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                "acrobat-ask-ignored reason=already-asking".to_owned()
            });
            return true;
        }
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // ★ The prompt shape is IN the line. A reader of a trace from a
            // machine they cannot see needs to know which of the three
            // questions was asked: `SaveFirst` means the operator had unsaved
            // work at that moment, and `NoFileOnDisk` means nothing was ever
            // going to happen.
            format!("acrobat-asked prompt={prompt:?} edits={edits}")
        });
        self.open_in_acrobat = Some(open_in_acrobat::OpenInAcrobatDialog::new(
            prompt, viewer, edits, file,
        ));
        true
    }

    /// Take the operator's answer to the Open-in-Acrobat question.
    ///
    /// Drained by `crate::app::PdfcerApp` immediately after [`Self::show`], for
    /// the reason every hand-over in this module has one: the acts it
    /// authorises — a save, a close and a process launch — belong to the
    /// application. A window that could call `close_document` would be a
    /// second route to the most destructive operation this shell has, and one
    /// that could spawn a process would be the only place in the crate that
    /// did.
    ///
    /// ★ **It clears the window on the way out**, so a confirmation cannot be
    /// answered once and acted on every frame — which here would mean starting
    /// Acrobat sixty times a second.
    pub fn take_open_in_acrobat_answer(
        &mut self,
    ) -> Option<(open_in_acrobat::Outcome, crate::acrobat::Viewer)> {
        let answer = self.open_in_acrobat.as_mut()?.take_outcome()?;
        self.open_in_acrobat = None;
        Some(answer)
    }

    /// **Was the Open-in-Acrobat question cancelled?** Drains the flag.
    ///
    /// ★ Draining rather than peeking, so one cancel produces one trace line.
    /// A flag that stayed set would have the application reporting a
    /// cancellation on every frame until the next question was asked.
    pub fn take_open_in_acrobat_cancelled(&mut self) -> bool {
        std::mem::take(&mut self.open_in_acrobat_cancelled)
    }

    /// Drop the state of every dialog that is about the open document.
    ///
    /// One place, so a document-scoped dialog added later cannot be forgotten
    /// by whichever of the close paths its author did not think of.
    /// Application-scoped dialogs are deliberately absent — see
    /// [`Self::show`].
    fn close_document_scoped(&mut self) {
        self.print = None;
        self.ocr = None;
        self.diagnostics = None;
        self.redact = None;
        self.protect = None;
        self.scale = None;
        self.insert_image = None;
        self.export_dxf = None;
        self.export_image = None;
        self.export_text = None;
        self.embed = None;
        self.unembed = None;
        self.compact = None;
    }

    /// **Ask for the password of `path`**, unless we are already asking for it.
    ///
    /// Called from the frame when the active document is
    /// [`crate::app::state::Status::NeedsPassword`]. Idempotent on the path, so
    /// the frame can call it unconditionally every frame — which is what makes
    /// it safe to drive from a state rather than from an event, and is why the
    /// prompt survives the operator clicking away and coming back.
    ///
    /// ★ Re-asking for a **different** document replaces the prompt. One
    /// password box at a time is the convention everywhere, and two would leave
    /// the operator guessing which file each belonged to — the prompt names its
    /// file for the same reason.
    pub fn ask_for_password(&mut self, path: &std::path::Path) {
        if self.password.as_ref().is_some_and(|d| d.path() == path) {
            return;
        }
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("password-prompt path={path:?}")
        });
        self.password = Some(password::PasswordDialog::new(path.to_path_buf()));
    }

    /// **Tell the prompt its last attempt failed**, and keep it open.
    ///
    /// Called when a retry comes back refused. Returns `false` when there is no
    /// prompt to tell — which happens if the operator cancelled between
    /// submitting and the load returning, and is not an error.
    pub fn reject_password(&mut self, why: password::Rejection) -> bool {
        let Some(d) = self.password.as_mut() else {
            return false;
        };
        d.reject(why);
        true
    }

    /// **Close the prompt**, because the document opened.
    pub fn password_accepted(&mut self) {
        if self.password.is_some() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "password-accepted".to_owned()
            });
            self.password = None;
        }
    }
}

// The dialog owner's assertions. Split out on 2026-09-04 under R2; see its
// header for why the tests were the seam and the code was not.
#[cfg(test)]
mod tests;
