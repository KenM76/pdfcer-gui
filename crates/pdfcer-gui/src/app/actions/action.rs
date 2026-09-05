//! # `app::actions::action` — the action vocabulary
//!
//! One item: the [`Action`] enum. See [`super`]'s header and `OVERVIEW.md` for
//! what an action *is*, when it is raised, when it is applied, and why the
//! funnel exists at all; this file is the list.
//!
//! ## Why it is a file of its own
//!
//! Rule R2, and the seam [`super`]'s header had already half-drawn. That
//! header says, of the day the module's prose moved to `OVERVIEW.md`:
//!
//! > this file has exactly one seam and it is not where the lines are: the
//! > whole body is **one enum**, which cannot be split without inventing a
//! > nested variant and rewriting every match arm in the crate.
//!
//! Both halves of that are still true. The enum still cannot be split
//! internally without a sub-enum, and the bulk is still prose — but *"one
//! enum"* and *"the module that declares six submodules and re-exports the
//! disclosure recorder"* are two subjects, and putting them in two files is
//! the ordinary Rust seam between a module and its principal type.
//!
//! ## ★ What this does NOT buy, stated so nobody has to find out
//!
//! Headroom. This file is close to the ceiling on the day it was made, and the
//! next family of variants to grow will have to become a sub-enum beside
//! `PageAction` and `DimensionAction`. [`super`]'s declaration of this module
//! carries the measurement of which family that should be and why.

use super::{ViewChrome, dimensions, pages};
use crate::viewer::FitMode;

/// One operator intent, applied after the frame that raised it.
///
/// Every variant is reachable from a real control today. A variant nothing
/// can raise is dead code wearing a design pattern, and the "no
/// placeholders" invariant (`PROJECT_PLAN.md` §3) applies to enums as much
/// as to labels.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// ★★★ **Select exactly this object** — raised by the Objects panel when a
    /// row is clicked.
    ///
    /// # Why a panel raises an action instead of writing the selection
    ///
    /// Because a panel body is handed `&OpenDoc`, not `&mut`, and that is
    /// deliberate: a surface that could mutate the document while it is being
    /// drawn is a surface that can change what a later widget in the same frame
    /// is describing. Every other panel that changes something raises an action
    /// for the same reason, and this is not the place to make an exception.
    ///
    /// # What it replaced
    ///
    /// `PanelsState::focus` — a second notion of *"the thing I am working on"*,
    /// written only by the Objects panel and read only by the Properties panel,
    /// which the canvas neither wrote nor read. The audit of 2026-08-26 found
    /// three such notions in parallel (the armed tool, the panel focus, the
    /// canvas selection) with no bridge between them, and named it the cause of
    /// the operator's *"when I have an object selected like text the Tool tab
    /// doesn't switch to giving me the editable stuff for that object."*
    ///
    /// Now there is one, written from both ends.
    SelectObject {
        /// The page the object is on, in the session's page space.
        page: usize,
        /// Which object, as a paint-order target — or `None` to select
        /// nothing.
        ///
        /// ★ `None` rather than a second variant, because a row click is one
        /// act with one outcome: *this row is now the selection*. Clicking the
        /// already-selected row makes that selection empty, which is what
        /// clicking a selected item does in every list in every application,
        /// and splitting it into Select and Clear would make the caller decide
        /// which act it was performing when it only ever performs one.
        object: Option<crate::canvas::target::TargetId>,
    },
    /// **Open this document, replacing whatever is open.**
    ///
    /// Raised by `file.open` once the picker has answered, by `file.recent`
    /// once the operator has chosen a row, and by nothing else — the two
    /// surfaces that can name a file. It is deliberately *not* raised by
    /// `crate::run`'s `argv` path, which calls
    /// [`PdfcerApp::open_path`] directly: there is no frame yet, so there is
    /// no frame to defer to, and routing it through an empty action queue
    /// would be ceremony rather than discipline.
    ///
    /// # Why the path travels, and why it is already absolute by the time
    /// anyone stores it
    ///
    /// Same reason [`Self::DeleteSelection`] carries its operand list: an
    /// action is a complete statement of intent, resolvable after the frame
    /// that raised it. The picker's answer cannot be re-derived later —
    /// the dialog is gone — so the only alternative would be a field on the
    /// application holding a half-finished intent between frames, which is
    /// precisely the state the funnel exists to avoid.
    ///
    /// Absolutizing happens in [`crate::app::recent::RecentFiles::remember`]
    /// rather than here, because it is a property of *storing* a path, not of
    /// opening one: `Document::load` is perfectly happy with a relative path
    /// and the operator's shell already resolved it.
    Open(std::path::PathBuf),

    /// **Open a document that needs a password, with one.**
    ///
    /// Raised only by [`crate::dialogs::password::PasswordDialog`], which is the
    /// only surface that can obtain a password. `OPERATOR_REQUESTS.md` O108.
    ///
    /// ★ **A separate variant rather than an `Option` on [`Self::Open`]**,
    /// because `load(path)` and `load_with_password(path, Some(pw))` are
    /// different requests and `Some("")` is a third — see
    /// [`crate::app::lifecycle::PdfcerApp::open_path_with_password`], which
    /// carries the argument.
    ///
    /// ★★★ **The password travels in a [`crate::secret::Secret`]**, whose whole
    /// purpose is a `Debug` that cannot print the value. That module's header
    /// carries the hazard in full; the one-line version is that this enum
    /// derives `Debug`, this crate traces to stderr, and `tools/ui-verify` keeps
    /// that stderr as evidence. `dialogs::password` asserts it on *this variant*
    /// rather than on the type alone.
    OpenWithPassword {
        /// The file to open.
        path: std::path::PathBuf,
        /// What the operator typed.
        password: crate::secret::Secret,
    },

    /// ★★★ **Put a completed recognition into the open document, as one
    /// undoable edit.**
    ///
    /// # Why an action rather than the dialog doing it
    ///
    /// Because `EditSession::add_ocr_layer` needs `&mut EditSession`, and a
    /// dialog body is handed `&OpenDoc`. That is not an inconvenience to route
    /// around — it is the rule that stops a window mutating a document while
    /// the frame that drew it is still reading one. Every other edit in this
    /// shell reaches the session the same way.
    ///
    /// # ★★ One entry for the whole run, however many pages
    ///
    /// The words for every page travel together and are applied in one call.
    /// Recognising forty pages and then pressing undo forty times is not a
    /// feature — the engine's own commit message says so in those words, and
    /// `CommandKind::AddOcrLayer` is one command for the slice.
    ///
    /// # What it replaced
    ///
    /// A `Vec<u8>` of a whole PDF, a Save-as picker, an in-place file write and
    /// a document reload — all of which existed because
    /// `ocr::layer::add_ocr_layer` took an immutable `&Document` and could not
    /// touch a session. The engine's Pass 135.0 removed the cause on
    /// 2026-08-27 and all of it went with it.
    ApplyOcr {
        /// The recognised words, paired with the page each belongs to.
        ///
        /// Paired rather than two vectors, matching `OcrPageLayer`'s own
        /// reasoning: two lists can differ in length or order, and either
        /// mistake puts one page's words on another page with no diagnostic
        /// short of reading the output.
        pages: Vec<(usize, pdfcer_core::ocr::OcrPage)>,
    },

    /// **Make a blank document, replacing whatever is open.**
    ///
    /// Raised by `file.new` and by nothing else. Carries no operand: unlike
    /// [`Self::Open`] there is nothing to name — the bytes are compiled in
    /// (`crate::app::blank::TEMPLATE`) and the document's own name is derived
    /// after the frame from a counter on the application, which is where a
    /// number that outlives one document belongs.
    ///
    /// Applied by [`PdfcerApp::new_document`], which carries the reasoning:
    /// where the template comes from, why the engine has no part in it, and
    /// why New leaves the operator's mode alone.
    /// **Remove the selected annotation from the document.**
    ///
    /// Raised by `format.delete` on the contextual Format tab and by the
    /// canvas's Delete/Backspace keys, both only while an annotation is
    /// selected — the same two controls that raise [`Self::DeleteSelection`]
    /// for page content, which is deliberate: one gesture, one meaning,
    /// whichever kind of thing is under it.
    ///
    /// # ★ Why this is a separate variant and not a case of `DeleteSelection`
    ///
    /// Because they name different things and call different verbs.
    /// `DeleteSelection` carries paint-order indices into page **content** and
    /// applies `delete_objects`; this carries a stable `ObjId` and applies
    /// `delete_annotation`. Folding them together would mean one variant whose
    /// payload had to be interpreted before it could be acted on, and the
    /// interpretation would be the thing that could be got wrong — with page
    /// content as the failure mode, which is the loss this project's own
    /// deletion guard calls *"one line and the whole view"*.
    ///
    /// # What it does not promise
    ///
    /// **This is not redaction.** It removes an entry from `/Annots`; it does
    /// not touch page content, and an incremental save leaves the previous
    /// revision in the file. `docs/core-api/03-capabilities.md` §3.4 states
    /// that rule and `crate::text::markup::deleted_collateral` observes it in
    /// the wording it chooses.
    /// **Paste a copied markup onto `page`**, displaced by `(dx, dy)`.
    ///
    /// Raised by [`crate::canvas::clipboard::paste`] and by nothing else.
    ///
    /// # ★ Why the displacement travels rather than being applied at the copy
    ///
    /// Because where a paste lands depends on **which page it lands on**, and
    /// that is not known until the operator presses the key: the same clipboard
    /// entry offsets on its home page (so the copy is visible instead of
    /// hidden under its original) and lands in place on any other (so a
    /// revision cloud copied to sheet 12 is where it was on sheet 1). Baking a
    /// displacement into the clipboard at copy time would make the second
    /// behaviour impossible without a second clipboard entry.
    ///
    /// # Why the spec is boxed
    ///
    /// `MarkupSpec` is a large enum — its widest variant carries a vertex list,
    /// a border effect and a style — and `Action` is passed by value through the
    /// funnel. One `Box` here keeps every *other* action cheap, which is the
    /// same trade `CommitTextAnnot` makes for the same reason.
    PasteMarkup {
        /// The 0-based page to author onto — **the page on screen now**, not
        /// the one it was copied from.
        page: usize,
        /// The spec, verbatim from `annot_author::spec_from_dict`.
        spec: Box<pdfcer_core::annot_author::MarkupSpec>,
        /// Horizontal displacement, PDF points.
        dx: f64,
        /// Vertical displacement, PDF points. **Negative is down** — y
        /// increases upward in PDF user space.
        dy: f64,
        /// ★★★ **The keys the spec cannot carry** — `/CA`, `/Contents`, `/T`
        /// and `/M` — read off the annotation at COPY time and reproduced
        /// verbatim.
        ///
        /// Not re-derived from the operator's current pen, deliberately: a
        /// paste is a reproduction of what was copied, and one that picked up
        /// today's opacity would be a different mark wearing the copied one's
        /// geometry. `canvas::clipboard::carried_options` carries the whole
        /// argument, including why this field will need to grow again.
        options: Box<pdfcer_core::edit::MarkupOptions>,
    },
    /// ★★★ **The verbs whose subject is a whole annotation** — move it, resize
    /// it, remove it.
    ///
    /// Moved into [`super::annot::AnnotAction`] under R2 on 2026-08-28. Its
    /// header carries the property that makes the three a family: they find
    /// their operand by **stable object id**, so none of them takes a page to
    /// locate one — and `Delete`'s page, which looks like a counter-example, is
    /// for the trace and the disclosure only.
    Annot(super::annot::AnnotAction),
    New,
    /// **Make a new document at a chosen sheet size.**
    ///
    /// Raised by [`crate::dialogs::new_document`] and by nothing else —
    /// `file.new_from_template` opens that dialog rather than acting, because
    /// the size is a question and a command cannot ask one.
    ///
    /// Applied by [`PdfcerApp::new_document_sized`], behind the same
    /// `save_pending` guard [`Self::New`] takes. The two are one verb with two
    /// sources of the sheet, and a second guard would eventually disagree with
    /// the first.
    ///
    /// # Why points, and why a size rather than a rectangle
    ///
    /// **Points** because that is the unit the engine's `/MediaBox` is in and
    /// the unit this action is one step away from writing. The dialog speaks
    /// millimetres to the operator and converts once, at the edge, which is
    /// the same discipline `pdfcer_core::paper` applies to its own table.
    ///
    /// **A size and not a rectangle** because a new page's lower-left corner
    /// is the origin and there is nothing to decide about it. Carrying a full
    /// rectangle would invite a caller to offer an offset page, which is a
    /// legal PDF and a thing no New command should be able to produce by
    /// accident. It also keeps this enum free of a `pdfcer_core` type, which
    /// every other variant already is.
    NewSized {
        /// Sheet width in points, after orientation has been applied.
        width_pt: f64,
        /// Sheet height in points, after orientation has been applied.
        height_pt: f64,
    },
    /// **Write the open document, edits and all, to a file the operator
    /// names.**
    ///
    /// Raised by `file.save_copy` and by nothing else. Applied by
    /// [`crate::app::save::save_copy`], whose header carries every decision in
    /// this feature: why the save mode is **incremental** (a promise already
    /// shipped in the command's tooltip), why nothing on
    /// [`crate::app::state::OpenDoc`] moves, and which `SaveOptions` fields
    /// were chosen and why.
    ///
    /// # ★ Why it carries no path, when [`Self::Open`] carries one
    ///
    /// Because there is no operand to carry. `Open`'s path is the **answer to a
    /// dialog that is gone by the time the action is applied** — it cannot be
    /// re-derived, so it travels, and its picker therefore runs during dispatch.
    /// A save has the opposite shape: what to *suggest* is a pure function of
    /// the open document (`save::suggested_path`), so nothing is lost by asking
    /// later, and asking later is required rather than merely allowed.
    ///
    /// `crate::app::files::pick_save_path` documents a **frame-timing
    /// requirement** — a native modal opened inside an `egui` layout closure
    /// blocks the frame it is being drawn in, leaving a half-painted window
    /// behind a dialog. Dispatch does not always satisfy it:
    /// `crate::app::PdfcerApp::central` dispatches the canvas's context-menu
    /// tokens from *inside* `egui::CentralPanel::show`. The apply phase always
    /// does — it is step 3, after every surface has closed. So raising an action
    /// is not ceremony here; it is the only placement that honours the
    /// requirement from every route the command can be invoked by.
    ///
    /// # It is matched before the document guard
    ///
    /// With nothing open there is nothing to write, and a keymap can reach
    /// `Ctrl+S` from any state. The guard's silent drop would make that
    /// indistinguishable from a chord that never arrived, so — like
    /// [`Self::Find`] — this is answered above it, by name, on the trace.
    SaveCopy,
    /// **Save As** (O95) — a separate act from [`Self::SaveCopy`], not a
    /// configured one: [`crate::app::save::save_as`] carries the argument.
    SaveAs,

    /// ★★★ **Save. In place. Over the file that was opened.**
    ///
    /// The operator, 2026-08-20: *"can I please have a save button like every
    /// other program in existence has? We're on week two of this and just have
    /// a save as button."*
    ///
    /// Matched beside [`Self::SaveCopy`] and above the document guard for the
    /// identical reason: a keymap can reach `Ctrl+S` from any state, and a
    /// silent drop with nothing open is indistinguishable from a chord that
    /// never arrived.
    ///
    /// # Why it is a separate variant and not `SaveCopy` with a path
    ///
    /// Because they are different acts with different risks. Save-a-copy writes
    /// a file that did not exist; Save-in-place is the only verb in this
    /// application that can **destroy the operator's work**, which is why
    /// `save::save_in_place` materialises the whole replacement in a temporary
    /// beside the target and then renames. Folding them together would put that
    /// distinction inside an `Option` and invite a caller to pass the wrong one.
    Save,
    /// **Close the open document and return to [`Status::Empty`].**
    ///
    /// Raised by `file.close`, which is gated on `doc.open`, so the no-document
    /// case is unreachable from the ribbon — and handled anyway, because a
    /// customized keymap can reach any command from any state and an action
    /// that assumed otherwise would be a panic waiting for an operator to
    /// find it.
    ///
    /// Carries nothing: it always means **the document on screen**, which is
    /// what `file.close` and `Ctrl+W` mean everywhere. Closing a document that
    /// is *not* on screen is [`Self::CloseDocument`], raised only by the ✕ on
    /// a tab and by a middle click on one.
    Close,
    /// ★★★ **Hand the open document to Acrobat and stop being the program that
    /// has it** — O122, raised by `file.open_in_acrobat` beside the mode
    /// selector. Carries nothing: always the document on screen. It raises a
    /// question and does nothing else, as [`Self::Close`] does and for its
    /// reason; `crate::app::actions::acrobat` holds the whole argument,
    /// including why the drain's order is save → launch → close.
    OpenInAcrobat,
    /// **Close the document in tab position `0`-based slot.**
    ///
    /// Raised by the document tab strip's ✕ and by a middle click on a tab,
    /// and by nothing else. [`Self::Close`] is the *active* document's verb
    /// and stays separate rather than becoming `CloseDocument(active_slot)`,
    /// for the reason `crate::app::actions::document`'s header gives about
    /// `New` and `NewSized`: two arms that are deliberately adjacent, so a
    /// change to what either guard means cannot be applied to one and missed
    /// on the other. It also keeps the trace able to say which control closed
    /// a document.
    ///
    /// # ★ It may activate the tab before it asks
    ///
    /// Closing a **modified** background tab shows that document first and
    /// then asks, because a question about unsaved edits over a document the
    /// operator cannot see is a question they have no way to answer. Word and
    /// VS Code both do exactly this. A clean background tab closes where it
    /// stands, because there is nothing to ask and switching to it would be a
    /// visible jolt for no reason.
    ///
    /// # Why a slot and not a path
    ///
    /// Because two tabs can name the same path only in states
    /// `crate::app::documents` §3 makes unreachable, and because a created
    /// document's path is a name rather than a location. The slot is the
    /// identity the strip itself is indexed by, so a click cannot resolve to a
    /// different document than the one under the pointer.
    CloseDocument(usize),
    /// **Close every open document except the one at this tab position.**
    ///
    /// Raised by `window.close_other_documents` on the tab strip's context
    /// menu, and by nothing else.
    ///
    /// # ★ It is one action and N closes, and the N is the point
    ///
    /// Each document is closed through the **same** guarded path a single close
    /// takes, one at a time, so a modified one still brings itself to the front
    /// and asks. The operator therefore answers a question per document that
    /// has unsaved work and none for the rest — which is what every editor
    /// does, and the only behaviour that does not either discard work silently
    /// or ask N times about nothing.
    ///
    /// A cancelled answer stops the sequence where it is. That is deliberate:
    /// *"close the others"* is a convenience, and an operator who cancels
    /// halfway has said something about the whole gesture, not just about one
    /// document.
    CloseOtherDocuments(usize),
    /// **Copy pages out of another open document into the one on screen.**
    ///
    /// Raised by a page drag that was released over a document other than the
    /// one it started in — from the Pages panel's grid or from the page view —
    /// and by nothing else. A drag released in the document it started in is a
    /// reorder and raises [`PageAction::ReorderPages`] instead.
    ///
    /// # ★ The target is not carried, and that is deliberate
    ///
    /// It is always the **active** document. A drop lands on a surface, the
    /// surface is showing whatever is active, and a target slot captured when
    /// the drag *started* would name exactly the wrong document — spring-loaded
    /// tabs mean the active document changes mid-gesture, which is the whole
    /// mechanism that lets a page reach another document at all.
    ///
    /// # It is a copy unless the operator held Shift
    ///
    /// Unmodified, the source document is not touched: no page is removed from
    /// it, its undo stack is unchanged, and its tab does not acquire the
    /// unsaved marker. With Shift held at the moment of release it is a
    /// **move** — see [`Self::InsertPagesFromOpenDocument::take`], and
    /// [`crate::app::actions::crossdoc`] §2 for why the copy is the default
    /// rather than the other way round.
    InsertPagesFromOpenDocument {
        /// The tab position of the document the pages come from.
        ///
        /// A slot rather than a path, for [`Self::CloseDocument`]'s reason: it
        /// is the identity the strip is indexed by, and a created document has
        /// no path that names a file.
        source_slot: usize,
        /// Which of the source's pages, 0-based, in the order they were
        /// picked.
        pages: Vec<usize>,
        /// Where they land in the target, in the engine's own vocabulary —
        /// [`PageAction::InsertPagesFromFile`]'s field, verbatim and for the
        /// same reason.
        position: pdfcer_core::pageops::InsertPosition,
        /// **Remove them from the source afterwards** — the Shift-held move.
        ///
        /// Windows' own drag modifier: Ctrl copies, Shift moves. Two documents
        /// are two files with two undo stacks, which is the "different volumes"
        /// case, so the unmodified drag copies and this is what asks for
        /// something else.
        ///
        /// ★ Sampled at the **release**, not at the press. That is what
        /// Explorer does — the cursor badge changes under your hand as you
        /// press and release the key mid-drag — and it is what lets an operator
        /// start a drag, see the caption say *copy*, and change their mind
        /// without letting go.
        ///
        /// ★★ It is TWO edits in two documents and the operator is told so.
        /// One `Ctrl+Z` reverses one half. `crate::text::doctabs::moved_out_of`
        /// is the sentence that says it, and it is the reason this is the
        /// modified gesture rather than the default.
        take: bool,
    },
    /// **A canvas gesture refused because what is selected lives inside a
    /// form XObject** — say so on the status bar.
    ///
    /// # ★ The one action that changes nothing and is still an action
    ///
    /// The worded-decline store is `pub(super)` inside `crate::app` on purpose
    /// — *"a decline is written by the one dispatcher and read by the one
    /// bar"* — while the gesture that has something to declare happens in
    /// `crate::canvas`. Raising an action rather than widening that visibility
    /// is what this crate does everywhere the same shape occurs: a surface
    /// holding `&OpenDoc` and not `&mut` **asks**. The full argument is at
    /// `canvas::moving::decline`, which is the only thing that raises this.
    ///
    /// **No payload**, deliberately: there is one thing to say and
    /// `text::status::selection_inside_form_declined` says it. A `Declined`
    /// payload would turn this into a general "print any decline" channel,
    /// which is how a single choke point becomes a bypass.
    ///
    /// ★ **A selection is still not an edit** — no `vector_edit`, no epoch
    /// bump, no cache invalidation, for the reason `Action::SelectObject`
    /// gives.
    DeclineInsideForm,
    /// Multiply the current zoom by a factor — the Ctrl+wheel path.
    ///
    /// Carries the factor rather than a target zoom because that is what
    /// egui's `zoom_delta` reports, and because the *clamp* must be applied
    /// by the state machine that owns the ceiling, not by the widget.
    ZoomBy(f32),
    /// Step to the next zoom-ladder rung above the current zoom.
    ZoomIn,
    /// Step to the next zoom-ladder rung below the current zoom.
    ZoomOut,
    /// Enter a fit mode — a *mode*, not a one-shot: the zoom is re-derived
    /// from the viewport every frame until an explicit zoom pins it.
    Fit(FitMode),
    /// Pin the zoom to an exact factor.
    ///
    /// **`Fit(FitMode::None)` is not this**, and the difference was a live
    /// defect the status bar surfaced. `FitMode::None` only stops the
    /// per-frame re-fit; it leaves `zoom` wherever it happened to be. So an
    /// "Actual size" control raising it pinned 73 % *at* 73 % while its
    /// tooltip promised one PDF point per screen point — a control whose
    /// label and behaviour disagreed, on two surfaces at once, because the
    /// ribbon's `view.zoom_actual` raised the same thing.
    ///
    /// `ZoomBy(1.0 / zoom)` would land on the right number and is still
    /// wrong: it routes a discrete command through the wheel path, which
    /// carries the 150 ms settle debounce that exists for continuous
    /// gestures. A command that arrives in one piece should commit in one
    /// piece.
    ZoomTo(f32),
    /// Step one page toward the end of the document.
    NextPage,
    /// Step one page toward the start of the document.
    PrevPage,
    /// Jump to a 0-based page index, clamped into the document.
    GoToPage(usize),
    /// ★★ **Everything that changes page GEOMETRY** — delete, the four move
    /// verbs, the Bézier handle, and the transform.
    ///
    /// Split into its own enum on 2026-08-20 under R2, when `TransformObjects`
    /// took this file past the 1,500-line ceiling. It is the seam this file
    /// already demonstrates twice — [`Self::Dimension`] and [`Self::Page`] — and
    /// it is a real subject rather than a size-driven cut: every variant in it
    /// addresses **paint-order indices into one page's content stream**, which
    /// nothing else here does, and every one of them is subject to
    /// `docs/core-api/02` §1.10.1's renumbering rule.
    Vector(super::vector::VectorAction),
    // =======================================================================
    // ★ THE PAGE VERBS — structural edits, and the family that renumbers
    //
    // Four variants for five commands (`pages.rotate_left` and
    // `pages.rotate_right` share one), plus `pages.extract`, which is not here
    // at all — see `ExtractPages` below for why it is and `pages.split` /
    // `pages.merge_into` / `pages.insert_from_file` for why they are not.
    //
    // # Why the operand list travels, and is not re-derived at apply time
    //
    // The same argument `DeleteSelection` makes, one structure up: the operand
    // is the **Pages panel's multi-select**, resolved once by
    // `crate::panels::pages::ops::operands`. Re-reading `PanelsState` during
    // the apply would be a second reading of a set the operator may have
    // changed between the frame that raised the action and the frame that
    // applies it — and for `DeletePages` the consequence of reading it twice
    // is destroying sheets nobody chose.
    //
    // # ★ What separates these from every action above them
    //
    // Everything else in this enum either leaves the page *count* and the page
    // *order* alone, or is not a document edit at all. These three do neither,
    // and that is why `crate::app::actions::apply::page_edit` exists beside
    // `vector_edit` rather than each arm doing its own bookkeeping: a page
    // delete or reorder invalidates the flattened page vector, every cached
    // raster keyed on a page index, the canvas selection's page identity and
    // the panel's own picks, all at once, and a missed one is a stale picture
    // or a verb aimed at the wrong sheet.
    /// **Author one markup annotation on the page** — the release of a band
    /// drag, the release of a freehand stroke, or the ending of a vertex run.
    ///
    /// ★ Raised by [`crate::canvas::markup`]'s gesture paths and by nothing on
    /// the ribbon. A `markup.*` command *arms a tool*; the tool draws; the
    /// gesture raises this. There is no path from a button to an annotation,
    /// which is the whole point of the substrate.
    ///
    /// # ★★★ Why an action carries the geometry rather than deriving it
    ///
    /// The old shell had the other arrangement and it produced the defect the
    /// markup work exists to fix: its `Action::AddMarkupShape` derived a
    /// rectangle from the page's own media-box centre and inserted it, so the
    /// shape appeared in the middle of the page *"no matter where the operator
    /// had been pointing"*. The operator's report was exact — **"they just drop
    /// things into the center of the pdf window."** An action that carries
    /// geometry the operator never supplied is not a shortcut; it is the
    /// feature not working, and it passes any test that asks whether an
    /// annotation was added.
    ///
    /// # Units, and why the endpoints are RAW
    ///
    /// Endpoints are **PDF user-space** points, Y-**up**, produced by
    /// [`crate::canvas::markup::endpoints`] — the one place a markup gesture
    /// crosses out of canvas space — and they are in **gesture order**,
    /// deliberately un-normalised: for an arrow the first is the tail and the
    /// second is the head, and normalising them into a rectangle here would
    /// silently reverse every arrow drawn up-and-left or up-and-right.
    /// [`crate::canvas::markup::spec`] normalises per kind, at the one moment a
    /// rectangle is actually needed, and carries the full argument.
    ///
    /// # Why the page travels
    ///
    /// The same reason it does on [`Self::DeleteSelection`]: an action is a
    /// complete statement of intent, resolvable after the frame that raised it.
    /// Re-deriving the page from `doc.view.page_index` in the apply would be a
    /// second source of truth that is right until a page step raised in the
    /// same frame is applied first.
    ///
    /// # ★ Why THREE gestures share one variant, where text markup got its own
    ///
    /// Because [`crate::canvas::markup::spec`] is *"the single place a gesture
    /// becomes a `MarkupSpec`"*, and that claim is what the equivalence with
    /// `pdfcer markup-add` rests on: a canvas-authored annotation has to be
    /// byte-identical to a CLI-authored one, and the cheapest way to keep two
    /// things identical is for there to be one of them. Three variants would be
    /// three apply arms, each free to build its own spec, and the day one of them
    /// acquired a normalisation the others did not is the day the guarantee
    /// quietly stopped holding — with nothing to notice it, because every arm
    /// would still author a perfectly valid annotation.
    ///
    /// So the geometry became an enum
    /// ([`crate::canvas::markup::Geometry`]) rather than the variant becoming
    /// three. [`Self::CommitTextMarkup`] stays separate for the reason its own
    /// docs give and the reason is *different*: its operand is not a gesture at
    /// all — it is a text selection that already exists on the document — so it
    /// shares no rule with anything here.
    CommitMarkup {
        /// The 0-based page the annotation is authored onto.
        page: usize,
        /// Which shape — and therefore which `/Subtype`, pen and normalisation
        /// rule. See [`crate::canvas::markup::spec`].
        kind: crate::canvas::markup::MarkupKind,
        /// The geometry the gesture produced, **in PDF user space**: two raw
        /// drag endpoints, a run of clicked vertices, or one or more freehand
        /// strokes. Which of the three is a property of the kind, and the pairing
        /// is checked by [`crate::canvas::markup::action`] before this is ever
        /// built.
        geometry: crate::canvas::markup::Geometry,
        /// ★ **The pen the operator had when the gesture completed**, carried
        /// in the action rather than read at apply time.
        ///
        /// The funnel's whole premise is that an `Action` is *plain data
        /// describing an edit*, and the colour and width are part of what the
        /// edit is — not context to be looked up later. Reading the live pen in
        /// the apply arm would author a mark in whatever colour the operator
        /// happened to have selected by the time the queue drained, which for a
        /// queue is a real gap and not a theoretical one: the dispatcher raises
        /// actions during the frame and `apply` runs at the end of it.
        ///
        /// It also makes the action **replayable**, which the variant's own
        /// docs already claim of the rest of its fields: an `Action` a test
        /// builds, or a future undo/redo surface re-runs, authors the same
        /// annotation it did the first time rather than the same shape in a
        /// different colour.
        pen: crate::canvas::markup::pen::Pen,
    },

    /// ★ **Everything the ce-dimension feature asks for**, as one variant
    /// carrying which.
    ///
    /// Eight verbs — author a ce dimension, create a group, calibrate one, set
    /// its drafting standard, set its appearance defaults, show or hide its
    /// layer, override one ce dimension's style, and switch a circular one
    /// between radius and diameter. They live in
    /// [`dimensions::DimensionAction`] rather than as eight variants here, and
    /// that type's own documentation gives the three reasons.
    ///
    /// The short version is the one that matters at this level: **four of them
    /// rewrite every member of a group, across every page it has members on,
    /// and four touch one annotation.** That routing rule is a property of the
    /// family rather than of any one verb, so it is expressed once — as
    /// `DimensionAction::regenerates_the_whole_group` — where a ninth verb has
    /// to pick a side in order to compile. Flat variants would have re-derived
    /// it in eight arms, and the day one of them got it wrong the symptom
    /// would be a stale number on a page the operator was not looking at.
    Dimension(dimensions::DimensionAction),
    /// ★ **Everything the operator asks of the SET OF PAGES**, as one variant
    /// carrying which.
    ///
    /// Five verbs — insert another document's pages, rotate, delete, reorder,
    /// extract. They live in [`pages::PageAction`] rather than as five variants
    /// here, and that type's own documentation gives the reasons.
    ///
    /// The short version is the one that matters at this level: **every one of
    /// them can renumber the document**, and what each of them does to the
    /// shell's *derived* state afterwards is different — a rotation preserves
    /// both selections, a reorder remaps the panel's picks and clears the
    /// canvas's, a delete clears both, an insert navigates. That rule is a
    /// property of the family rather than of any one verb, and expressing it in
    /// one place is what stops a sixth page verb being added with the wrong
    /// invalidation and nothing noticing — because nothing would: every
    /// individual edit would still be correct in the document and wrong only
    /// on screen.
    Page(pages::PageAction),
    /// ★★ **Everything whose subject is one entry in the document's outline**
    /// — add, rename, and delete-with-its-subtree.
    ///
    /// Moved into [`super::bookmarks::BookmarkAction`] under **R2** on
    /// 2026-08-28, when `pdfcer-core` `Pass 156.0` turned a one-verb family into
    /// a three-verb one. This file's own header named the rule in advance:
    /// *"the next family of variants to **grow** is the one that will have to
    /// become a sub-enum beside `PageAction` and `DimensionAction`."*
    ///
    /// Its header carries the two things a reader must not have to rediscover:
    ///
    /// * **Every variant addresses its operand by `ObjId`, never by a position
    ///   in the tree**, because an outline is renumbered by every edit to it —
    ///   the defect the engine hit in its own CLI, where *"the output looked
    ///   entirely plausible"*.
    /// * **`/Count` is two different quantities and its SIGN carries
    ///   open-or-closed** (§12.3.3), which is why no verb in the family
    ///   describes itself by diffing a count.
    Bookmark(super::bookmarks::BookmarkAction),
    /// ★★ **Everything whose subject is a whole FILE living inside the
    /// document** — attach, remove, and save one out (§7.11.4.1).
    ///
    /// A sub-enum from the day it was written, under **R2** and this file's own
    /// rule: a family that arrives with three verbs at once has grown before
    /// anybody had to measure it.
    ///
    /// Its header carries the two things a reader must not have to rediscover:
    ///
    /// * **All three open a native file dialog**, so all three are `Action`s
    ///   for `super::write`'s reason as well as the funnel's — a modal OS window
    ///   must not open inside a layout pass.
    /// * **Nothing any of them does appears on the page.** An embedded file is
    ///   reached from the catalogue and is drawn nowhere, so every one of them
    ///   owes a sentence to `crate::app::status` — the exact inverse of
    ///   [`Self::Bookmark`]'s rename, which owes none because the row the
    ///   operator is looking at already says what happened.
    Attachment(super::attachments::AttachmentAction),
    /// ★★★ **Everything whose subject is a form XObject** — one verb today:
    /// give this page its own private copy of a shared drawing.
    ///
    /// A sub-enum from the day it was written, under **R2** and this file's own
    /// rule, for a reason [`super::xobject`]'s header states in full: this file
    /// was at 1,441 of 1,500 lines, and the variant needs a doc comment it
    /// cannot afford. [`Self::Attachment`] is the precedent for arriving as a
    /// sub-enum rather than growing into one.
    ///
    /// Its header carries the two things a reader must not have to rediscover:
    ///
    /// * **The granularity is one PAGE, not one invocation**, and that is the
    ///   engine's decision. A page that draws the same form under three names
    ///   has all three re-pointed at the one copy — so there is deliberately no
    ///   variant here that names *which* invocation was clicked, because that
    ///   would offer a granularity `unshare_form` does not implement.
    /// * **The operand is resolved before the action is raised.**
    ///   `crate::app::dispatch::format` reads the selection's first leaf and
    ///   asks for its **outermost** enclosing form's `ObjId`; the innermost one
    ///   is exactly what `EditError::FormNestedInAnotherForm` refuses. A
    ///   `TargetId` is not resolvable after the frame that raised it and an
    ///   `ObjId` is, which is what the funnel requires of an operand.
    XObject(super::xobject::XObjectAction),
    /// ★★★ **Re-shape the page's own text** — a reflow today, and the caret
    /// and restyle commits when they follow.
    ///
    /// Moved into [`super::text::TextAction`] under R2 on 2026-08-28. Its
    /// header carries the one thing a reader must not assume: reflow does NOT
    /// accumulate the way its neighbours do, so a page already edited this
    /// session refuses it by name.
    Text(super::text::TextAction),
    /// ★★★ **Write something out to a file the operator picks.**
    ///
    /// Three verbs — DXF, form data, and a compacted copy — moved into
    /// [`super::write::WriteAction`] under R2 on 2026-08-28. Its header carries
    /// the one property they share and no other family here does: they are
    /// `Action`s **only** because a native file dialog must not open inside a
    /// layout pass.
    Write(super::write::WriteAction),
    /// **Put the font programs a document references but does not carry into
    /// it**, as one undoable command.
    ///
    /// Raised by `crate::dialogs::embed` and by nothing else.
    ///
    /// # ★★ Why the whole request travels, donor bytes and all
    ///
    /// Because it IS the operand, and it cannot be rebuilt at apply time
    /// without changing it. Reconstructing it there would re-scan the
    /// operator's font folders, and a file added to one while the window was
    /// open would resolve a different donor - so the operator would confirm one
    /// thing and commit another. The dialog closes on the frame that raises
    /// this, so nothing else is still holding the bytes by the time the queue
    /// drains.
    ///
    /// # ★ Why it is boxed
    ///
    /// `Action` is moved through a queue by value on every gesture, and its
    /// size is the largest variant's. The request carries a map of donor
    /// programs; boxing keeps the cost on the one gesture that has it rather
    /// than on every zoom.
    EmbedFonts {
        /// Which fonts, and the program the shell resolved for each.
        request: Box<pdfcer_core::font_embed_missing::EmbedRequest>,
    },
    /// **Take the embedded font programs OUT of a document**, as one undoable
    /// command.
    ///
    /// Raised by `crate::dialogs::unembed` and by nothing else.
    ///
    /// # ★ Why it carries a request at all, when the selection is always the
    /// # same today
    ///
    /// The dialog sends `UnembedSelection::AllRemovable` every time, so this
    /// could be a unit variant like [`Self::ExportFormData`]. It carries the
    /// request because the request is what `unembed_preview` was called with,
    /// and the preview the operator READ is only the plan for that exact
    /// request. Rebuilding it in the apply arm would put a second constructor
    /// on the path between what was shown and what is done - which is the
    /// property `embed_preview` and `embed_fonts` are built to have and the one
    /// this variant exists to preserve.
    ///
    /// Boxed for [`Self::EmbedFonts`]' reason: `Action` moves through a queue
    /// by value on every gesture and its size is the largest variant's.
    UnembedFonts {
        /// Which fonts, and what happens to their subset tags.
        request: Box<pdfcer_core::font_unembed::UnembedRequest>,
    },
    /// ★ **Place a raster image on the page.**
    ///
    /// Raised by `crate::dialogs::insert_image` and by nothing else.
    ///
    /// # Why the imported picture travels in the action
    ///
    /// Because it is the **operand**, and an `Action` is a complete statement
    /// of intent resolvable after the frame that raised it. The dialog closes
    /// on the same frame it commits, so nothing else would still be holding the
    /// bytes by the time the queue drains.
    ///
    /// `Arc`, not the value: an `ImportedImage` owns the decoded or re-encoded
    /// stream — megabytes for a scan — and moving a clone through the queue
    /// would double the peak for no gain. The engine takes it by reference, so
    /// the apply arm never copies it either.
    ///
    /// # Why the rectangle is in POINTS
    ///
    /// The dialog asks in millimetres, because that is what a drafter measures
    /// in, and converts **once** — in one function that the validity check, the
    /// landing preview and this field all read. Carrying millimetres here would
    /// put a second conversion in the apply arm and give the window two chances
    /// to disagree with the document about where the picture went.
    ///
    /// # What the apply arm owes afterwards
    ///
    /// `add_image` returns an `ImageAuthorOutcome` whose `disclosures` are
    /// **all facts the operator cannot see at editing zoom**: the effective
    /// resolution, whether the shape was preserved or stretched, and whether
    /// pdfcer re-encoded the source rather than storing its bytes. Every one of
    /// them looks identical on screen and different on a plot, which makes them
    /// rule 4's surviving half exactly. They are returned from `vector_edit`'s
    /// closure, which is how they reach the status bar.
    InsertImage {
        /// The 0-based page it is placed on, frozen when the dialog opened.
        page: usize,
        /// The box, in PDF user space.
        rect: pdfcer_core::page_tree::Rect,
        /// What happens when the box's shape differs from the picture's.
        fit: pdfcer_core::edit::ImageFit,
        /// The imported picture. See above for why it is an `Arc`.
        image: std::sync::Arc<pdfcer_core::image_import::ImportedImage>,
    },
    /// ★ **Set or clear one of the document's own information fields** —
    /// `/Title`, `/Author`, `/Subject`, `/Keywords`.
    ///
    /// Raised by `crate::panels::properties::info` and by nothing else.
    ///
    /// ★ **`Option<String>` is not a defaulted `String`** — `None` REMOVES the
    /// key from `/Info` and `Some("")` writes an empty string object, which are
    /// different files. And it goes through the funnel for the undo log rather
    /// than for size: the panel re-seeds its text drafts on an epoch bump, which
    /// is what makes `Ctrl+Z` visibly restore the old value in the box. Both
    /// arguments in full: [`crate::panels::properties::info`].
    SetInfoField {
        /// Which field. The engine's own enum, carried unchanged, so a field
        /// added to `pdfcer-core` reaches this variant without a translation
        /// layer that could grow its own opinion.
        field: pdfcer_core::edit::InfoField,
        /// The new value, or `None` to remove the key entirely.
        value: Option<String>,
    },
    /// ★ **Mark the text the operator has selected** — underline, strikeout or
    /// squiggly.
    ///
    /// Raised by `crate::app::dispatch` when one of the three Text markup
    /// commands is invoked, through
    /// [`crate::canvas::markup::text::mark`], which is where every rule about
    /// *which* selection is eligible lives.
    ///
    /// # Why this is not [`Self::CommitMarkup`] with a different kind
    ///
    /// Because the operand is a different shape and no amount of naming hides
    /// it. `CommitMarkup` carries **two points** — a drag — and normalises or
    /// preserves them per kind; this carries **a list of quads**, one per line
    /// of a text selection, already in PDF user space and already grouped. A
    /// single variant would have to carry both and leave half of itself empty
    /// for every value, which is the shape that makes an apply arm ask *which
    /// kind is this again* before it can read its own operands.
    ///
    /// # Why the quads travel, and why the page travels with them
    ///
    /// An action is a complete statement of intent, resolvable after the frame
    /// that raised it — the property [`Self::DeleteSelection`] is built on. The
    /// selection it came from may be cleared by the same frame's Escape or
    /// replaced by a click before this is applied, so the quads are copied out
    /// at the moment the operator asked. The `page` is the **selection's**, not
    /// `doc.view.page_index`: a selection made on one sheet and marked after
    /// paging away must mark the sheet it was made on.
    CommitTextMarkup {
        /// The 0-based page the annotation is authored onto — the page the
        /// selection was made on.
        page: usize,
        /// Which subtype, and therefore which appearance the engine draws.
        kind: crate::canvas::markup::text::TextMarkKind,
        /// The selected lines' boxes, PDF user space, in content order —
        /// `crate::canvas::textsel::TextSelection::page_quads`, which is the
        /// same list the wash was painted from.
        quads: Vec<pdfcer_core::annot_author::Quad>,
        /// The pen at the moment the command was invoked.
        ///
        /// ★ **Added 2026-08-17, closing a shipped inconsistency.** This variant
        /// went without a pen for the whole of the project because
        /// `crate::canvas::markup::text` held its own hard-coded red — and when
        /// the Style group landed in `4035b64` and gave [`Self::CommitMarkup`]
        /// this field, the text sibling was not given it too. The result an
        /// operator saw: the swatch moved the colour of every drawn shape and
        /// none of the three text marks.
        ///
        /// The field carries the same three obligations its twin's does — see
        /// [`Self::CommitMarkup`]'s `pen`, which states them at length and is
        /// not repeated here — of which the load-bearing one is that the pen is
        /// **sampled when the operator asks**, not read when the queue drains.
        ///
        /// Only [`crate::canvas::markup::text::TextMarkKind::rgb`] reads it, and
        /// it takes the **ink**: these three kinds are lines, so they are the
        /// biro rather than the marker. Highlight is not in this variant at all.
        pen: crate::canvas::markup::pen::Pen,
    },
    /// ★ **Replace the words in ONE show operator** — `DEFECTS.md` D4's verb.
    ///
    /// One operator, one `EditSession::edit_text`, one undo entry. The scope
    /// limit is `pdfcer-core`'s and is stated on `EditRequest`: a request pins to
    /// one show operator, and a `TJ` array is one operator. A caret that landed
    /// where two runs meet never becomes this variant —
    /// `canvas::textedit::Refusal::SpansRuns` refuses it in a sentence first.
    ///
    /// # Why it carries the ORIGINAL as well as the replacement
    ///
    /// Because the engine's request is a find/replace, not an index-and-splice:
    /// `EditRequest::find` is *"the text to locate within one show operator's
    /// decoded run"*, and the surgery re-tokenises the content buffer to find
    /// it. The `run` index alone would not survive the round trip.
    ///
    /// # Why it does NOT carry the disposition
    ///
    /// That is the whole of D4b, so it is worth stating where it is not.
    /// `FollowerDisposition` is derived at apply time by
    /// `canvas::textedit::plan`, from the page **as it is when the action
    /// lands** — not from what the canvas believed when the key was pressed.
    /// Carrying it would make the choice a fact about a frame; deriving it makes
    /// it a fact about the document. The old shell's failure was of exactly the
    /// second kind read the first way: it wrote `EditOptions::default()` at its
    /// single call site and never asked the page anything.
    CommitTextEdit {
        /// The 0-based page holding the run.
        page: usize,
        /// Which run, by index into `PageText::runs` — the anchor for the
        /// provenance pin `plan` re-derives.
        run: usize,
        /// The run's text when the caret landed on it. `EditRequest::find`.
        original: String,
        /// What the operator typed. `EditRequest::replace`.
        replacement: String,
    },
    /// **Place NEW page text** — `edit.add_text`'s verb.
    ///
    /// Additive, and that is the difference from [`Self::CommitTextEdit`] rather
    /// than a detail: it rewrites no existing operator, so there is nothing whose
    /// form changed and the engine's own R46 additivity applies. It is
    /// deliberately **not** `Action::CommitMarkup` with a text kind — a markup
    /// text box is an annotation layered over the page and is removable by
    /// deleting it; this becomes the page's own content, exactly like the text
    /// already there, which is what the command's shipped tooltip promises.
    CommitAddText {
        /// The 0-based page.
        page: usize,
        /// Where the baseline starts, in **PDF user space** — the space
        /// `AddTextRequest::origin` is specified in, converted once at the click
        /// through `viewer::canvas_to_pdf_space` so no second conversion can
        /// disagree with the caret the operator saw.
        origin: (f64, f64),
        /// What the operator typed.
        text: String,
        /// ★★ **The face, size and colour it is written in**, sampled from
        /// `canvas::textedit::pen` at the moment the draft committed.
        ///
        /// Carried on the action rather than re-read in `apply`, and the rule
        /// is the funnel's own: an `Action` is *what the operator asked for*,
        /// and it is applied on a later frame. Re-reading the pen at apply time
        /// would let a pen changed between the two frames rewrite an edit the
        /// operator had already finished — the same hazard `DeleteSelection`'s
        /// operand list travels for, one value smaller.
        ///
        /// It is the shell's own type rather than the engine's `NewTextFace` /
        /// `NewTextColor` pair, because those are two values with one meaning
        /// and `TextPen` resolves them at the boundary. See its docs for why
        /// black is written `Black` and not `Rgb(0, 0, 0)`.
        pen: crate::canvas::textedit::pen::TextPen,
        /// ★★★ **The wrap rectangle**, in PDF user space, or `None` for a
        /// single-line run at [`Self::CommitAddText::origin`].
        ///
        /// The operator, 2026-08-21: *"I should be able to make it multi line."*
        ///
        /// `Some((llx, lly, urx, ury))` reaches `AddTextRequest::with_box`
        /// (`Pass 16.1`), which is what makes multi-line expressible at all: a
        /// PDF has no paragraph, so each visual line is its own show operator at
        /// its own absolute position, and *something* has to decide where the
        /// second line starts. A width to wrap against and a leading to step by
        /// is that something.
        ///
        /// ★ It gives the operator **both** behaviours from one field: hard
        /// newlines split paragraphs and each paragraph is wrapped
        /// independently to the box's width. So Enter makes a new paragraph and
        /// running past the right edge makes a new line, which is what anyone
        /// who has used a text box expects and is not two features.
        ///
        /// A tuple rather than the engine's `Rect` for the reason every other
        /// geometric field here is a tuple: this crate's `Action` is a
        /// statement of what the operator asked for, and it does not carry
        /// engine types across the funnel where a pair of primitives says the
        /// same thing. `apply` builds the `Rect`, once, at the boundary.
        wrap: Option<(f64, f64, f64, f64)>,
    },
    /// ★★ **Restyle a markup that is already on the page** — the action behind
    /// `EditSession::set_markup_style`, and the first caller that verb has ever
    /// had in this shell.
    ///
    /// It shipped in the engine on 2026-08-18 and had **zero GUI call sites**
    /// until 2026-08-19: it appeared only in doc comments, which
    /// `pdfcer`'s own capability register recorded as a ⬜ this project put
    /// there. Both blockers `shell::manifest::format`'s header named are
    /// discharged — the verb, and a selection model that can address an
    /// annotation — so what was left was work.
    ///
    /// # ★ The style carries ONE field, not a whole struct
    ///
    /// `MarkupStyle`'s every field is `Option`, and its own doc says why: *"a
    /// Format tab whose colour picker also had to restate the current width
    /// would overwrite whatever the operator had set from the other control."*
    ///
    /// So the surface raises one of these per control that changed, carrying
    /// the one field that changed, and never assembles a struct from what its
    /// widgets happen to be showing. The failure that prevents is a colour
    /// change that silently reverts a width set a moment earlier, from a widget
    /// that was one frame stale.
    ///
    /// # Why the page travels beside the id
    ///
    /// The verb does not need it — an `ObjId` names the annotation outright —
    /// and `vector_edit` does: it invalidates the page's raster and the strip's
    /// entry for it, and a restyle changes what that page draws. Deriving the
    /// page at apply time would mean asking which page an object is on, which
    /// is a graph walk for a number the surface already had.
    SetMarkupStyle {
        /// The 0-based page the annotation lives on.
        page: usize,
        /// The annotation, by object id — **stable**, unlike a content
        /// object's paint-order index.
        id: pdfcer_core::object::ObjId,
        /// What to change. Every field `None` but the one control that moved.
        style: pdfcer_core::edit::MarkupStyle,
    },
    /// Show or hide one optional-content group.
    ///
    /// **View state, not document state.** It changes what this session
    /// draws and never what a save would write — which is why it does not
    /// bump `edit_epoch` and why the Layers panel's own note says a toggle
    /// changes what you see and not the document.
    ///
    /// It does invalidate the page raster, and that is now expressible:
    /// `RenderKey` gained `layers_generation` in the same stage as this
    /// variant, honouring `render/worker.rs`'s rule that *"the key ships in
    /// the same commit as its control"*. Before that, a checkbox here would
    /// have redrawn nothing — which is why the panel shipped without one.
    SetLayerVisible {
        /// The optional-content group.
        group: pdfcer_core::object::ObjId,
        /// Whether it should be drawn.
        visible: bool,
    },
    /// Restore every optional-content group to the document's own default.
    ///
    /// Not "show everything": a document may declare groups that are off by
    /// default, and revealing those would be a different act from undoing
    /// the operator's own hiding.
    ResetLayers,
    /// **Choose how many pages are on screen, and in what arrangement.**
    ///
    /// The four positions of View ▸ Page display, as one action carrying which
    /// — because they are a radio and an action per position would be four
    /// arms doing the same thing to four different constants.
    ///
    /// # It is a view stance, and it still goes through the funnel
    ///
    /// Same nature as [`Self::ToggleAnnotations`] and [`Self::SetLayerVisible`]:
    /// it changes what is drawn and nothing a save would write, so it does not
    /// bump `edit_epoch`. It goes through the funnel anyway for the reason the
    /// funnel exists — the mode is changed from a ribbon button *while the
    /// canvas is drawing the old arrangement*, and applying it mid-frame would
    /// leave the frame's layout, its scroll offset and its texture lookups
    /// describing two different modes at once.
    ///
    /// # ★ Applying it does three things, and the third is why this is not a
    /// one-line arm
    ///
    /// 1. **sets `view.display`** — the arrangement itself;
    /// 2. **remembers it against this document**, through
    ///    [`crate::viewer::remembered`], which is the operator's requirement of
    ///    2026-08-12: *"so a sheet set does not inherit a report's setting."*
    ///    Recording it here rather than in the dispatcher is deliberate — a
    ///    customized keymap can reach the command too, and a choice made by a
    ///    chord must persist exactly as one made by a click;
    /// 3. **drops the strip's cached rasters**, because a mode change is the
    ///    one event that makes a *visible* page stop being visible. Leaving
    ///    them would hold GPU memory for pages that cannot be reached until the
    ///    operator switches back.
    SetPageDisplay(crate::viewer::PageDisplay),
    /// Show or hide annotations as a class.
    ///
    /// Same nature as [`Self::SetLayerVisible`] — a view stance, tracked by
    /// `RenderKey`, invisible to a save.
    ToggleAnnotations,
    /// **The three View ▸ Display chrome toggles** — rulers, grid, guides.
    ///
    /// One variant carrying which, rather than three variants, for the reason
    /// [`Self::SetPageDisplay`] gives about the page-display radio: the
    /// operand *is* the command, `crate::shell::commands::chrome_for_command`
    /// is the single binding between an id and a [`ViewChrome`], and its
    /// inverse is what publishes the `selected:` condition that renders each
    /// one pressed. Three arms would be three places for that mapping to be
    /// spelled and a fourth toggle would be added to two of them.
    ///
    /// # Why it goes through the funnel when it changes nothing a save writes
    ///
    /// The same reason `SetPageDisplay` does, and it is sharper here: the
    /// rulers change how much room the canvas has. Applying that in the middle
    /// of the frame that is *already laying the strip out into the old
    /// viewport* would leave the frame's fit scale, its scroll offset and its
    /// page rects describing two different canvases at once. Deferred, the
    /// next frame reserves the gutters once and everything downstream is
    /// consistent with them.
    ///
    /// Deliberately does **not** bump `edit_epoch`: nothing about the document
    /// has changed, only what is drawn beside and over it. Bumping would throw
    /// away the decomposition and the font inventory to no purpose.
    ToggleViewChrome(ViewChrome),
    /// **The operator's guides for this document, after a gesture changed
    /// them.**
    ///
    /// Carries the whole next collection rather than an add / move / remove
    /// verb. Three reasons, and the first is the one that decided it:
    ///
    /// 1. **The gesture already computes it.** `canvas::guides::release`
    ///    resolves create, move and delete through one table, and handing over
    ///    the result is the same "compute the next value from the previous one
    ///    and store it" shape the canvas already uses for the selection.
    /// 2. **The apply has exactly one thing to persist.** `guides.txt` is
    ///    rewritten from the whole set either way — it is a
    ///    read-modify-write of one line — so a verb would be decomposed here
    ///    and recomposed there.
    /// 3. **The operand is small.** Bounded by
    ///    `canvas::guides::MAX_PER_DOCUMENT`, twelve bytes each, and raised
    ///    once per *release* rather than once per frame of a drag.
    SetGuides(crate::canvas::guides::Guides),
    /// **One thing the operator asked Find to do** — run the search, or step
    /// to the adjacent hit.
    ///
    /// # Why a search goes through the funnel at all, when it changes nothing
    ///
    /// Because it needs `&mut EditSession`.
    /// [`pdfcer_core::edit::EditSession::find_text_with`] takes a mutable
    /// borrow — it is a read that mutates the session's own working state —
    /// and `OpenDoc::session` is an `Arc` precisely so the render worker can
    /// hold a clone while it rasterizes. `Arc::get_mut` fails while any other
    /// strong reference exists, so the worker has to be stopped first, and
    /// stopping a render **in the middle of laying out a frame** is exactly
    /// what this funnel exists to prevent. Applied after the frame, it is one
    /// short pause in a rasterization that was going to restart anyway.
    ///
    /// So the rule this variant honours is not the letter of
    /// "actions-not-mutations" (a search mutates no document) but its
    /// *reason*: do no expensive or ordering-sensitive work in the middle of
    /// a frame.
    ///
    /// # Why stepping is an action too, when it moves no bytes
    ///
    /// Because it **navigates**: moving to the next hit changes the page and
    /// the scroll offset, which is `Action::GoToPage`'s territory. Doing it
    /// in the widget would put a page change inside the frame that is already
    /// drawing the old page — the one-frame-late class of defect
    /// `crate::app`'s header describes for the whole apply phase.
    ///
    /// The operand is carried, exactly as [`Self::DeleteSelection`] carries
    /// its index list, because an action is a complete statement of intent:
    /// *which* way to step cannot be re-derived after the frame that asked.
    /// See `crate::find` for what happens on the other end.
    Find(crate::find::FindRequest),
    /// ★ **Everything done to a form FIELD**, as its own family — [`super::forms`].
    ///
    /// Eight verbs: fill a control, select one on the page, place one, author
    /// the one a dialog accepted, rename, delete a field, delete one of its
    /// widgets, and register a widget no field claims.
    ///
    /// # Why they moved out of this enum, 2026-08-27
    ///
    /// **R2**, and a real seam rather than a size-driven cut. They share a
    /// property nothing else here has: every one of them addresses a control by
    /// its **fully qualified name** or by the widget's `ObjId` — never by a
    /// paint-order index — because `/AcroForm` is document-level and a widget is
    /// reached through the field that claims it, not through the page that draws
    /// it. That is the same test [`super::vector`] passes for paint-order
    /// indices and [`super::pages`] passes for page positions.
    ///
    /// ★★ The move also repaired a documentation defect that no gate could see.
    /// Three `///` blocks had stacked contiguously onto `SelectFormField`, so
    /// rustdoc showed one variant carrying three unrelated explanations while
    /// `BeginFormField` and [`Self::BeginTextAnnot`] carried none. Doc comments
    /// concatenate silently, and a variant that loses its own is invisible to
    /// `check-ui-strings`, to clippy and to every test in this crate — the only
    /// instrument that finds it is a reader. Each block is back on its subject.
    Field(super::forms::FieldAction),
    /// ★★ **Change how existing text LOOKS** — size, colour, face, weight and
    /// slant — on every run the operator's text selection covers.
    ///
    /// Raised by the Properties panel's Text section and by nothing else yet;
    /// the Format tab's Font group is the second surface and takes the same
    /// variant.
    ///
    /// # Why the RUNS travel and the selection does not
    ///
    /// The same staleness rule [`Self::CommitTextAnnot`] follows, and here it
    /// is sharper: by the time the queue drains, an action ahead of this one in
    /// the same drain could have changed the text selection, and this one would
    /// then restyle text the operator was not looking at when they pressed. A
    /// list of run ordinals measured at the press is a complete statement of
    /// what they asked for.
    ///
    /// ★ The **page** travels for the same reason it does on every vector verb:
    /// a run ordinal means nothing without one, and re-deriving it at apply
    /// time would read whichever page the view had reached by then.
    ///
    /// # ★★ There is deliberately no pin on it
    ///
    /// A `pinned_span` is a byte offset into a content stream, and applying
    /// this action *rewrites that stream*. A pin measured here would be
    /// correct for the first run and wrong for every one after it. The apply
    /// arm re-resolves one per step, and `super::textstyle`'s header carries
    /// the argument — including why the runs are then walked backwards.
    TextStyle {
        /// The 0-based page the runs are on.
        page: usize,
        /// Which runs of that page's extraction, in any order; the arm sorts
        /// and reverses them itself.
        runs: Vec<usize>,
        /// The one property being changed. One press, one property, one undo
        /// entry — see the type's own docs.
        change: super::textstyle::StyleChange,
    },
    /// **A text-bearing annotation has been placed and now needs its words.**
    ///
    /// Raised by the canvas on the release (or click) that finishes the
    /// placing gesture, and by nothing else. It **changes no document** — it
    /// opens `crate::dialogs::textannot`, which is where the operator types.
    ///
    /// # ★ Why the geometry travels and the words do not
    ///
    /// The rectangle is the operator's choice and it is made *now*, on the
    /// page they were looking at, at the zoom they were at. The words are made
    /// later, in a dialog, and may never be made at all. Carrying the rect on
    /// the action is the same rule `CommitMarkup` follows for its pen: an
    /// `Action` is plain data describing what the operator did, and what they
    /// did was draw a box.
    BeginTextAnnot {
        /// The 0-based page the annotation will be authored onto.
        page: usize,
        /// Which text-bearing kind is being placed.
        kind: crate::canvas::textannot::TextAnnotKind,
        /// The rectangle, in PDF user space, already normalised.
        rect: pdfcer_core::page_tree::Rect,
    },
    /// **Author the text-bearing annotation the dialog just accepted.**
    ///
    /// Raised by `crate::dialogs::textannot` and by nothing else. This is the
    /// one that reaches the document.
    ///
    /// # ★ Everything it needs travels with it, including the stamp
    ///
    /// The same argument `CommitMarkup` makes about the pen, applied to three
    /// values instead of one: by the time the queue drains, the dialog is
    /// closed and its fields are gone. Reading them at apply time is not
    /// merely fragile, it is impossible — which is the tidier version of the
    /// hazard the pen has, where the value still exists and is simply the
    /// wrong one.
    CommitTextAnnot {
        /// The 0-based page.
        page: usize,
        /// Which kind to author.
        kind: crate::canvas::textannot::TextAnnotKind,
        /// The rectangle, in PDF user space.
        rect: pdfcer_core::page_tree::Rect,
        /// What the operator typed. Empty for a stamp, whose words are its
        /// `/Name`.
        text: String,
        /// The stamp chosen from the gallery. Ignored by the other two kinds,
        /// and carried unconditionally rather than as an `Option` because a
        /// gallery always has a selection — there is no "no stamp chosen"
        /// state for the dialog to be in.
        stamp: pdfcer_core::annot_author::StampName,
    },
    // =======================================================================
    // ★ THE REDACTION MARKING VERBS
    //
    // Three variants, all **reversible**, and that is the property that puts
    // them in this enum at all. Marking authors a `/Redact` annotation and
    // removes nothing; the engine records each one as an undoable command, so
    // every one of these goes through `vector_edit` exactly as a markup does
    // and `Ctrl+Z` takes it back.
    //
    // ★★★ CORRECTED 2026-09-04: applying can now change the OPEN document
    // (`Pass 250.1`), so `ApplyRedactionsIntoDocument` is below. The paragraph
    // that argued it never could be is quoted and answered in `actions::redact`.
    // =======================================================================
    /// **Mark every occurrence of some text for redaction.**
    ///
    /// Raised by [`crate::panels::redact`]'s Find & mark control. Applied
    /// through `vector_edit`, so it is one undoable command however many marks
    /// it creates — which is the right granularity: the operator asked one
    /// question, and taking back "mark every occurrence of this name" one
    /// annotation at a time would be unusable.
    ///
    /// # ★ The query is carried, not a hit list
    ///
    /// The panel could resolve the matches itself and push the quads, the way
    /// [`Self::CommitTextMarkup`] carries the selection's boxes. It must not,
    /// for a reason specific to this verb: `pdfcer-core`'s own
    /// `mark_redactions_by_search_with` documents the trap — a front end whose
    /// search and whose marking disagree about *which hits exist* produces
    /// "three highlights and eleven redaction marks", and *"on the one
    /// operation whose whole purpose is removing content irreversibly, 'the
    /// mark set is a superset of the highlight set' is not a cosmetic
    /// difference."* Handing the engine the query lets the engine answer both
    /// halves with one scan.
    MarkRedactionsBySearch {
        /// The text, already trimmed by the panel.
        query: String,
        /// Whether to read the query as a pattern (`#` any digit, `?` any
        /// character) rather than as literal text.
        ///
        /// A `bool` here rather than an enum, unlike
        /// `crate::redact::ResidualAcknowledgement` — because this one is
        /// *named at its field* and reads as a sentence at the one call site
        /// that builds it, while that one is a positional argument at a call
        /// site where a transposition would write a file.
        pattern: bool,
        /// How the marks this creates will look once applied.
        ///
        /// ★ Carried on the action, not read at apply time, and it is the same
        /// rule the pen follows for markup: the operator's choice is the one
        /// they had **when they pressed the control**. Reading it in the
        /// dispatcher would let a frame in which they also changed the fill
        /// swatch author marks they did not choose — and on this verb the
        /// difference is not cosmetic, because the appearance is baked into
        /// each `/Redact` annotation at creation and there is no verb that
        /// modifies one afterwards.
        appearance: pdfcer_core::annot_author::RedactAppearance,
    },
    /// **Mark the whole of one page for redaction.**
    ///
    /// Raised by [`crate::panels::redact`]'s Mark whole page control. The page
    /// is carried rather than read from `doc.view` at apply time, on
    /// [`Self::CommitTextMarkup`]'s rule: the operator marked the sheet they
    /// were looking at, and an action applied after a frame in which they also
    /// paged away must mark the sheet they meant.
    ///
    /// The rectangle is not carried, because it is not the operator's choice —
    /// it is the page's crop box, and `crate::panels::redact::whole_page_spec`
    /// is the one place that decision is made and tested.
    MarkPageForRedaction {
        /// The 0-based page to cover.
        page: usize,
        /// How the mark will look once applied. See
        /// [`Self::MarkRedactionsBySearch`]'s field of the same name.
        appearance: pdfcer_core::annot_author::RedactAppearance,
    },
    /// ★★★ **Mark what is SELECTED on the page for redaction** — the third
    /// marking route, and the first that does not go through text.
    ///
    /// **Ken, 2026-08-30:** *"am I able to select objects on the canvas and
    /// redact them that way yet? … it just told me it couldn't."* It could not:
    /// [`Self::MarkRedactionsBySearch`] reaches text pdfcer can read as text and
    /// [`Self::MarkPageForRedaction`] reaches everything, and on a CAD drawing
    /// most of what wants redacting is in between.
    ///
    /// `super::redactsel`'s header carries the argument in full, including why
    /// neither a page nor a rectangle is carried here.
    MarkSelectionForRedaction {
        /// How the mark will look once applied. See
        /// [`Self::MarkRedactionsBySearch`]'s field of the same name.
        appearance: pdfcer_core::annot_author::RedactAppearance,
    },
    /// **Take one redaction mark off.**
    ///
    /// Raised by a row's Remove control. The engine's
    /// `EditSession::delete_redaction_mark` rather than its general annotation
    /// delete, deliberately and on core's own instruction: the two record
    /// different `CommandKind`s so that an undo tooltip can say *"remove a
    /// redaction mark"* rather than *"delete annotation"*, and — as that
    /// method's docs put it — *"I decided not to redact that"* is a different
    /// claim from *"delete annotation"*.
    ///
    /// The **annotation id**, not a row index: a list position is a position in
    /// a census rebuilt every frame, and by the time the apply phase runs the
    /// same index may name a different mark. `crate::app` §10's rule —
    /// *selection is an identity, not a position* — applied to a list.
    RemoveRedactionMark {
        /// The `/Redact` annotation to delete.
        annot_id: pdfcer_core::object::ObjId,
    },
    /// ★★★ **Apply every redaction mark INTO the open document, writing
    /// nothing** — `OPERATOR_REQUESTS.md` O125, 2026-09-04. **The whole
    /// argument — no fields, irreversible, undo cleared — is on the apply arm**
    /// in `app::actions::redact`, on this file's own R2 rule.
    ApplyRedactionsIntoDocument,
    /// **Select everything on the current page**, including anything that
    /// has been moved OFF it.
    ///
    /// ★★★ The recovery route for a one-way door. 2026-09-01: *"I sometimes
    /// drop objects there, and when I do I can't get them back."* The canvas
    /// senses input over the page rect only — correctly, or a hit area would
    /// overlap its neighbours in a continuous strip — so an object dragged
    /// past the edge is unclickable, unbandable and unpainted, while still
    /// being in the file.
    ///
    /// ★ **The whole argument, including why an infinite rect is the right
    /// way to ask and what this deliberately does NOT fix, is on the apply
    /// arm** in `app::actions::apply` — this file is 1,500 lines of one
    /// enum and R2 puts the reasoning next to the mechanism when it cannot
    /// have both.
    /// **Recolour selected paths** (`pdfcer-core` `Pass 218.0`/`219.0`).
    ///
    /// `OPERATOR_REQUESTS.md` O89's vector half. `None` on a channel leaves it
    /// alone; the two are independent, and recolouring the fill of an object
    /// whose stroke is a spot ink is not blocked by the channel nobody touched.
    ///
    /// ★ The full argument — including why an undecodable ink gets no swatch
    /// rather than a black one — is on `panels::properties::paint`, where the
    /// control is.
    SetObjectPaint {
        /// The page the objects are on.
        page: usize,
        /// Page-object indices. Leaves are not operands: a paint-order verb
        /// writes to the page's content stream and a leaf's span indexes the
        /// form's.
        objects: Vec<usize>,
        /// The new fill, or `None` to leave it.
        fill: Option<[u8; 3]>,
        /// The new stroke, or `None` to leave it.
        stroke: Option<[u8; 3]>,
    },
    /// **Go to a bookmark's destination** — the position half of `/XYZ`,
    /// `/FitH` and `/FitV`, and the whole of `/FitR`.
    ///
    /// ★ ONE variant for both shapes, carrying the type `canvas::destination`
    /// already defines. Two actions would have been two spellings of one
    /// concept, and the module that owns the subject owns the vocabulary.
    GoToDestination(crate::canvas::destination::PendingDestination),
    SelectAllOnPage,
    Undo,
    /// **Re-apply the most recently undone change.**
    ///
    /// Raised by `edit.redo`, bound to both `Ctrl+Y` and `Ctrl+Shift+Z`, and
    /// applied by the same `apply::history_step` — one function, one direction
    /// parameter, because the two differ in exactly which engine verb they call
    /// and in nothing else. Two arms would be two copies of the guard, the
    /// trace and the decline, and one of them would eventually acquire a step
    /// the other did not.
    ///
    /// Everything in [`Self::Undo`]'s docs applies unchanged. The one fact
    /// worth stating separately is the redo stack's own lifetime: the engine
    /// clears it whenever a new command is recorded (`EditSession::commit` —
    /// *"the redone future no longer exists once history diverges"*), so a
    /// redo that was available before an edit is not available after it, and
    /// the condition follows on the next frame with nothing here to remember.
    Redo,
    /// **Invoke a registered command by id**, from a surface that is not the
    /// ribbon.
    ///
    /// ★ The one variant that is not a statement about the document. It exists
    /// so a *second route to an existing command* cannot become a second
    /// implementation of it: the Find bar's OCR offer means exactly what
    /// `file.ocr` on the ribbon means, and wiring it straight to
    /// `DialogsState::open_ocr` would have put that command's guards in two
    /// places — the failure `crate::app`'s one-choke-point invariant names.
    ///
    /// **Drained during the frame, never in [`PdfcerApp::apply_actions`]**, for
    /// two reasons that are hard rather than stylistic: `dispatch_command`
    /// needs an `&egui::Context` and the apply phase is deliberately given
    /// none, and a dialog it opens must be drawn by `DialogsState::show` on the
    /// same frame. The drain and its full argument are at the call site in
    /// `crate::app`; the arm below exists only to notice if it is ever removed.
    Command(String),
}
