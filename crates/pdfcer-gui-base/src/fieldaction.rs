//! # `fieldaction` — the verbs whose subject is a form field

/// One thing done to a form field, carried by
/// `pdfcer_gui::app::actions::Action::Field`.
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
    /// the same way `pdfcer_gui::app::actions::Action::DeleteSelection` carries its operand list.
    ///
    /// # Why the arm below is one line and not four
    ///
    /// It does not go through `pdfcer_gui::app::actions::apply::vector_edit`, and `crate::panels::forms::edit`'s
    /// own header carries the reason: the six form outcome types do not unify
    /// into `Result<Vec<String>, EditError>`, so that module performs the
    /// cancel-mutate-bump-invalidate protocol itself, once, for all of them.
    /// A second copy of the protocol here would be the fifth hand-written
    /// instance of a four-step sequence `vector_edit` exists to have exactly
    /// one of.
    Edit(crate::formedit::FormEdit),
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
    Select(Option<crate::docidentity::SelectedField>),
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
    /// **Ask for a picture and make it a push button's icon**, `/MK /I`.
    ///
    /// Carries no image: the picker opens in the apply phase, as
    /// `Action::ExportFormData`'s does, because a native modal opened from a
    /// widget's `clicked()` blocks mid-frame.
    PickButtonIcon {
        /// The button's fully-qualified name.
        field: String,
        /// Which placement, indexing `Field::widgets`.
        widget: usize,
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
    /// Write or remove a page's `/Tabs` entry —
    /// `EditSession::set_page_tabs`. The engine refuses a PDF 2.0 value below
    /// 2.0, a value PDF/UA forbids, and a value ISO 32000 does not define.
    SetPageTabs {
        /// The page, 0-based.
        page: usize,
        /// What to write; `PageTabs::Absent` removes the entry.
        tabs: pdfcer_core::edit::PageTabs,
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
    /// See `groups` for the two-press protocol, why the answer is kept in a
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
    /// `pdfcer_gui::app::actions::Action::BeginTextAnnot` gives at length: the rectangle is a choice the
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
        kind: crate::formfieldkind::FormFieldKind,
        /// The rectangle, in PDF user space, already normalised.
        rect: pdfcer_core::page_tree::Rect,
    },
    /// **Sign into this empty signature field** — a click on its box on the
    /// page. Opens the *Sign here* window on that box; writes nothing.
    Sign {
        /// The field's fully-qualified name.
        field: String,
        /// The 0-based page the clicked box is on.
        page: usize,
        /// The clicked box, in canvas space.
        rect: egui::Rect,
    },
    /// **Sign this field with a digital ID instead** — the certificate route,
    /// chosen from the *Sign here* window. Opens the Sign window on the field.
    SignWithId {
        /// The field's fully-qualified name.
        field: String,
    },
    /// **Write a hand signature, drawn or typed, into the page, inside this
    /// box.** One undo step; the `/Sig` field itself is left empty.
    HandSign {
        /// The field's fully-qualified name, for the session's ledger.
        field: String,
        /// The 0-based page.
        page: usize,
        /// The box, in canvas space.
        rect: egui::Rect,
        /// The signature; a drawn one is normalised.
        signature: crate::handsign::Signature,
    },
    /// **Author the form control the dialog just accepted.**
    ///
    /// Raised by `crate::dialogs::formfield` and by nothing else. This is the
    /// one that reaches the document, through the same `vector_edit` funnel
    /// every other authoring verb uses.
    ///
    /// The whole draft travels, for the reason `pdfcer_gui::app::actions::Action::CommitTextAnnot`
    /// states: by the time the queue drains the dialog is closed and its fields
    /// are gone, so reading them at apply time is not fragile but impossible.
    Commit {
        /// The 0-based page.
        page: usize,
        /// The rectangle, in PDF user space, already normalised.
        rect: pdfcer_core::page_tree::Rect,
        /// Everything the operator chose, including which kind it is.
        draft: Box<crate::formdraft::Draft>,
    },
    /// **Author the form control that came off the clipboard.**
    ///
    /// Raised by `pdfcer_gui::canvas::fieldclip::paste` and by nothing else.
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
    /// is settled entirely by `pdfcer_gui::canvas::formfield::Draft::name` before
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
        /// `pdfcer_gui::canvas::fieldclip::paste`'s same-page/cross-page rule.
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
    /// The same reason [`crate::subactions::BookmarkAction::Add`]'s parent is: a position is
    /// invalidated by the edit itself. Registering a widget moves it out of the
    /// unclaimed list and into the rows, so a second registration keyed on
    /// "the second unclaimed box" would act on a different box than the one the
    /// operator pressed beside. `adopt_widget` takes an id for this reason and
    /// the listing carries one for the same reason.
    ///
    /// # Why the page travels with it
    ///
    /// Only for the funnel: `pdfcer_gui::app::actions::apply::vector_edit` wants a page for its trace
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
