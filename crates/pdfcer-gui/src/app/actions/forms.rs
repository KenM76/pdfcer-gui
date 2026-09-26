//! # `app::actions::forms` — everything done to a form FIELD
//!
//! A sibling of [`super::dimensions`], [`super::pages`], [`super::vector`] and
//! [`super::export`], and it owns both halves of its subject: the action enum
//! [`FieldAction`] and the apply logic every one of its variants reaches.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/forms.md`.

/// **Authoring a form control from the placement dialog's choices.**
///
/// Its sibling [`paste`] is authoring from a SOURCE; the two are not
/// duplicates, and each header says which input it starts from.
pub(super) mod author;
/// Deleting a **grouping node** — the two-press verb, its preview store and
/// both apply paths.
pub mod delete;
pub mod groups;
/// **Putting a copied form field back** — `EditSession::paste_field`. Its
/// header carries why the shell does almost nothing in it any more.
mod paste;
/// **Verbs about the BOX rather than the field** — rotation today, and the
/// natural home for the next one. A field's identity and a widget's placement
/// are two subjects.
mod widget;

use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;
use crate::app::status::decline::{self, Declined};
use crate::text::status as t;

impl From<FieldAction> for super::action::Action {
    /// So a call site says what it MEANS and the wrapping is not its problem.
    fn from(f: FieldAction) -> Self {
        Self::Field(f)
    }
}

/// One thing done to a form field, carried by
/// [`super::action::Action::Field`].
///
/// See the module header for why these are one family and what test a new
/// variant has to pass to join them.
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
    /// It changes **no document** and must never bump the edit epoch. A
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
    /// # A flag is editable in place; never offer delete-and-replace instead
    ///
    /// Delete-and-replace loses the field's name, its filled value and its
    /// place in the tab order, so no panel may tell an operator to do it in
    /// order to change a flag. Every property this variant reaches is settable
    /// on a field that is already placed.
    ///
    /// The general rule behind that: **an absence claim about the engine has a
    /// shelf life**, because the engine is a branch dependency that moves
    /// independently of this shell. Before a panel says a capability does not
    /// exist, check the engine's current surface and its replies on the request
    /// channel — a claim that was true when written goes false without anything
    /// in this repository changing.
    ///
    /// # One variant, one property, one undo entry
    ///
    /// `FieldEdit` can carry fourteen properties at once and this deliberately
    /// sends one at a time, which is `StyleChange`'s rule for the same reason:
    /// **one control press is one undo entry.** A pane that batched a required
    /// flag and a max-length into one request — which the engine supports —
    /// would make `Ctrl+Z` after two separate presses take back a state the
    /// operator never saw.
    ///
    /// The **one** exception the engine names is genuinely a single act:
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
        /// The engine's §6: *"the gates are checked against the RESULT, not
        /// against your request"*, so `.with_max_len(None)` on a **comb** field
        /// refuses with `CombPreconditionUnmet` — naming a property the request
        /// never mentioned. Its own instruction: *"show it against the control
        /// the operator touched, not the one the standard named."* This carries
        /// which control that was, because after the fact nothing else can say.
        touched: &'static str,
    },
    /// **Turn a field's box**, in ninety-degree steps.
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
    /// # It carries the widget INDEX, and that is the whole difference
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
    /// # The path is carried, and the picker ran BEFORE the action
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
    /// The old name travels even though the selection holds it, and that is
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
    /// **Give an existing push button an action, or take one away.**
    ///
    /// Raised by `crate::panels::forms::button` and by nothing else.
    ///
    /// # `None` is a real operand, not an absence
    ///
    /// `set_button_action` takes `Option<ButtonAction>` and `None` **removes**
    /// whatever is there — the half a form editor needs when it opens somebody
    /// else's document and wants the button inert. So this variant carries an
    /// `Option` rather than being two variants: *set* and *clear* are one verb
    /// with one refusal set, and splitting them here would give the shell two
    /// spellings of one act.
    ///
    /// Boxed because `ButtonAction` carries a `SubmitSpec`, which is much the
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
    /// Distinct from [`Self::DeleteWidget`] and the distinction is not a
    /// nicety: one field may be drawn in several places, and "remove this box"
    /// and "remove this field" are different requests with different
    /// consequences. Offering only the second would make removing one of three
    /// copies impossible; offering only the first would leave a named field
    /// behind with no widgets, which is a field nothing can fill.
    DeleteField {
        /// The field's fully-qualified name.
        field: String,
    },
    /// **Move one widget of a field by a page-space delta.**
    ///
    /// Raised by `crate::canvas::widgetdrag` on the release of a drag, and by
    /// nothing else.
    ///
    /// # Why not `Action::MoveAnnotation`, when a widget IS an annotation
    ///
    /// Because the engine refuses that by name and says why: `move_widget`
    /// *"does strictly more, and quietly doing less under this name would give
    /// you a second way to move the same thing that silently produces a worse
    /// result."* What it does more of is the **field** -- a widget is addressed
    /// by its field's fully-qualified name and an index within it, because one
    /// field can draw boxes on three pages and the `/Annots` entry is not the
    /// thing the operator renamed.
    ///
    /// The two verbs differ in their ADDRESS, not in their geometry. Worth
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
    /// # Object ids, not indices, and the engine asked for it by name
    ///
    /// `EditSession::reorder_annotations` takes `&[ObjId]`. Its shipping note
    /// says why in one sentence: *"the index you hold is almost never a raw
    /// `/Annots` index — `page_annotations` skips null and non-dictionary
    /// entries, so the numberings diverge on exactly the malformed files where a
    /// guess costs most."*
    ///
    /// The tab-order panel's `TabRow::position` is emphatically **not** an
    /// address: 1-based, widgets only, and a label an operator counts while
    /// tabbing. `TabRow::id` is the address, and it exists for this variant.
    ///
    /// # Why it is a FieldAction and not a top-level `Action`
    ///
    /// Because it is a form verb raised by the Forms panel, which is what this
    /// sub-enum is for. Grouping by subject rather than by which enum happens
    /// to be open is the established pattern (`Annot`, `Page`, `Vector`,
    /// `Text`, `Write`, …), and this is the case it was made for.
    ReorderAnnotations {
        /// The page, 0-based.
        page: usize,
        /// Every **indirect** entry of that page's `/Annots`, each once, in the
        /// wanted order.
        ///
        /// *Every* entry, not only the widgets the panel lists. The engine
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
    /// It **changes no document** and must never bump the edit epoch — the
    /// preview writes nothing. It is here rather than in the panel for one
    /// reason: the engine's signature is `&mut self`, a panel body holds
    /// `&OpenDoc`, and `Arc::get_mut` only succeeds inside the funnel. So a
    /// query that changes nothing is nonetheless an action, exactly as
    /// [`Self::Select`] is.
    ///
    /// `Option` rather than a second variant, for [`Self::Select`]'s reason:
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
    /// Distinct from [`Self::DeleteField`], and the engine refuses to let
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
    /// The geometry travels and the details do not, for the reason
    /// [`super::action::Action::BeginTextAnnot`] gives at length: the rectangle is a choice the
    /// operator made *now*, on the page they were looking at, and the details
    /// are made later in a dialog and may never be made at all.
    ///
    /// There is deliberately no `name` on it. The name is generated when the
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
    /// The whole draft travels, for the reason [`super::action::Action::CommitTextAnnot`]
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
    /// **Author the form control that came off the clipboard.**
    ///
    /// Raised by [`crate::canvas::fieldclip::paste`] and by nothing else.
    /// `OPERATOR_REQUESTS.md` **O58**.
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
    /// The two variants carry identical data and differ in one side effect,
    /// which is exactly when two variants are correct rather than one with a
    /// flag: the flag would be read at the only place that can act on it and
    /// would be invisible everywhere else, including here.
    ///
    /// # What is NOT decided here
    ///
    /// Whether this is a new field or a second widget of an existing one. That
    /// is settled entirely by [`crate::canvas::formfield::Draft::name`] before
    /// the action is raised — a name that matches an existing field **merges**,
    /// which the engine reports as `FieldAuthorOutcome::merged`; a fresh one
    /// does not. So the two chords produce the same variant carrying different
    /// names, and
    /// `super::author`'s existing disclosure pass reports `merged` without
    /// needing to know which chord was pressed.
    Paste {
        /// The 0-based page it lands on.
        page: usize,
        /// Where, in PDF user space — already offset or already in place, per
        /// [`crate::canvas::fieldclip::paste`]'s same-page/cross-page rule.
        ///
        /// On a **radio group** the engine uses only the lower-left corner
        /// and discloses that the size was ignored: a group's geometry is part
        /// of its meaning, so it translates rather than rescaling.
        rect: pdfcer_core::page_tree::Rect,
        /// `FieldClip::to_bytes` — the clip itself, verbatim off the clipboard.
        ///
        /// Bytes rather than a live `FieldClip` because that is what the
        /// clipboard holds and because an `Action` is `Clone + PartialEq`. The
        /// engine's format is total and byte-exact — it tests that a clip
        /// through bytes and one that stayed in memory produce identical
        /// documents — so nothing is lost by carrying it this way.
        clip: Vec<u8>,
        /// New independent field, or another widget of the existing one.
        ///
        /// Boxed: `FieldPastePolicy::NewField` carries a `String` name and a
        /// `PasteTooltip` that may carry another, and this enum has thirty-odd
        /// variants that would all grow to match.
        policy: Box<pdfcer_core::formclip::FieldPastePolicy>,
    },
    /// **Register a form control the document draws but no field claims.**
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
    /// one of those **overrides** a name the file already had. See [the module
    /// header](self) for the two shapes and why an operator cannot tell them
    /// apart by looking.
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
pub(super) fn apply(doc: &mut OpenDoc, action: FieldAction) {
    match action {
        // Selection is VIEW STATE. It changes no document, bumps no epoch and
        // invalidates no page — which is why it does not go near the funnel.
        FieldAction::Select(selected) => doc.selected_field = selected,
        // One line: the panel already resolved left/right into a
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
        // The arm changes no document and bumps no epoch, so it does not go
        // near `vector_edit`; the deletion does, like every other structural
        // form verb. See `groups`' header for why a query needs to be an action
        // at all.
        FieldAction::ArmGroupDeletion(group) => groups::arm(doc, group),
        FieldAction::DeleteGroup { group } => groups::delete(doc, &group),
        FieldAction::Adopt { page, widget, name } => adopt(doc, page, widget, name),
        FieldAction::Edit(edit) => crate::panels::forms::edit::apply(doc, &edit),
        // Unreachable rather than unhandled, and named so the compiler will
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

// A dotted field name that crosses an existing TERMINAL field is refused by
// the engine, not by a pre-check here. `place_new_field_deferred` is the single
// choke point every `add_*` verb and `paste_field` reach, and it raises
// `FieldPathCrossesTerminal` before a byte is staged or an undo entry pushed.
//
// Do not reintroduce a shell-side prefix walk. The engine refuses only when the
// deepest existing node on the path is a **terminal** (`child_field_count ==
// 0`, §12.7.3.1's own definition); a walk that refuses on any prefix present in
// `AcroForm::fields` is a different, wider set. The two differ on the **mixed
// node** — a node carrying child fields *and* its own bare widget kids, which
// `pdfcer_core::forms`'s `walk_field` models and pdfcer's own same-name merge
// can generate. A second child there loses nothing and the engine allows it; a
// prefix walk refuses it while claiming a field would be destroyed.
//
// The shell's whole share of this is `correctable`'s `FieldPathCrossesTerminal`
// arm, which words the refusal with the name **the engine identified**. That
// answer cannot drift from the engine's, because it is the engine's.
//
// The sentence lives in `crate::text::fieldclip::name_crosses_a_field`. The
// engine's own `Display` prose names the victim and cites the clause, but it
// reaches `PDFCER_DIAG` and stops there: `check-ui-strings.sh`'s exclusion 3 is
// explicit that an error type's `Display` is not permission to route operator
// text through it. Without the arm, a correct engine refusal would reach the
// operator as the floor's generic *"That change was refused"*.

/// **Change one property of an existing field.**
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
            // Verbatim, and FIRST. It is the one line that says the
            // operator's stored data no longer matches the field's own rules,
            // which outranks every count.
            if let Some(why) = outcome.value_no_longer_fits {
                lines.push(why);
            }
            if outcome.sort_claim_unmet {
                lines.push(crate::text::forms::field_sort_claim_unmet().to_owned());
            }
            // A reorder the shell did not expect. `choiceopts` sorts the list
            // it sends with the engine's own exported sorter, so this is
            // normally false and says nothing — it is true only if the two
            // orderings have come apart, which is the drift `G026` exported the
            // comparator to prevent and which no test on either side can see.
            if outcome.options_sorted {
                lines.push(crate::text::forms::field_options_reordered().to_owned());
            }
            if outcome.widgets_affected > 1 {
                lines.push(crate::text::forms::field_widgets_affected(
                    outcome.widgets_affected,
                ));
            }
            // Nothing at all when the edit was ordinary, which is most of the
            // time. A bar that narrated every checkbox would stop being read,
            // and `vector_edit` treats an empty list as "no line".
            let _ = touched;
            lines
        })
    });
}

/// **Change one placement of a field** — its rectangle, its border, its
/// caption, its visibility, or either of its two `/MK` colours.
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
            // **The trace this verb never had** — `OPERATOR_REQUESTS.md`
            // O76. Its two siblings, `move-widget-applied` and
            // `rotate-widget-applied`, have always reported their outcome; this
            // one reported nothing, which is why a check box quietly stretching
            // its own artwork for weeks was invisible to every driven run.
            //
            // All three fields, because a screenshot cannot separate them: a
            // border that thickened because `/BS /W` changed and one that
            // thickened because §12.5.5's placement matrix scaled it are the
            // same pixels. `regenerated=` is the field that tells them apart,
            // and it is the assertion the O76 check is built on.
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "edit-widget-applied field={field} widget={widget} touched={touched:?} \
                     geometry={} resized={} appearance={:?}",
                    outcome.rect_after.is_some(),
                    outcome.resized,
                    outcome.appearance,
                )
            });
            let mut lines = Vec::new();
            // **First**, because it is the only one about something on screen.
            //
            // Read off `AppearanceOutcome` rather than off `appearance_stale`
            // being `Some`, which is what the engine's own field doc asks for:
            // `appearance_regenerated: false` + `appearance_stale: None` meant
            // two different things — *nothing needed redrawing* and *something
            // did and pdfcer could not* — and this crate read the ambiguous
            // pair for the whole of the time `with_background` existed. An enum
            // cannot silently acquire a fourth meaning for an existing value.
            if let pdfcer_core::edit::AppearanceOutcome::RecordedNotPainted(why) =
                &outcome.appearance
            {
                lines.push(crate::text::forms::field_appearance_not_repainted(
                    outcome.resized,
                    why,
                ));
            }
            // **`rect_after`, not "always"**. This line was pushed
            // unconditionally, so every border, caption, visibility and — since
            // O202 — colour edit ended with *"The box was moved."* on the
            // status line, naming an act the operator had not performed. The
            // engine reports geometry as `Option`; a `None` there means the
            // edit never touched the rectangle, and there is nothing to say
            // about a move that did not happen.
            if outcome.rect_after.is_some() {
                lines.push(
                    crate::text::forms::field_widget_moved(
                        outcome.resized,
                        outcome.appearance_regenerated,
                    )
                    .to_owned(),
                );
            } else {
                lines.push(crate::text::forms::field_widget_property_changed(
                    touched,
                    outcome.appearance_regenerated,
                ));
            }
            if outcome.siblings_untouched > 0 {
                lines.push(crate::text::forms::field_siblings_untouched(
                    outcome.siblings_untouched,
                ));
            }
            lines
        })
    });
}

/// **Read an FDF, XFDF or CSV file and set this document's field values from
/// it.**
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
    // The extension decides the parser, matching the export's rule exactly —
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
                    // `-applied`, NOT plain `import-form-data`.
                    //
                    // `vector_edit` writes its own line for the same edit —
                    // `import-form-data page=0 n=1 epoch=1 disclosures=…` —
                    // and trace matching is on the **exact event name**. Two
                    // lines sharing a name is how a check reads the wrong one
                    // and then reports failure about a gesture that worked: a
                    // driven check taking `.last()` gets `vector_edit`'s line,
                    // which carries no `applied=` key, and reports
                    // `applied=0` about an import that set every field it was
                    // given.
                    //
                    // **A module's own summary line takes a verb suffix; the
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

/// **Rename the selected field.**
pub(super) fn rename(doc: &mut OpenDoc, from: &str, to: &str) {
    let to = to.trim().to_owned();
    if to.is_empty() || to == from {
        return;
    }
    doc.selected_field = None;
    super::apply::vector_edit(doc, "rename-field", 0, 1, |session| {
        session
            .rename_field(from, &to)
            // `inspect_err`, so the error is still returned unchanged for the
            // funnel to trace and for the floor to word if `correctable`
            // declines to claim it. Recording is an addition to the error path,
            // never a substitution for it.
            .inspect_err(|error| {
                if let Some(declined) = correctable(error) {
                    decline::record_field_rename_refusal(declined);
                }
            })
            .map(|outcome| {
                let mut lines = vec![crate::text::forms::form_field_renamed(
                    &outcome.to,
                    outcome.descendants_renamed,
                )];
                // Rule 4: pdfcer rewrote buttons the operator did not touch.
                //
                // `/ResetForm` and `/SubmitForm` name their targets as fully
                // qualified NAME STRINGS, so a rename that did nothing else would
                // leave them pointing at nothing. `rename_field` repairs them —
                // correctly, invisibly, and not as anything the operator pressed.
                // No view in this shell shows an action's target list, so without
                // this sentence the repair is unobservable.
                //
                // Conditional, like every disclosure on this surface: a rename
                // that touched no action says one thing. A receipt that recites
                // "0 buttons updated" after every rename is a form, and by the
                // third one nobody reads the line that matters.
                if outcome.action_targets_retargeted > 0 {
                    lines.push(crate::text::forms::form_field_actions_retargeted(
                        outcome.action_targets_retargeted,
                    ));
                }
                lines
            })
    });
}

/// The status lines one authoring outcome owes the operator.
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
fn correctable(error: &pdfcer_core::edit::EditError) -> Option<Declined> {
    use pdfcer_core::edit::EditError as E;
    use pdfcer_core::forms_author::FormAuthorError as A;
    match error {
        E::FieldNameTaken { .. } => Some(Declined::FieldNameTaken),
        E::WidgetHasNoFieldIdentity { .. } => Some(Declined::WidgetHasNoName),
        // Unreachable from `adopt_widget` and the only refusal `author` can
        // raise — which is why this is a table shared by two surfaces rather
        // than a helper belonging to one of them.
        //
        // `EditError::FieldAuthoring` is `#[error(transparent)]`, so the inner
        // variant is the whole of the error and matching it is matching the
        // refusal. `terminal` is the fully-qualified name of the existing field
        // in the way, which the engine built by walking `/Parent` — it is not
        // derivable from the operator's string, which is exactly why it is
        // cloned out and carried rather than recomputed.
        //
        // Matched on the VARIANT, never on `error.to_string()`. The engine's
        // prose for this refusal is good and names the field, and it is still
        // the wrong thing to key on: prose is the part of an API with no
        // compatibility promise, so an arm keyed on words goes on firing after
        // the engine narrows the condition underneath it.
        E::FieldAuthoring(A::FieldPathCrossesTerminal { terminal, .. }) => {
            Some(Declined::FieldPathCrossesTerminal(terminal.clone()))
        }
        // A **floor**, not the first line of defence: no surface in this shell
        // currently reaches it, because each of the three routes to a partial
        // name greys its own control first.
        //
        // | route | the control | reaches this arm |
        // |---|---|---|
        // | `adopt_widget` | `panels::forms::tab_order::register`'s per-widget name box — free text, but the row calls `adopt_preview` every frame and `add_enabled(false, ..)`s the button on any `Err` | no — the button greys while the period is typed |
        // | `rename_field` | `panels::properties::formfield`'s name box — commit greyed on `validate_partial_name` | no |
        // | `sign` | no name box at all; the window lists fields that exist, and an existing FQN takes the engine's reuse branch where a period is legitimate | no |
        //
        // Kept rather than deleted because every `no` above is a claim about a
        // shell-side gate, not about the engine. Two are judgments a refactor
        // can relax; the third is a preview call that a `&self`-to-`&mut self`
        // change upstream would silently end. When one goes, the operator gets
        // a sentence instead of `Display` output with nobody having to notice.
        //
        // The general rule the table teaches: **a guard's placement decides
        // which surface has to explain it.** `reject_dotted_partial` lives
        // inside the engine's `adopt_plan`, and `adopt_preview` is that same
        // plan with the writes dropped, so the refusal arrives in the preview a
        // row already draws from — a hover, not the status bar. A panel that
        // greys on a refusal it cannot name falls into `refusal_hint`'s
        // catch-all, which is the program apologising for a rule it is
        // enforcing correctly; give it wording of its own instead.
        //
        // Measured by two deliberately separate tests: the grey by
        // `panels::forms::tab_order::register::tests::a_dotted_name_greys_the_register_button_and_the_hover_names_the_rule`,
        // this arm by `tests::adopting_under_a_dotted_name_is_refused_and_worded`.
        E::FieldAuthoring(A::DottedPartialName { supplied }) => {
            Some(Declined::DottedPartialName(supplied.clone()))
        }
        // A rename onto a name something already bears: the **most common**
        // way a rename fails, and the **only** refusal the rename surface can
        // reach that its own gate cannot predict, because predicting it needs
        // the field tree rather than the typed string.
        //
        // Mapped to `FieldNameTaken` rather than to a variant of its own,
        // under *one fact, one wording*: the engine raises two variants here
        // (`EditError::FieldNameTaken` from `adopt_widget`,
        // `FormAuthorError::RenameCollision` from `rename_field`) because they
        // are produced by different code, but from the operator's chair they
        // are one sentence — *something already has that name* — with one
        // remedy. See that variant's own doc for the table.
        E::FieldAuthoring(A::RenameCollision { .. }) => Some(Declined::FieldNameTaken),
        _ => None,
    }
}

/// **Move one widget by a page-space delta.**
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

#[cfg(test)]
mod tests;
