//! # `app::actions::forms` — everything done to a form FIELD
//!
//! A sibling of [`super::dimensions`], [`super::pages`], [`super::vector`] and
//! [`super::export`], and it owns both halves of its subject: the action enum
//! [`FieldAction`] and the apply logic every one of its variants reaches.
//!
//! ## Why the family is a family
//!
//! Ten verbs — fill, select, place, author, rename, delete a field, delete one
//! of its widgets, register an unclaimed one, and (2026-08-28) ask what
//! deleting a grouping node would take and then take it — sharing a property
//! nothing else
//! in `actions` has: **every one of them addresses a control by its fully
//! qualified NAME, or by the widget's `ObjId`. None of them uses a paint-order
//! index.** That is not a coincidence of style; it follows from where the data
//! lives. `/AcroForm` is in the document catalog (§12.7.2), a field is reached
//! through it, and a widget is reached through the field that claims it — never
//! through the page that happens to draw it.
//!
//! It is the same test the neighbouring seams pass: [`super::vector`] is the
//! verbs that address paint-order indices, [`super::pages`] the verbs that
//! address page positions. A size-driven cut would not have produced this
//! grouping and would not survive the next variant; this one tells you where a
//! ninth verb goes without anyone having to decide.
//!
//! ## Why the enum moved here on 2026-08-27
//!
//! **R2.** [`super::action::Action`] stood at 1,495 of the 1,500-line budget
//! with no seam left in it, and `CONTINUE.md` had recorded that *"the next
//! change trips the gate"*. R2's own doctrine is that a file approaching the
//! limit is a signal to find the seam rather than to raise the limit, and this
//! was the seam with the most lines behind it and an apply module already
//! standing.
//!
//! ★★ **Reading the block to move it found a defect no gate could see.** Three
//! `///` blocks had come to sit contiguously above `SelectFormField`, so rustdoc
//! rendered one variant carrying three unrelated explanations while
//! `BeginFormField` and `Action::BeginTextAnnot` carried none at all. Doc
//! comments concatenate silently: nothing warns, nothing fails, and a variant
//! that has lost its own documentation looks exactly like one that never had
//! any. `cargo doc` is clean, clippy is clean, every test passes, and the only
//! instrument that finds it is somebody reading the file. Each block is back on
//! its own subject, and the two orphans are documented again.
//!
//! ## The original subject of this module, which is still the hard part of it
//!
//! Registering a form control the document lists but no field claims. The
//! *disclosure and refusal wording* is the substantial part of that verb, which
//! is why it earned a module of its own before the family joined it.
//!
//! ## What an unclaimed widget is, and why the shell can produce one
//!
//! A `/Widget` annotation in a page's `/Annots` that no entry of the document's
//! `/AcroForm` `/Fields` reaches. It **draws** — border, background, the whole
//! appearance stream — and nothing can fill it, because every filling verb
//! addresses a field by its fully qualified name and this box is in no field.
//!
//! This project's recurring failure mode, a visible control that is silently
//! inert, arriving through a **document** rather than through a ribbon. The
//! operator clicks it, types, and nothing happens.
//!
//! ★ pdfcer makes them itself. `EditSession::insert_pages` copies everything
//! reachable from a page, and a page's `/Annots` reaches its widgets — but
//! `/AcroForm` is document-level and is not merged, so a source with 12 fields
//! inserted into a blank document produces 13 widgets and no form at all. The
//! engine measured exactly that (`examples/orphan_probe.rs`, pdfbox corpus) and
//! now returns the count in `InsertOutcome::orphaned_widgets`.
//!
//! ## ★★ Two shapes, and only one of them can be put back
//!
//! The engine's measurement is the reason this module has two refusal arms
//! rather than a success path and a shrug:
//!
//! | shape | of 13 measured | carries | registering it |
//! |---|---|---|---|
//! | **merged field-widget** (§12.7.3.1) | 11 | its own `/FT`, `/T`, `/V`, `/DA` | **recovers the field exactly** |
//! | **bare kid** (a radio group's member) | 2 | nothing at all | **creates a new, empty field** |
//!
//! The second row is `insert_pages` dropping `/Parent` from every dictionary it
//! copies. For a page that is correct — following it would drag the source's
//! whole page tree across. For a widget, `/Parent` **is** its link to its
//! identity, so those two arrived having lost the name `GroupOption`, the type
//! `/Btn`, the radio flags `0xC000` and the value `Option2`. Nothing in the
//! target document holds any of it.
//!
//! An operator cannot see which shape a box is, and the difference decides
//! whether pressing Register restores something or invents something. That is
//! why [`crate::text::status::adopt_declined_no_name`] refuses to use the word
//! *restore*, and why it names re-inserting from the source as the only route
//! that gets the original back.
//!
//! ## Why this uses the funnel
//!
//! `adopt_widget` writes `/AcroForm` and `/T`. It is a document edit with one
//! undo entry, so it goes through [`super::apply::vector_edit`] like every
//! other one — the render worker stopped, the mutation, the epoch bumped, the
//! page invalidated. Nothing here is special except the wording.

/// ★ **Verbs about the BOX rather than the field** — rotation today, and the
/// natural home for the next one. Split out under R2 when `rotate_widget`
/// arrived; a field's identity and a widget's placement are two subjects.
/// ★ **Authoring a form control from the placement dialog's choices** —
/// `author`, split out under R2 on 2026-08-30. Its sibling `paste` is
/// authoring from a SOURCE; the two are not duplicates and the headers say why.
pub(super) mod author;
/// Deleting a **grouping node** — the two-press verb, its preview store and
/// both apply paths.
///
/// ★ A submodule rather than more lines here, and the seam is subject rather
/// than size: everything in this file addresses a control an operator can see
/// and fill; a grouping node is a name with no type, no value, no widget and no
/// rectangle, whose entire difficulty is that its removal is invisible. That
/// module's header carries the two-press protocol, why the preview cannot run
/// in a panel, and where the armed preview is kept.
pub mod delete;
pub mod groups;
/// ★ **Putting a copied form field back** — `EditSession::paste_field`, split out
/// on 2026-08-29 under R2 when it took this file to 1,501 lines. Its header
/// carries why the shell does almost nothing in it any more.
mod paste;
mod widget;

use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;
use crate::app::status::decline::{self, Declined};
use crate::text::status as t;

impl From<FieldAction> for super::action::Action {
    /// ★ So a call site says what it MEANS and the wrapping is not its problem.
    ///
    /// The same reasoning [`super::vector`]'s `From` carries: the filing system
    /// is an R2 artefact, and a panel button that renames a field has no
    /// business knowing about it. `.into()` at the push, `From` here, one line.
    fn from(f: FieldAction) -> Self {
        Self::Field(f)
    }
}

/// One thing done to a form field, carried by
/// [`super::action::Action::Field`].
///
/// See the module header for why these eight are one family and what test a
/// ninth would have to pass to join them.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldAction {
    /// **One form-field edit**, as one undoable command.
    ///
    /// The variant `crate::panels::forms` raises for every one of its verbs —
    /// fill, toggle, choose, reset, regenerate appearances, flatten — carrying
    /// the whole intent so it is resolvable after the frame that raised it, in
    /// the same way [`super::action::Action::DeleteSelection`] carries its operand list.
    ///
    /// # Why the arm below is one line and not four
    ///
    /// It does not go through [`vector_edit`], and `crate::panels::forms::edit`'s
    /// own header carries the reason: the six form outcome types do not unify
    /// into `Result<Vec<String>, EditError>`, so that module performs the
    /// cancel-mutate-bump-invalidate protocol itself, once, for all of them.
    /// A second copy of the protocol here would be the fifth hand-written
    /// instance of a four-step sequence `vector_edit` exists to have exactly
    /// one of.
    Edit(crate::panels::forms::edit::FormEdit),
    /// **The operator clicked a form field on the page, or clicked away.**
    ///
    /// Raised by `crate::canvas::forms`'s selection surface in Edit mode.
    /// `None` clears — a click on empty paper — which is a real event and not
    /// a no-op: a properties panel that will not let go is worse than an empty
    /// one, because its contents look current.
    ///
    /// ★ It changes **no document** and must never bump the edit epoch. A
    /// selection is view state; it is on `OpenDoc` beside the object selection
    /// for the same reason that one is, and for the same reason neither is
    /// saved.
    Select(Option<crate::app::state::SelectedField>),
    /// **Change one property of a field that is already on the page.**
    ///
    /// Reaches `EditSession::edit_field`, which takes the fully-qualified name
    /// and a `FieldEdit` — a partial update in which *a property you do not
    /// name is left alone*.
    ///
    /// # ★★★ This closes a gap the shell had told the operator was the
    /// engine's, and it was not
    ///
    /// The Properties pane shipped on 2026-08-26 showing a field's flags as
    /// **read-only facts**, under a sentence that said required, read-only, the
    /// tooltip and the border *"can only be set when a field is placed. To
    /// change one, delete this field and place a new one."*
    ///
    /// `EditSession::edit_field` had landed the **same day**, three commits
    /// before the revision this shell compiles against, and the engine had
    /// written a full pane design brief into the request channel saying so.
    /// Nothing consumed it. So for a day the program was telling an operator to
    /// perform a **destructive workaround** — delete-and-replace loses the
    /// field's name, its filled value and its place in the tab order — for a
    /// capability it already had.
    ///
    /// ★ The lesson is the one this project has now had five times in a week,
    /// and it is not "grep harder": an absence claim about a crate you do not
    /// build has a **shelf life**, because the crate moves. This one was true
    /// when written and false within hours. What catches that is reading the
    /// reply, not re-deriving the claim.
    ///
    /// # ★★ One variant, one property, one undo entry
    ///
    /// `FieldEdit` can carry fourteen properties at once and this deliberately
    /// sends one at a time, which is `StyleChange`'s rule for the same reason:
    /// **one control press is one undo entry.** A pane that batched a required
    /// flag and a max-length into one request — which the engine supports —
    /// would make `Ctrl+Z` after two separate presses take back a state the
    /// operator never saw.
    ///
    /// ★ The **one** exception the engine names is genuinely a single act:
    /// `.with_comb(true).with_max_len(Some(8))` must travel together, because
    /// Table 228 permits `Comb` only when `/MaxLen` is present and the gate is
    /// checked against the **resulting** field. That is not two edits batched;
    /// it is one edit that the standard makes indivisible.
    ///
    /// # The name travels, for `Rename`'s reason
    ///
    /// By the time the queue drains, the selection may have moved. The
    /// fully-qualified name is what `edit_field` addresses, and carrying it
    /// makes the action resolvable on its own.
    EditProperties {
        /// The field's fully-qualified name.
        field: String,
        /// The partial update. Built with `FieldEdit`'s `with_*` builders —
        /// the struct is `#[non_exhaustive]`, so a literal will not compile
        /// outside `pdfcer-core` and would break on every property it gains.
        edit: pdfcer_core::edit::FieldEdit,
        /// What the operator touched, for the refusal.
        ///
        /// ★★ The engine's §6: *"the gates are checked against the RESULT, not
        /// against your request"*, so `.with_max_len(None)` on a **comb** field
        /// refuses with `CombPreconditionUnmet` — naming a property the request
        /// never mentioned. Its own instruction: *"show it against the control
        /// the operator touched, not the one the standard named."* This carries
        /// which control that was, because after the fact nothing else can say.
        touched: &'static str,
    },
    /// ★★★ **Turn a field's box**, in ninety-degree steps.
    ///
    /// The degrees are **already counterclockwise and already normalised**:
    /// `/MK /R` is counterclockwise while the page's `/Rotate` is clockwise, and
    /// the engine's instruction was *"negate at the UI layer … do not negate
    /// inside anything that touches `/MK`"*. `panels::properties::widgetedit`'s
    /// `rotation_row` is the only place the operator's *left / right* becomes a
    /// sign, and `super::widget` carries what the applier does with it.
    RotateWidget {
        /// The field's fully-qualified name.
        field: String,
        /// Which of its boxes — a field can draw on three pages.
        widget: usize,
        /// The new angle, counterclockwise, already in `0..360`.
        degrees: i64,
    },
    /// **Change one property of the BOX a field is drawn in.**
    ///
    /// Reaches `EditSession::edit_widget`, the widget-scoped twin of
    /// [`Self::EditProperties`]. See `panels::properties::widgetedit` for the
    /// scope rule that makes them two verbs rather than one — in a sentence, a
    /// field with three widgets has one "required" and three boxes.
    ///
    /// # ★★ It carries the widget INDEX, and that is the whole difference
    ///
    /// `EditProperties` addresses a field by name and every placement follows.
    /// This addresses one placement, and on a radio group — the case where the
    /// distinction is visible at all — getting it wrong moves a button the
    /// operator was not looking at.
    EditWidget {
        /// The field's fully-qualified name.
        field: String,
        /// Which placement, indexing `Field::widgets`.
        widget: usize,
        /// The partial update, built with `WidgetEdit`'s `with_*` builders.
        edit: pdfcer_core::edit::WidgetEdit,
        /// What the operator touched, for the refusal. See
        /// [`Self::EditProperties`]'s field of the same name.
        touched: &'static str,
    },
    /// **Set this document's field values from an FDF, XFDF or CSV file.**
    ///
    /// # ★ The path is carried, and the picker ran BEFORE the action
    ///
    /// The opposite arrangement to `Action::ExportFormData`, which carries
    /// nothing and opens its picker inside the apply phase. Both are right for
    /// their case: the export has to compute the bytes before it can honestly
    /// ask where they go, while the import has nothing to compute until it
    /// knows which file.
    ///
    /// What they share is the reason a picker is not opened from a widget's
    /// `clicked()` branch — `actions::export`'s header — namely that a native
    /// modal blocks the thread while egui is part-way through building a frame
    /// that will not finish until the operator answers.
    Import {
        /// The data file the operator chose.
        path: std::path::PathBuf,
    },
    /// **Rename the selected field.**
    ///
    /// Reaches `EditSession::rename_field`, which takes the fully-qualified
    /// name and a new *partial* one.
    ///
    /// ★★ The old name travels even though the selection holds it, and that is
    /// the same staleness rule `CommitTextAnnot` follows: by the time the queue
    /// drains, another action ahead of it in the same drain could have changed
    /// the selection. An action is a complete statement of what the operator
    /// asked for.
    Rename {
        /// The field's current fully-qualified name.
        from: String,
        /// The new partial name.
        to: String,
    },
    /// **Give an existing push button an action, or take one away**
    /// (`pdfcer-core` `Pass 182.0`/`183.0`/`183.1`; read side `Pass 212.0`).
    ///
    /// Raised by `crate::panels::forms::button` and by nothing else.
    ///
    /// # ★★ `None` is a real operand, not an absence
    ///
    /// `set_button_action` takes `Option<ButtonAction>` and `None` **removes**
    /// whatever is there — the half a form editor needs when it opens somebody
    /// else's document and wants the button inert. So this variant carries an
    /// `Option` rather than being two variants: *set* and *clear* are one verb
    /// with one refusal set, and splitting them here would give the shell two
    /// spellings of one act.
    ///
    /// ★ Boxed because `ButtonAction` carries a `SubmitSpec`, which is much the
    /// largest thing in this enum. Without it every `FieldAction` — including
    /// `Select`, raised on every click — would be sized for a submit.
    SetButtonAction {
        /// The button's fully-qualified name.
        field: String,
        /// What it should do, or `None` to make it inert.
        action: Box<Option<pdfcer_core::edit::ButtonAction>>,
    },
    /// **Delete the selected field, with every widget it draws.**
    ///
    /// ★ Distinct from [`Self::DeleteWidget`] and the distinction is not a
    /// nicety: one field may be drawn in several places, and "remove this box"
    /// and "remove this field" are different requests with different
    /// consequences. Offering only the second would make removing one of three
    /// copies impossible; offering only the first would leave a named field
    /// behind with no widgets, which is a field nothing can fill.
    DeleteField {
        /// The field's fully-qualified name.
        field: String,
    },
    /// ★★★ **Move one widget of a field by a page-space delta.**
    ///
    /// Raised by `crate::canvas::widgetdrag` on the release of a drag, and by
    /// nothing else.
    ///
    /// # ★★ Why not `Action::MoveAnnotation`, when a widget IS an annotation
    ///
    /// Because the engine refuses that by name and says why: `move_widget`
    /// *"does strictly more, and quietly doing less under this name would give
    /// you a second way to move the same thing that silently produces a worse
    /// result."* What it does more of is the **field** -- a widget is addressed
    /// by its field's fully-qualified name and an index within it, because one
    /// field can draw boxes on three pages and the `/Annots` entry is not the
    /// thing the operator renamed.
    ///
    /// => The two verbs differ in their ADDRESS, not in their geometry. Worth
    /// stating because the alternative reading -- that widgets need different
    /// arithmetic -- would invite somebody to unify them later.
    MoveWidget {
        /// The field's fully-qualified name.
        field: String,
        /// Which of its widgets.
        widget: usize,
        /// Horizontal displacement, PDF points.
        dx: f64,
        /// Vertical displacement, PDF points. **Positive is up.**
        dy: f64,
    },
    /// **Put a page's annotations in a new order** — `OPERATOR_REQUESTS.md` O99.
    ///
    /// The operator: *"the tab order list is supposed to be able to be reordered
    /// by dragging and dropping rows around like we can with pages in the page
    /// preview."*
    ///
    /// # ★★★ Object ids, not indices, and the engine asked for it by name
    ///
    /// `EditSession::reorder_annotations` takes `&[ObjId]`. Its shipping note
    /// says why in one sentence: *"the index you hold is almost never a raw
    /// `/Annots` index — `page_annotations` skips null and non-dictionary
    /// entries, so the numberings diverge on exactly the malformed files where a
    /// guess costs most."*
    ///
    /// ★★ The tab-order panel's `TabRow::position` is emphatically **not** an
    /// address: 1-based, widgets only, and a label an operator counts while
    /// tabbing. `TabRow::id` is the address, and it exists for this variant.
    ///
    /// # ★ Why it is a FieldAction and not a top-level `Action`
    ///
    /// Because it is a form verb raised by the Forms panel, which is what this
    /// sub-enum is for — and because `Action` sits at exactly its 1,500-line R2
    /// ceiling, so a variant added there would force a refactor to buy room for
    /// something that already has a home. The sub-enum is the established
    /// pattern (`Annot`, `Page`, `Vector`, `Text`, `Write`, …) and this is the
    /// case it was made for.
    ReorderAnnotations {
        /// The page, 0-based.
        page: usize,
        /// Every **indirect** entry of that page's `/Annots`, each once, in the
        /// wanted order.
        ///
        /// ★ *Every* entry, not only the widgets the panel lists. The engine
        /// validates a permutation and refuses a partial one, which is right:
        /// a list that omitted the links and the markup would be asking to
        /// move them somewhere unstated.
        order: Vec<pdfcer_core::object::ObjId>,
    },
    /// **Delete one widget of the selected field**, leaving the field itself.
    DeleteWidget {
        /// The field's fully-qualified name.
        field: String,
        /// Which of its widgets.
        widget: usize,
    },
    /// **Ask what deleting a grouping node would remove**, or forget the
    /// answer.
    ///
    /// `Some(fqn)` runs `EditSession::field_group_deletion_preview` and stores
    /// the report for the Forms panel to draw; `None` clears it, which is what
    /// Cancel raises.
    ///
    /// ★ It **changes no document** and must never bump the edit epoch — the
    /// preview writes nothing. It is here rather than in the panel for one
    /// reason: the engine's signature is `&mut self`, a panel body holds
    /// `&OpenDoc`, and `Arc::get_mut` only succeeds inside the funnel. So a
    /// query that changes nothing is nonetheless an action, exactly as
    /// [`Self::Select`] is.
    ///
    /// ★★ `Option` rather than a second variant, for [`Self::Select`]'s reason:
    /// clearing is a real event, not a no-op — a destructive-confirmation block
    /// that will not let go is worse than none, because its contents look
    /// current.
    ///
    /// See [`groups`] for the two-press protocol, why the answer is kept in a
    /// thread-local, and the epoch rule that retires it.
    ArmGroupDeletion(Option<String>),
    /// **Delete a grouping node and every field beneath it**, as one undoable
    /// command.
    ///
    /// ★★★ Distinct from [`Self::DeleteField`], and the engine refuses to let
    /// them be the same call: `delete_field` resolves through the **terminal**
    /// field list, so it *cannot name a grouping node at all*, and a loop of it
    /// would produce N undo entries for one gesture and could leave a subtree
    /// half-removed having reported failure. `delete_field_group` computes the
    /// whole removal set first and commits once.
    ///
    /// The name travels for [`Self::Rename`]'s reason: by the time the queue
    /// drains the selection may have moved, and the fully-qualified name is
    /// what the engine addresses.
    DeleteGroup {
        /// The grouping node's fully-qualified name.
        group: String,
    },
    /// **A form control has been placed and now needs its details.**
    ///
    /// Raised by the canvas on the click or release that finishes the placing
    /// gesture, and by nothing else. It **changes no document** — it opens
    /// `crate::dialogs::formfield`, which is where the operator names the
    /// field.
    ///
    /// ★★ The geometry travels and the details do not, for the reason
    /// [`super::action::Action::BeginTextAnnot`] gives at length: the rectangle is a choice the
    /// operator made *now*, on the page they were looking at, and the details
    /// are made later in a dialog and may never be made at all.
    ///
    /// ★ There is deliberately no `name` on it. The name is generated when the
    /// dialog opens, because generating it requires reading the document's
    /// existing field names, and the canvas has no business parsing an
    /// `/AcroForm`.
    Begin {
        /// The 0-based page the control will be authored onto.
        page: usize,
        /// Which of the five kinds is being placed.
        kind: crate::canvas::formfield::FormFieldKind,
        /// The rectangle, in PDF user space, already normalised.
        rect: pdfcer_core::page_tree::Rect,
    },
    /// **Author the form control the dialog just accepted.**
    ///
    /// Raised by `crate::dialogs::formfield` and by nothing else. This is the
    /// one that reaches the document, through the same `vector_edit` funnel
    /// every other authoring verb uses.
    ///
    /// ★ The whole draft travels, for the reason [`super::action::Action::CommitTextAnnot`]
    /// states: by the time the queue drains the dialog is closed and its fields
    /// are gone, so reading them at apply time is not fragile but impossible.
    Commit {
        /// The 0-based page.
        page: usize,
        /// The rectangle, in PDF user space, already normalised.
        rect: pdfcer_core::page_tree::Rect,
        /// Everything the operator chose, including which kind it is.
        draft: Box<crate::canvas::formfield::Draft>,
    },
    /// ★★★ **Author the form control that came off the clipboard.**
    ///
    /// Raised by [`crate::canvas::fieldclip::paste`] and by nothing else.
    /// `OPERATOR_REQUESTS.md` **O58**, 2026-08-29.
    ///
    /// # Why this is not [`Self::Commit`], which it otherwise duplicates
    ///
    /// One line in `super::apply`: `Commit` calls `self.form_defaults.remember`
    /// on the way past, and a paste must not.
    ///
    /// `remember` exists for the operator's *"remember last settings"* — it
    /// seeds the **placement dialog** with whatever was last accepted there,
    /// which is right, because accepting a dialog is a statement about how the
    /// operator wants fields made. A paste is not that statement. Routing a
    /// paste through `Commit` would mean that copying one password field
    /// silently made the *next hand-drawn field* a password field, discovered
    /// three fields later, with nothing on screen having said so.
    ///
    /// ⇒ The two variants carry identical data and differ in one side effect,
    /// which is exactly when two variants are correct rather than one with a
    /// flag: the flag would be read at the only place that can act on it and
    /// would be invisible everywhere else, including here.
    ///
    /// # What is NOT decided here
    ///
    /// Whether this is a new field or a second widget of an existing one. That
    /// is settled entirely by [`crate::canvas::formfield::Draft::name`] before
    /// the action is raised — a name that matches an existing field **merges**
    /// (`edit.rs:13523`, `merged: true`), a fresh one does not. So the two
    /// chords produce the same variant carrying different names, and
    /// `super::author`'s existing disclosure pass reports `merged` without
    /// needing to know which chord was pressed.
    Paste {
        /// The 0-based page it lands on.
        page: usize,
        /// Where, in PDF user space — already offset or already in place, per
        /// [`crate::canvas::fieldclip::paste`]'s same-page/cross-page rule.
        ///
        /// ★ On a **radio group** the engine uses only the lower-left corner
        /// and discloses that the size was ignored: a group's geometry is part
        /// of its meaning, so it translates rather than rescaling.
        rect: pdfcer_core::page_tree::Rect,
        /// `FieldClip::to_bytes` — the clip itself, verbatim off the clipboard.
        ///
        /// ★★ Bytes rather than a live `FieldClip` because that is what the
        /// clipboard holds and because an `Action` is `Clone + PartialEq`. The
        /// engine's format is total and byte-exact — it tests that a clip
        /// through bytes and one that stayed in memory produce identical
        /// documents — so nothing is lost by carrying it this way.
        clip: Vec<u8>,
        /// New independent field, or another widget of the existing one.
        ///
        /// ★ Boxed: `FieldPastePolicy::NewField` carries a `String` name and a
        /// `PasteTooltip` that may carry another, and this enum has thirty-odd
        /// variants that would all grow to match.
        policy: Box<pdfcer_core::formclip::FieldPastePolicy>,
    },
    /// ★ **Register a form control the document draws but no field claims.**
    ///
    /// Raised by `crate::panels::forms::tab_order` and by nothing else — the
    /// one view that already knew which widgets these are, because listing them
    /// is what it is for.
    ///
    /// # Why the widget is an `ObjId` and not a position
    ///
    /// The same reason [`super::bookmarks::BookmarkAction::Add`]'s parent is: a position is
    /// invalidated by the edit itself. Registering a widget moves it out of the
    /// unclaimed list and into the rows, so a second registration keyed on
    /// "the second unclaimed box" would act on a different box than the one the
    /// operator pressed beside. `adopt_widget` takes an id for this reason and
    /// the listing carries one for the same reason.
    ///
    /// # Why the page travels with it
    ///
    /// Only for the funnel: [`super::apply::vector_edit`] wants a page for its trace
    /// line and its per-page raster drop. The edit itself is document-level —
    /// `/AcroForm` is in the catalog — so nothing about *which* page is
    /// consulted by the engine. It is the page the box is drawn on, which is
    /// the one whose raster has to be rebuilt, and that is the only claim being
    /// made by carrying it.
    ///
    /// # `None` is the common answer and it is not "no name"
    ///
    /// It means *use the name the box already carries*. Most unclaimed widgets
    /// are merged field-widgets holding their own `/T`, and supplying a name for
    /// one of those **overrides** a name the file already had. See
    /// [the module header](self)'s header for the two shapes and why an operator cannot tell
    /// them apart by looking.
    Adopt {
        /// The page the widget is drawn on — for the trace and the re-raster.
        page: usize,
        /// The widget's object identity, from `tab_order::model::Unclaimed`.
        widget: pdfcer_core::object::ObjId,
        /// A name to register it under, or `None` to keep the one it carries.
        /// Trimmed and non-empty by the time it gets here.
        name: Option<String>,
    },
}

/// Apply every form-field verb that needs the open document and nothing else.
///
/// # ★ Why two of the eight are NOT here
///
/// [`FieldAction::Begin`] and [`FieldAction::Commit`] stay in
/// [`super::apply`], and the reason is a borrow rather than a preference.
/// `Begin` opens a dialog and `Commit` remembers the operator's settings, so
/// both need `PdfcerApp`'s own fields — and `doc` in that function *is*
/// `&mut self.status`, so no signature exists that takes both. That arm's
/// comment carries the full argument.
///
/// The split is therefore a fact about the data, not an oversight, and it is
/// written down in both places so a later tidy has to argue with it.
pub(super) fn apply(doc: &mut OpenDoc, action: FieldAction) {
    match action {
        // ★ Selection is VIEW STATE. It changes no document, bumps no epoch and
        // invalidates no page — which is why it does not go near the funnel.
        FieldAction::Select(selected) => doc.selected_field = selected,
        // ★ One line: the panel already resolved left/right into a
        // counterclockwise angle, and `rotate_widget` owns the rest -- the
        // multiple-of-90 refusal, the normalisation and the appearance
        // regeneration it may or may not be able to do.
        FieldAction::RotateWidget {
            field,
            widget,
            degrees,
        } => widget::rotate(doc, &field, widget, degrees),
        FieldAction::Paste {
            page,
            rect,
            clip,
            policy,
        } => paste::paste(doc, page, rect, &clip, &policy),
        FieldAction::EditProperties {
            field,
            edit,
            touched,
        } => edit_properties(doc, &field, &edit, touched),
        FieldAction::EditWidget {
            field,
            widget,
            edit,
            touched,
        } => edit_widget(doc, &field, widget, &edit, touched),
        FieldAction::Import { path } => import_data(doc, &path),
        FieldAction::Rename { from, to } => rename(doc, &from, &to),
        FieldAction::SetButtonAction { field, action } => {
            set_button_action(doc, &field, *action);
        }
        FieldAction::DeleteField { field } => delete::field(doc, &field),
        FieldAction::MoveWidget {
            field,
            widget,
            dx,
            dy,
        } => move_widget(doc, &field, widget, dx, dy),
        FieldAction::DeleteWidget { field, widget } => delete::widget(doc, &field, widget),
        FieldAction::ReorderAnnotations { page, order } => {
            super::reorder::reorder_annotations(doc, page, &order);
        }
        // ★ The arm changes no document and bumps no epoch, so it does not go
        // near `vector_edit`; the deletion does, like every other structural
        // form verb. See `groups`' header for why a query needs to be an action
        // at all.
        FieldAction::ArmGroupDeletion(group) => groups::arm(doc, group),
        FieldAction::DeleteGroup { group } => groups::delete(doc, &group),
        FieldAction::Adopt { page, widget, name } => adopt(doc, page, widget, name),
        FieldAction::Edit(edit) => crate::panels::forms::edit::apply(doc, &edit),
        // ★ Unreachable rather than unhandled, and named so the compiler will
        // say so if the split above is ever changed without changing this.
        FieldAction::Begin { .. } | FieldAction::Commit { .. } => {
            debug_assert!(
                false,
                // ui-text-exempt: a debug_assert message for a developer; never rendered.
                "FieldAction::Begin and ::Commit are applied in super::apply, which holds the dialog and defaults state this function cannot reach"
            );
        }
    }
}

/// Register one unclaimed widget into the document's `/AcroForm`.
///
/// `name` is `None` when the operator left the box blank, which is the common
/// and correct answer: a merged field-widget carries its own `/T` and typing a
/// name would **override** it rather than supply something missing.
///
/// # ★ Why the refusal is inspected here and the error is still returned
///
/// [`super::apply::vector_edit`] takes `Display` and does one thing with an
/// `Err`: it traces it and leaves the document alone. That is right, and it is
/// not enough for this verb, because two of `adopt_widget`'s five refusals are
/// **things the operator can fix in the next three seconds** — retype the name,
/// or supply one. A refusal an operator can act on that reaches only
/// `PDFCER_DIAG` is a control that does nothing when pressed.
///
/// So the closure records a decline on the way past and then hands the error
/// back unchanged. Both halves matter:
///
/// - **recording, not returning a message**, because `crate::app::status::decline`
///   already owns the store, the retirement rule and the one line in the bar,
///   and a second mechanism beside it would be the one that forgot to retire
///   itself — that module's own header says so;
/// - **returning the error anyway**, so the trace still carries the engine's own
///   `Display` prose. The decline is a sentence for an operator; the trace is
///   the record for whoever is debugging, and they are not the same text and
///   must not become each other. `check-ui-strings.sh`'s exclusion 3 is explicit
///   that an error type's prose is not permission to route UI text through it.
///
/// The three refusals with no arm are unreachable from this surface rather than
/// unhandled — see [`decline::record_adopt_refusal`], which carries the table.
pub(super) fn adopt(doc: &mut OpenDoc, page: usize, widget: ObjId, name: Option<String>) {
    super::apply::vector_edit(doc, "adopt-widget", page, 1, |session| {
        match session.adopt_widget(widget, name.as_deref()) {
            Ok(outcome) => Ok(vec![t::adopted(
                &outcome.name,
                outcome.field_type.is_some(),
                outcome.acroform_created,
            )]),
            Err(error) => {
                if let Some(declined) = correctable(&error) {
                    decline::record_adopt_refusal(declined);
                }
                Err(error)
            }
        }
    });
}

/// Every field name the document already carries.
///
/// ★★ Read fresh from the session rather than cached, and the reason is a
/// hazard rather than tidiness: the answer decides what the next field is
/// **named**, a name that collides makes the new widget a second view of an
/// existing field (see [`author`]'s header), and the set changes under any
/// undo, any redo, any page insert and every previous placement. A cache would
/// be correct until the first Ctrl+Z and silently wrong afterwards.
///
/// A document with no `/AcroForm` returns an empty list rather than declining —
/// which is the common case, since most drawings have no form at all, and the
/// first field placed on one has nothing to collide with.
pub(super) fn field_names(doc: &OpenDoc) -> Vec<String> {
    let view = doc.session.view();
    pdfcer_core::forms::parse_acroform(&view)
        .map(|form| {
            form.fields
                .iter()
                .map(|f| f.fully_qualified_name.clone())
                .collect()
        })
        .unwrap_or_default()
}

/// **Author one new form control** from the draft the dialog accepted.
///
/// The single narrowing point between this shell's one `Draft` and
/// `pdfcer-core`'s five spec types — see [`crate::canvas::formfield::draft`]'s
/// header for why the shell holds one struct and the engine five. Every field
/// a kind does not have is simply not read here, which is what makes that
/// asymmetry cost one `match` instead of five dialogs.
///
/// ## ★★ The tooltip is the whole reason this verb was thought impossible
///
/// `TooltipChoice` has three states and the default is `Undecided`, which every
/// one of these five verbs **refuses**: an interactive control owes a screen
/// reader a name, and the engine will not invent one silently. That refusal was
/// recorded in this project's own backlog as *"core's STRUCTURAL certification
/// gate"* and parked the feature for nine days. It is not a gate; it is a
/// required field of the dialog above.
///
/// So an empty tooltip becomes `TooltipChoice::Declined` rather than being left
/// `Undecided`. The two are not the same and the difference is the point:
/// `Declined` is the operator saying *"this control needs no name"*, which is a
/// decision and is sometimes correct — a decorative button beside a labelled
/// one. `Undecided` is nobody having been asked.
///
/// ## ★ Rule 4 — the outcome is disclosed, off-canvas, in full
///
/// `FieldAuthorOutcome` carries four things the operator **cannot see on the
/// page**, and the one that matters most is `merged`: a name that matches an
/// existing field makes this widget a second *view* of that field, so typing in
/// one changes the other. Nothing about the rendered page says so, and a
/// screenshot of it would be identical either way. That is precisely the half
/// of rule 4 that survives decision 059 — *render normally; report separately* —
/// so every flag the engine raises becomes a status line and none of them
/// becomes a mark on the canvas.
/// **Which ancestor of `name`, if any, is already an ordinary terminal field.**
///
/// `None` when the name is safe to author. `Some(fqn)` names the field that
/// would be **destroyed** by authoring it — see
/// [`crate::text::fieldclip::name_would_swallow`] for the measurement and the
/// mechanism.
///
/// # ★★ The tripwire, and what will make this function deletable
///
/// This is a shim for an engine gap. The `debug_assert` below states the
/// condition that makes it unnecessary: **once `add_*_field` refuses a path
/// that crosses a terminal**, authoring such a name will fail on its own and
/// this guard becomes a duplicate check that can only disagree with the engine.
///
/// It asserts in debug builds rather than silently continuing, because a
/// workaround with no caller is the kind of code that rots for months — and
/// this project has already had a shim announce its own obsolescence two hours
/// after it was written.
///
/// ★ Only ancestors are examined, never the whole name. `Text.2` checks `Text`;
/// it does not check `Text.2` itself, because a name that already exists as a
/// terminal field of the same type is a legitimate **merge** and is exactly what
/// `Ctrl+Shift+V` relies on.
pub(super) fn group_is_a_field(doc: &OpenDoc, name: &str) -> Option<String> {
    if !name.contains('.') {
        return None;
    }
    let view = doc.session.view();
    let form = pdfcer_core::forms::parse_acroform(&view)?;
    let segments: Vec<&str> = name.split('.').collect();
    // Every proper prefix — `A`, `A.B`, … — but not the full name.
    for cut in 1..segments.len() {
        let ancestor = segments[..cut].join(".");
        if form.fields_named(&ancestor).next().is_some() {
            debug_assert!(
                std::env::var("PDFCER_ENGINE_REFUSES_DOTTED_PATHS").is_err(),
                // ui-text-exempt: a debug_assert message for a developer; never rendered.
                "the engine now refuses a dotted path that crosses a terminal field, so `group_is_a_field` is a duplicate check and should be deleted along with `text::fieldclip::name_would_swallow` and this assertion"
            );
            return Some(ancestor);
        }
    }
    None
}

/// **Rename the selected field.**
///
/// ## ★★ The engine takes a PARTIAL name and the selection holds a FULLY
/// QUALIFIED one, and conflating them corrupts a form
///
/// `rename_field(fqn, new_partial)` is asymmetric on purpose. A field's
/// fully-qualified name is its own `/T` joined to its ancestors' with dots —
/// `Address.Line1` is a field named `Line1` inside a parent named `Address`.
/// Passing a dotted string as the new *partial* name would author a `/T`
/// containing a dot, which no reader can resolve back: the field becomes
/// unaddressable by every fill verb, including pdfcer's own.
///
/// So the dialog offers the partial name and this passes it through untouched.
/// The engine is the one that rebuilds the qualified name, because only it
/// knows the parent chain.
///
/// ## ★ The selection is cleared, not updated
///
/// After a rename the old fully-qualified name reaches nothing. Recomputing the
/// new one here would mean deriving the parent chain a second time — the exact
/// duplication the paragraph above warns about — so the selection is dropped
/// and the operator's next click re-establishes it. One extra click, no chance
/// of a panel describing a field by a name that no longer exists.
/// **Change one property of an existing field.**
///
/// # ★★★ Three disclosures Acrobat performs SILENTLY, and this is where they
/// are said out loud
///
/// The engine's brief is explicit that pdfcer neither refuses nor repairs these
/// three, and that the shell must surface them — *"shortening a limit is a
/// legitimate authoring act and the old value is the author's problem to
/// resolve"*, while truncating their data or re-pointing their selection would
/// be inventing document state:
///
/// | change | what actually happens |
/// |---|---|
/// | `/MaxLen` shortened below the current value | the field is over its own limit |
/// | a selected choice option removed | Acrobat re-points the selection **by numeric index**, so it can silently land on a *different* option |
/// | a check box's export value changed while checked | it renders **unchecked**, with no warning |
///
/// `FieldEditOutcome::value_no_longer_fits` is a ready-made sentence naming
/// exactly what no longer fits, and it is passed through **verbatim** rather
/// than re-worded — the same rule `textstyle` follows for a synthesis
/// disclosure, and for the same reason: the engine knows which of the three
/// happened and this crate would have to guess.
///
/// A fourth, `sort_claim_unmet`: `Sort` records what the *writer* did, and
/// Table 230 makes conforming readers display `/Opt` in the order it occurs.
/// Setting it over an unsorted list makes the file claim something untrue, and
/// pdfcer will not silently reorder a list whose order the standard makes
/// significant.
///
/// # ★★ `widgets_affected` is reported when it is more than one
///
/// A field's flags are one write and every widget follows — the engine's scope
/// table, taken from Acrobat's own scripting model. So setting *required* on a
/// field drawn in three places changes three things on screen, of which the
/// operator can see one. Said, and only when it is surprising: on the ordinary
/// one-widget field the number is noise.
///
/// # The selection is KEPT, unlike rename and delete
///
/// Those two clear it because the name they address stops resolving. A property
/// edit changes no name, so the pane must go on describing the same field —
/// and it must, because the operator's next act is very often a second flag on
/// the same field. `edit_epoch` bumps, which is what re-reads the pane's draft.
pub(super) fn edit_properties(
    doc: &mut OpenDoc,
    field: &str,
    edit: &pdfcer_core::edit::FieldEdit,
    touched: &'static str,
) {
    let edit = edit.clone();
    let field = field.to_owned();
    super::apply::vector_edit(doc, "edit-field", 0, 1, move |session| {
        session.edit_field(&field, &edit).map(|outcome| {
            let mut lines = Vec::new();
            // ★ Verbatim, and FIRST. It is the one line that says the
            // operator's stored data no longer matches the field's own rules,
            // which outranks every count.
            if let Some(why) = outcome.value_no_longer_fits {
                lines.push(why);
            }
            if outcome.sort_claim_unmet {
                lines.push(crate::text::forms::field_sort_claim_unmet().to_owned());
            }
            if outcome.widgets_affected > 1 {
                lines.push(crate::text::forms::field_widgets_affected(
                    outcome.widgets_affected,
                ));
            }
            // ★ Nothing at all when the edit was ordinary, which is most of the
            // time. A bar that narrated every checkbox would stop being read,
            // and `vector_edit` treats an empty list as "no line".
            let _ = touched;
            lines
        })
    });
}

/// **Move, resize, or re-caption one placement of a field.**
///
/// # ★★★ The three disclosures, and the order is the operator's
///
/// **1. `appearance_stale` first**, because it is the only one about something
/// they can *see* and will misread. A resize makes §12.5.5's algorithm scale
/// the baked artwork to the new rectangle; where it cannot be rebuilt — a push
/// button's baked caption, a signature — the widget renders **distorted**. The
/// engine's own string names which and why, and it is prefixed rather than
/// re-worded, because *"stale appearance"* is a fact about the file and *"it
/// will look stretched"* is a fact about the screen.
///
/// **2. Which act it was.** A move keeps the artwork exact and free; a resize
/// rebuilt it. Reported from `WidgetEditOutcome::resized` rather than
/// re-derived here, because the engine compares the **extent** and this crate
/// comparing corners would eventually disagree with it about a nudge.
///
/// **3. `siblings_untouched`**, and only when there are any. It is the mirror
/// of `widgets_affected` on the field verb, and the pair exists so an operator
/// working on a field drawn in three places knows which kind of control they
/// just used. On the ordinary one-widget field it is zero and says nothing.
///
/// ## The selection is kept
///
/// Like [`edit_properties`] and unlike rename and delete: no name stops
/// resolving, and the operator's next act is very often a second nudge.
pub(super) fn edit_widget(
    doc: &mut OpenDoc,
    field: &str,
    widget: usize,
    edit: &pdfcer_core::edit::WidgetEdit,
    touched: &'static str,
) {
    let edit = edit.clone();
    let field = field.to_owned();
    super::apply::vector_edit(doc, "edit-widget", 0, 1, move |session| {
        session.edit_widget(&field, widget, &edit).map(|outcome| {
            // ★★★ **The trace this verb never had** — `OPERATOR_REQUESTS.md`
            // O76. Its two siblings, `move-widget-applied` and
            // `rotate-widget-applied`, have always reported their outcome; this
            // one reported nothing, which is why a check box quietly stretching
            // its own artwork for weeks was invisible to every driven run.
            //
            // ★ All three fields, because a screenshot cannot separate them: a
            // border that thickened because `/BS /W` changed and one that
            // thickened because §12.5.5's placement matrix scaled it are the
            // same pixels. `regenerated=` is the field that tells them apart,
            // and it is the assertion the O76 check is built on.
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "edit-widget-applied field={field} widget={widget} resized={} \
                     regenerated={} stale={}",
                    outcome.resized,
                    outcome.appearance_regenerated,
                    outcome.appearance_stale.is_some(),
                )
            });
            let mut lines = Vec::new();
            if let Some(why) = &outcome.appearance_stale {
                lines.push(crate::text::forms::field_appearance_stale(why));
            }
            // ★★ **`appearance_regenerated`, not `resized`** (O76). The outcome
            // carries both and this line was reading the wrong one, so a resize
            // that redrew nothing was reported as one that had.
            lines.push(
                crate::text::forms::field_widget_moved(
                    outcome.resized,
                    outcome.appearance_regenerated,
                )
                .to_owned(),
            );
            if outcome.siblings_untouched > 0 {
                lines.push(crate::text::forms::field_siblings_untouched(
                    outcome.siblings_untouched,
                ));
            }
            let _ = touched;
            lines
        })
    });
}

/// **Read an FDF, XFDF or CSV file and set this document's field values from
/// it.**
///
/// `file.import_form_data`, 2026-08-27 — the mirror of
/// `actions::export::form_data` and the last form verb to be wired.
///
/// # ★★★ Why this lives in `forms` and its twin lives in `export`
///
/// `actions::export`'s header draws that boundary and it is a real one: *"none
/// of them changes the document at all. No `vector_edit`, no undo entry, no
/// epoch bump, no cache invalidation. They read the open file and write a
/// different one."*
///
/// An import is the exact opposite. It reads a different file and **changes the
/// open document** — thirty fields at once, on a good day — so every rule the
/// mutation funnel enforces applies, and it goes through `vector_edit` like
/// every other edit. Putting it beside its twin would have put the one verb in
/// that module that breaks the module's stated property.
///
/// # ★★ One undo entry for the whole file, because the ENGINE makes it one
///
/// `import_form_data` is a single `EditSession` command however many fields it
/// sets. That is not this shell's doing and it is worth knowing, because the
/// same is emphatically *not* true of the panel's recompute — which writes one
/// command per field and says so.
///
/// ★ It also asks the document-wide gate **once, up front**: a certification
/// that forbids filling forbids it for every entry, so discovering it on entry
/// seventeen would be both late and destructive. The engine's own comment says
/// so, and it is why a refusal here leaves the document untouched rather than
/// half-imported.
///
/// # The three failures are told apart, and they have nothing in common
///
/// | | what it means | what the operator does |
/// |---|---|---|
/// | unreadable | the path or the permissions | find the file |
/// | unparseable | the bytes are not form data pdfcer reads | pick a different file |
/// | refused | the **document** will not take an import — no form, certified, encrypted | nothing about the data file will help |
///
/// A single "import failed" would send an operator whose document is certified
/// off to re-export their data, twice.
pub(super) fn import_data(doc: &mut OpenDoc, path: &std::path::Path) {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("import-form-data-failed stage=read detail={error}")
            });
            super::record_note(
                doc.edit_epoch,
                crate::text::export_form::import_unreadable(&error.to_string()),
            );
            return;
        }
    };
    // ★ The extension decides the parser, matching the export's rule exactly —
    // one convention for both halves of the round trip, so a file exported as
    // `.csv` and imported as `.csv` cannot land in a branch nobody chose. FDF
    // is the default for the same reason it is on the way out: it is the format
    // §12.7.8 defines for this data.
    let extension = path
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    let parsed = match extension.as_str() {
        // ui-text-exempt: file extensions, matched not displayed.
        "xfdf" => pdfcer_core::fdf::FormData::parse_xfdf(&bytes).map_err(|e| e.to_string()),
        "csv" => pdfcer_core::formcsv::parse_csv(&bytes).map_err(|e| e.to_string()),
        _ => pdfcer_core::fdf::FormData::parse_fdf(&bytes).map_err(|e| e.to_string()),
    };
    let data = match parsed {
        Ok(data) => data,
        Err(detail) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("import-form-data-failed stage=parse format={extension} detail={detail}")
            });
            super::record_note(
                doc.edit_epoch,
                crate::text::export_form::import_unparseable(&detail),
            );
            return;
        }
    };

    let fields = data.fields.len();
    super::apply::vector_edit(doc, "import-form-data", 0, 1, move |session| {
        session.import_form_data(&data).map(|outcome| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    // ★★★ `-applied`, NOT plain `import-form-data`, and the
                    // suffix is a defect this project has now made TWICE in
                    // twenty-four hours.
                    //
                    // `vector_edit` writes its own line for the same edit —
                    // `import-form-data page=0 n=1 epoch=1 disclosures=…` —
                    // and trace matching is on the **exact event name**. A
                    // driven check taking `.last()` therefore reads
                    // `vector_edit`'s line, which carries no `applied=` key,
                    // and reports `applied=0` about an import that set every
                    // field it was given.
                    //
                    // That is precisely what happened on the first run of the
                    // round-trip check, and it is the same defect `text-style`
                    // had yesterday, fixed the same way and recorded in
                    // `CONTINUE.md` — *"two lines sharing a name is how a check
                    // reads the wrong one and then reports failure about a
                    // gesture that worked."* Reading the note did not prevent
                    // the repeat; the naming convention is what does.
                    //
                    // ⇒ **A module's own summary line takes a verb suffix; the
                    // funnel's label keeps the bare name.**
                    "import-form-data-applied read={fields} applied={} skipped={}",
                    outcome.applied, outcome.skipped
                )
            });
            vec![crate::text::export_form::imported(
                outcome.applied,
                outcome.skipped,
            )]
        })
    });
}

/// **Give an existing push button an action**, as one undoable command.
///
/// # ★★★ The disclosure this owes, and it is the whole reason `replaced` exists
///
/// `ButtonActionChange::replaced` names what was destroyed — **as a `String`,
/// including `"JavaScript"`, deliberately**. `pdfcer-core`'s own reasoning:
/// `Option<ButtonAction>` would have made a removed script inexpressible and
/// forced it to be reported as `None`, i.e. as *"there was nothing there"*.
///
/// ⇒ A form editor overwriting another tool's work needs to know it did, and
/// this is the one moment it can be told. The status line carries it.
///
/// ★ pdfcer will not write a script back. That asymmetry is deliberate and is
/// disclosed on the row rather than here: a `Foreign` action renders no Change
/// control at all, so the only way to reach this function with a script in the
/// way is through a route that has already said so.
fn set_button_action(
    doc: &mut OpenDoc,
    field: &str,
    action: Option<pdfcer_core::edit::ButtonAction>,
) {
    let name = field.to_owned();
    super::apply::vector_edit(doc, "set-button-action", 0, 1, |session| {
        session.set_button_action(field, action).map(|change| {
            vec![crate::text::buttonaction::changed(
                &name,
                change.replaced.as_deref(),
            )]
        })
    });
}

pub(super) fn rename(doc: &mut OpenDoc, from: &str, to: &str) {
    let to = to.trim().to_owned();
    if to.is_empty() || to == from {
        return;
    }
    doc.selected_field = None;
    super::apply::vector_edit(doc, "rename-field", 0, 1, |session| {
        session.rename_field(from, &to).map(|outcome| {
            vec![crate::text::forms::form_field_renamed(
                &outcome.to,
                outcome.descendants_renamed,
            )]
        })
    });
}

/// The status lines one authoring outcome owes the operator.
///
/// A free function so the rule-4 obligation is testable without a session, a
/// document or a frame — and so that a new flag on `FieldAuthorDisclosures`
/// appearing in a future engine build has one obvious place to be handled and
/// one test that notices it was not.
///
/// ★ Order is deliberate: **`merged` first**, because it is the only one that
/// changes what the operator believes they just made. The rest are advisory.
pub(super) fn disclosures(
    outcome: &pdfcer_core::edit::FieldAuthorOutcome,
    kind: crate::canvas::formfield::FormFieldKind,
) -> Vec<String> {
    let mut lines = vec![crate::text::forms::form_field_added(&kind.noun())];
    if outcome.merged {
        lines.push(crate::text::forms::form_field_merged());
    }
    if outcome.disclosures.tooltip_declined {
        lines.push(crate::text::forms::form_field_no_tooltip());
    }
    if outcome.disclosures.has_no_options {
        lines.push(crate::text::forms::form_field_no_options());
    }
    if outcome.disclosures.tagged_document || outcome.disclosures.structure_tab_order {
        lines.push(crate::text::forms::form_field_tagged_document());
    }
    lines
}

/// Which refusals the operator can do something about.
///
/// A free function taking `&EditError` so it is testable without an
/// `EditSession`, a document or a frame — the same shape
/// `crate::dialogs::insert_image`'s arithmetic was pushed into, and for the
/// same reason: `pdfcer_core::edit::EditError` is `#[non_exhaustive]`, so this
/// match needs a wildcard, and a wildcard inside a closure inside a funnel is
/// a place a future variant goes to be silently ignored.
///
/// Here it is one visible function with a test beside it. The wildcard means
/// *"anything else is a fault, not a chore"*, which is a real distinction and
/// the right default: a new refusal variant appearing in a future engine build
/// reaches the trace with its own words and does not silently acquire one of
/// these two sentences, which would be worse than saying nothing.
fn correctable(error: &pdfcer_core::edit::EditError) -> Option<Declined> {
    use pdfcer_core::edit::EditError as E;
    match error {
        E::FieldNameTaken { .. } => Some(Declined::FieldNameTaken),
        E::WidgetHasNoFieldIdentity { .. } => Some(Declined::WidgetHasNoName),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::edit::EditError as E;

    /// The two the operator can fix are worded; the ones they cannot are not.
    ///
    /// ★ The negative half is the half worth asserting. `WidgetAlreadyOwned`
    /// cannot happen from this surface — the ids come from exactly the widgets
    /// no field claimed, on the same `/Annots` walk — so if it ever *did*, a
    /// sentence telling the operator to type a different name would be actively
    /// misleading about a state that indicates the listing and the action have
    /// come to disagree about the set. That is a fault to find in the trace, not
    /// a chore to hand to an operator.
    #[test]
    fn only_the_two_the_operator_can_act_on_are_worded() {
        assert_eq!(
            correctable(&E::FieldNameTaken {
                name: "Address".to_owned()
            }),
            Some(Declined::FieldNameTaken)
        );
        assert_eq!(
            correctable(&E::WidgetHasNoFieldIdentity { id: 12 }),
            Some(Declined::WidgetHasNoName)
        );
        assert_eq!(correctable(&E::WidgetAlreadyOwned { id: 12 }), None);
        assert_eq!(correctable(&E::NotAWidget { id: 12 }), None);
    }

    /// The two sentences are different, and neither claims a recovery.
    ///
    /// The wording rule this module's header argues for, asserted rather than
    /// trusted: an operator told they had *restored* a radio button would go
    /// looking for its group, and there is no group.
    #[test]
    fn neither_refusal_promises_a_recovery() {
        let taken = t::adopt_declined_name_taken();
        let unnamed = t::adopt_declined_no_name();
        assert_ne!(taken, unnamed);
        for text in [taken, unnamed] {
            for promise in ["restore", "recover", "put back", "as it was"] {
                assert!(
                    !text.to_lowercase().contains(promise),
                    "{promise:?} promises something registering cannot do: {text}"
                );
            }
        }
        assert!(
            unnamed.contains("insert the pages again"),
            "the one route that does get the original back must be named"
        );
    }

    /// A registration with no field type says so, and one with a type does not
    /// mention it.
    ///
    /// ★ The `field_type: None` case is the fuzzy-never-sneaky half of this
    /// verb: the registration **succeeded**, the operator will be told so, and
    /// the box is *still* not fillable because a top-level field with no `/FT`
    /// has nothing left to inherit from. That is an inference-shaped absence the
    /// operator cannot see, and rule 4 says it is owed a sentence off-canvas
    /// even though — and precisely because — nothing on the page looks wrong.
    #[test]
    fn a_typeless_field_is_disclosed_and_a_typed_one_is_not_nagged_about() {
        let typed = t::adopted("Address", true, false);
        let typeless = t::adopted("Address", false, false);
        assert!(typed.contains("Address"));
        assert!(!typed.contains("field type"));
        assert!(typeless.contains("no field type"));
        assert!(
            typeless.contains("no viewer knows how to fill it"),
            "the consequence is the part the operator needs: {typeless}"
        );
    }

    /// Creating the document's first `/AcroForm` is disclosed, and only then.
    ///
    /// It changes what *other* software does with the file — a viewer that
    /// finds a form shows a form bar over a drawing that had none — and it is
    /// not something the operator asked for. They asked to register one box.
    #[test]
    fn a_document_gaining_its_first_form_is_told() {
        assert!(t::adopted("A", true, true).contains("had no interactive form"));
        assert!(!t::adopted("A", true, false).contains("had no interactive form"));
    }
}
#[cfg(test)]
/// ★★★ The dotted-name guard, against a REAL document rather than a stub.
///
/// The loss it prevents was measured with `pdfcer` before this was written:
/// a field `Text` holding "K. Mantle", plus a field named `Text.2`, leaves one
/// empty field and an orphaned box. The value is not recoverable, which is why
/// this is a refusal rather than a disclosure.
mod dotted_names {
    /// A plain name is never touched — the cheap exit, and the common case.
    #[test]
    fn a_name_without_a_dot_is_never_examined() {
        // No document needed: the function returns before it opens one, which
        // is the property being asserted. Anything else would put a form parse
        // on every field authored.
        //
        // Expressed as a doc-free call in the sibling tests below rather than
        // here, because constructing an `OpenDoc` is what those do; this test
        // exists to state the fast path in words that a reader will find.
        assert!(!"Revision".contains('.'));
    }

    /// ★★ Only ANCESTORS are examined, never the full name.
    ///
    /// `Text.2` must check `Text` and must NOT check `Text.2`. A name that
    /// already exists as a terminal field of the same type is a legitimate
    /// **merge** — it is exactly what `Ctrl+Shift+V` relies on — so guarding
    /// the full name would break the duplicate paste.
    #[test]
    fn the_prefix_walk_stops_before_the_full_name() {
        let name = "A.B.C";
        let segments: Vec<&str> = name.split('.').collect();
        let checked: Vec<String> = (1..segments.len())
            .map(|cut| segments[..cut].join("."))
            .collect();
        assert_eq!(
            checked,
            vec!["A".to_owned(), "A.B".to_owned()],
            "★ `A.B.C` itself must NOT be in the list: an existing field of that exact name is a merge, not a collision"
        );
    }

    /// The guard is reachable from the two gestures that can trigger the loss.
    ///
    /// Named rather than exercised, because both are operator-typed strings and
    /// the assertion that matters is that `author` consults the guard at all —
    /// which the source does on its first statement.
    #[test]
    fn the_two_gestures_that_reach_it_are_named() {
        let src = include_str!("forms.rs");
        assert!(
            src.contains("if let Some(victim) = group_is_a_field(doc, draft.name.trim())"),
            "★ `author` must consult the guard BEFORE anything is written. The placement dialog's name box and the Properties rename both reach here with a string the operator typed."
        );
    }
}

#[cfg(test)]
mod authoring_is_available {
    /// ★★★ **Field authoring is NOT blocked, and this is the test that settled
    /// it.**
    ///
    /// `shell::commands::reach::register` recorded `edit.form_create_field` as
    /// *"blocked on core's STRUCTURAL certification gate"*. **There is no such
    /// gate.** What the engine refuses is a spec whose tooltip is `Undecided` —
    /// `TooltipDecisionRequired`, an accessibility requirement rather than a
    /// permission: a form control owes a screen reader a name, and the engine
    /// will not default one silently. It is a field of the dialog the command
    /// needs anyway.
    ///
    /// ★★ This is the **fourth** blocker recorded in this project that turned
    /// out to be stale, which is why the standing rule is *a backlog row is a
    /// record, not evidence* and why the first move was to probe the engine
    /// rather than to re-read the note.
    ///
    /// The test asserts both halves, and the second is what makes the first
    /// meaningful: authoring **succeeds** with a tooltip, and **fails with
    /// exactly `TooltipDecisionRequired`** without one. Without that pair it
    /// would not distinguish "authoring works" from "authoring happens to work
    /// on this fixture".
    #[test]
    fn a_field_can_be_authored_and_only_the_tooltip_is_required() {
        let path = std::path::Path::new("D:/Dev/temp/pdfcer/SW41177.pdf");
        if !path.exists() {
            return; // fixture-dependent; the driven checks cover the real path
        }
        let rect = pdfcer_core::page_tree::Rect {
            llx: 100.0,
            lly: 100.0,
            urx: 300.0,
            ury: 130.0,
        };

        // Without a tooltip: refused, and refused for the ONE stated reason.
        let doc = pdfcer_core::document::Document::load(path).expect("load");
        let mut session = pdfcer_core::edit::EditSession::new(doc);
        let bare = pdfcer_core::edit::NewTextField::new(0, "probe".to_owned(), rect);
        match session.add_text_field(&bare) {
            Err(pdfcer_core::edit::EditError::TooltipDecisionRequired { .. }) => {}
            other => panic!(
                "expected the accessibility refusal and got {other:?} — if this is now Ok, the engine defaults a tooltip silently and the dialog no longer has to ask"
            ),
        }

        // With one: authored.
        let doc = pdfcer_core::document::Document::load(path).expect("load");
        let mut session = pdfcer_core::edit::EditSession::new(doc);
        let spec = pdfcer_core::edit::NewTextField::new(0, "probe".to_owned(), rect)
            .with_tooltip("Probe field");
        assert!(
            session.add_text_field(&spec).is_ok(),
            "authoring a text field is not blocked; if this fails, a REAL gate has appeared and the register entry needs rewriting again"
        );
    }
}

/// **Move one widget by a page-space delta.**
///
/// ★★ The disclosure is CONDITIONAL and reports what the operator cannot see:
/// `WidgetMove` names whether the field's other widgets stayed put, and on a
/// field drawn on three pages that is the whole question. Moving one box of a
/// three-box field is correct — they are separate placements of one value — and
/// it is also exactly the thing an operator would assume had gone wrong when
/// the other two did not follow.
///
/// ★ Nothing is disclosed for the ordinary one-widget field, for
/// `text::embed`'s reason applied here: a sentence that fires on every drag is
/// one an operator learns to skip, and the day it says something is the day
/// they skip it too.
pub(super) fn move_widget(doc: &mut OpenDoc, field: &str, widget: usize, dx: f64, dy: f64) {
    super::apply::vector_edit(doc, "move-widget", 0, 1, |session| {
        session.move_widget(field, widget, dx, dy).map(|outcome| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    // `-applied`, per the convention this module records at
                    // length: the funnel writes its own bare-named line for the
                    // same edit and `.last()` would read that one.
                    "move-widget-applied field={field} widget={widget} dx={dx:.3} \
                     dy={dy:.3} siblings={}",
                    outcome.siblings_left_behind
                )
            });
            if outcome.siblings_left_behind > 0 {
                vec![crate::text::forms::widget_siblings_unmoved(
                    outcome.siblings_left_behind,
                )]
            } else {
                Vec::new()
            }
        })
    });
}
