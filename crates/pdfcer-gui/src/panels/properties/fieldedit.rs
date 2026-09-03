//! # `panels::properties::fieldedit` — a placed form field's properties, and
//! the controls that change them
//!
//! `Pass 134.0`'s `EditSession::edit_field`, consumed 2026-08-27.
//!
//! ## ★★★ The sentence this module deletes
//!
//! [`super::formfield`] shipped on 2026-08-26 showing a field's flags as
//! **read-only facts**, under a sentence the operator actually read:
//!
//! > ~~Required, read-only, the tooltip and the border can only be set when a
//! > field is placed. **To change one, delete this field and place a new one.**~~
//!
//! `EditSession::edit_field` landed the **same day**, three commits before the
//! revision this shell compiles against, and the engine wrote a full pane
//! design brief into the request channel saying so. Nothing consumed it.
//!
//! So the program spent a day telling an operator to perform a **destructive
//! workaround** for a capability it already had — and delete-and-replace is
//! genuinely destructive: it loses the field's name, its filled value and its
//! place in the tab order, every one of which an FDF import or a filling script
//! keys on.
//!
//! ★ The lesson is not *grep harder*. The claim was **true when it was
//! written** and false within hours, because it was an absence claim about a
//! crate this project does not build. Such a claim has a shelf life. What
//! catches it is reading the reply, and the reply was sitting unread in
//! `open/`.
//!
//! ## ★★★ SCOPE — field or widget — and getting it backwards is invisible
//!
//! The engine took this verbatim from Acrobat's own scripting model, and it is
//! the decision that shapes the whole pane: some properties *"apply to all
//! widgets that are children of that field"*, others *"are specific to
//! individual widgets"*.
//!
//! | scope | verb | properties |
//! |---|---|---|
//! | **field** — one write, every placement | `edit_field` | required, read-only, tooltip, multiline, password, comb, max-len, no-toggle-to-off, radios-in-unison, combo, editable, multi-select, sort, options |
//! | **widget** — per placement | `edit_widget` | rect, border, visibility, caption |
//!
//! > **Getting this backwards is invisible on the ordinary one-widget field
//! > and wrong on every radio group** — where "the border" can only sensibly
//! > mean one button and "required" can only sensibly mean the group.
//!
//! This module holds the **field** half. The widget half is the next piece of
//! work and is named in `FEATURES.md` rather than left as a silence.
//!
//! ## ★★ One press is one undo entry, and one exception the standard forces
//!
//! Every control here sends a `FieldEdit` naming **one** property, though the
//! struct can carry fourteen. That is `StyleChange`'s rule for its reason: a
//! pane that batched a flag and a max-length into one request would make
//! `Ctrl+Z` after two presses take back a state the operator never saw.
//!
//! ★ The exception is not a batch. Table 228 permits `Comb` only when
//! `/MaxLen` is present, and the engine checks its gates **against the
//! resulting field** — so turning comb on for a field with no max-length must
//! send both in one edit or be refused. That is one act the standard makes
//! indivisible, not two the pane chose to combine.
//!
//! ## ★★ The refusals arrive from a direction the request does not name
//!
//! The engine's §6, and it is the part most likely to produce a confusing
//! message:
//!
//! * `.with_max_len(None)` on a **comb** field → `CombPreconditionUnmet`,
//!   naming comb, which the request never mentioned;
//! * `.with_combo(false)` on an **editable** drop-down → `ChoiceEditWithoutCombo`.
//!
//! Its instruction: *"show it against the control the operator touched, not the
//! one the standard named."* So every action carries a `touched` label, and the
//! decline names the control that was pressed.
//!
//! ## There is no type change, and there is nothing to grey
//!
//! Acrobat has offered no field-type conversion since Acrobat 6; the only route
//! is delete-and-recreate. pdfcer models the same limit by making the request
//! **unrepresentable** rather than by returning an error, so this pane has
//! nothing to disable — the property does not exist. Said here because "why is
//! there no Type control" is the obvious question and an absence with no
//! explanation is indistinguishable from an oversight.
//!
//! ## Rule 4
//!
//! Nothing here marks the canvas. Every disclosure — a value that no longer
//! fits its own limit, a `Sort` claim the list does not meet, three widgets
//! changed by one write — lands in the status bar through
//! `app::actions::forms`. The field renders exactly as the saved file will
//! render it.

use egui::Ui;
use pdfcer_core::edit::FieldEdit;
use pdfcer_core::forms::{Field, FieldFlags, FieldType};

use crate::app::actions::Action;
use crate::app::actions::forms::FieldAction;
use crate::panels::PanelsState;
use crate::text::panels::formfield as t;

/// The section's rect, for `ui-verify`.
// ui-text-exempt: trace region name, never displayed
pub const REGION: &str = "properties.field_edit";
/// The Required checkbox's own region.
///
/// ★ Published per control rather than leaving a driven check to divide
/// [`REGION`] by eye — `panels::properties::text`'s note on this is the
/// precedent, and the reason is that a check computing a control's position
/// from a section's bounds passes on a build where the controls moved.
// ui-text-exempt: trace region name, never displayed
pub const REQUIRED_REGION: &str = "properties.field_edit.required";
/// The Max length field's own region.
// ui-text-exempt: trace region name, never displayed
pub const MAX_LEN_REGION: &str = "properties.field_edit.max_len";

/// Draw the editable properties of the selected field.
///
/// Returns whether it drew, which is always `true` when a field is selected:
/// **every** field type has required, read-only and a tooltip, so there is no
/// field for which this section is empty. The type-specific rows come and go.
pub fn section(
    ui: &mut Ui,
    field: &Field,
    fqn: &str,
    state: &mut PanelsState,
    epoch: u64,
    actions: &mut Vec<Action>,
) -> bool {
    let draft = state.field_props_mut();
    draft.read(field, fqn, epoch);

    ui.label(t::editable_heading());
    ui.add_space(2.0);

    // -- Every type ---------------------------------------------------------
    flag_row(
        ui,
        t::flag_required(),
        t::flag_required_hover(),
        field.flags.required(),
        REQUIRED_REGION,
        fqn,
        // ui-text-exempt: a control name carried for a refusal message.
        "required",
        FieldEdit::new,
        |edit, on| edit.with_required(on),
        actions,
    );
    flag_row(
        ui,
        t::flag_read_only(),
        t::flag_read_only_hover(),
        field.flags.read_only(),
        // ui-text-exempt: trace region name, never displayed
        "properties.field_edit.read_only",
        fqn,
        // ui-text-exempt: a control name carried for a refusal message.
        "read only",
        FieldEdit::new,
        |edit, on| edit.with_read_only(on),
        actions,
    );

    // -- Text fields --------------------------------------------------------
    //
    // ★ Gated on the field's TYPE, and the gate is not cosmetic: `/Ff` is one
    // shared 32-bit word whose bits mean **different things per type** — bit 26
    // is `RadiosInUnison` on a `/Btn` and `RichText` on a `/Tx` — so a
    // mis-typed edit does not do nothing, it does something else. The engine
    // refuses it by name (`FieldPropertyTypeMismatch`), and a pane that drew
    // the control anyway would be offering a press that always fails.
    if matches!(field.field_type, Some(FieldType::Text)) {
        flag_row(
            ui,
            t::flag_multiline(),
            t::flag_multiline_hover(),
            field.flags.has(FieldFlags::MULTILINE),
            // ui-text-exempt: trace region name, never displayed
            "properties.field_edit.multiline",
            fqn,
            // ui-text-exempt: a control name carried for a refusal message.
            "multi-line",
            FieldEdit::new,
            |edit, on| edit.with_multiline(on),
            actions,
        );
        flag_row(
            ui,
            t::flag_password(),
            t::flag_password_hover(),
            field.flags.has(FieldFlags::PASSWORD),
            // ui-text-exempt: trace region name, never displayed
            "properties.field_edit.password",
            fqn,
            // ui-text-exempt: a control name carried for a refusal message.
            "hidden as typed",
            FieldEdit::new,
            |edit, on| edit.with_password(on),
            actions,
        );
        max_len_row(ui, field, fqn, state, actions);
        comb_row(ui, field, fqn, state, actions);
    }

    // -- Radio buttons ------------------------------------------------------
    if matches!(field.field_type, Some(FieldType::Button)) && field.widgets.len() > 1 {
        flag_row(
            ui,
            t::flag_no_toggle_off(),
            t::flag_no_toggle_off_hover(),
            field.flags.has(FieldFlags::NO_TOGGLE_TO_OFF),
            // ui-text-exempt: trace region name, never displayed
            "properties.field_edit.no_toggle_off",
            fqn,
            // ui-text-exempt: a control name carried for a refusal message.
            "cannot be turned off",
            FieldEdit::new,
            |edit, on| edit.with_no_toggle_to_off(on),
            actions,
        );
    }

    // -- Choice fields ------------------------------------------------------
    if matches!(field.field_type, Some(FieldType::Choice)) {
        flag_row(
            ui,
            t::flag_combo(),
            t::flag_combo_hover(),
            field.flags.has(FieldFlags::COMBO),
            // ui-text-exempt: trace region name, never displayed
            "properties.field_edit.combo",
            fqn,
            // ui-text-exempt: a control name carried for a refusal message.
            "drop-down",
            FieldEdit::new,
            |edit, on| edit.with_combo(on),
            actions,
        );
        flag_row(
            ui,
            t::flag_multi_select(),
            t::flag_multi_select_hover(),
            field.flags.has(FieldFlags::MULTI_SELECT),
            // ui-text-exempt: trace region name, never displayed
            "properties.field_edit.multi_select",
            fqn,
            // ui-text-exempt: a control name carried for a refusal message.
            "multi-select",
            FieldEdit::new,
            |edit, on| edit.with_multi_select(on),
            actions,
        );
    }

    tooltip_row(ui, fqn, state, actions);

    // ★★★ `ui_rect`, NOT `ui_rect_visible` — and the difference is a finding
    // rather than a preference, measured by driving on 2026-08-27.
    //
    // `ui_rect_visible` publishes a region only when at least 60 % of it lies
    // inside the clip. That threshold exists for an excellent reason
    // (`diag::VISIBLE_FRACTION`: a settings heading two points inside a scroll
    // area's bottom edge measured 1.53:1, because the sampler was reading the
    // anti-aliased tops of glyphs whose bodies had been clipped away) and the
    // reason is about **sampling a surface**.
    //
    // A SECTION rect is not a surface anybody samples. It answers *"did this
    // draw?"* and *"where can I scroll?"*, and it is **taller than the panel's
    // slot by construction** — this section alone is seven controls. So gating
    // it on visibility makes it vanish from the trace exactly when the section
    // is long, which is always. The first driven run of this feature reported
    // the controls present and their enclosing section absent.
    //
    // ⇒ The rule, and it is now in `D:/dev/rag/egui/`: **`ui_rect_visible` for
    // a control a check will CLICK or SAMPLE; `ui_rect` for a section a check
    // will ask a yes/no question about.** Every per-control region above takes
    // the visible form, because a check clicks those and clicking a sliver
    // lands on whatever is genuinely at those coordinates. This one does not.
    //
    // `super::formfield`'s own `REGION` already used the plain form, which is
    // why it survived the same run — that was luck rather than a decision, and
    // this comment is the decision.
    crate::diag::ui_rect(REGION, ui.min_rect());
    true
}

/// One boolean property, as a checkbox that commits on click.
///
/// # ★★ It reads its state from the DOCUMENT, not from a draft
///
/// The `checked` argument is `field.flags`, re-read every frame from the
/// session. There is no local copy to go stale, and the visible consequence is
/// the right one: a press that the engine **refuses** leaves the box where it
/// was, because the document did not change. A draft-backed checkbox would
/// show the operator's intent and the document would disagree with it silently
/// — which is the "the control does nothing" report with an extra step.
///
/// `PanelsState` therefore holds no boolean for any of these, and the two
/// controls that *do* need a draft — the tooltip and the max-length — are the
/// two that take typing.
#[allow(clippy::too_many_arguments)]
fn flag_row(
    ui: &mut Ui,
    label: &str,
    hover: &str,
    checked: bool,
    region: &str,
    fqn: &str,
    touched: &'static str,
    new: fn() -> FieldEdit,
    set: fn(FieldEdit, bool) -> FieldEdit,
    actions: &mut Vec<Action>,
) {
    let mut on = checked;
    let response = ui.checkbox(&mut on, label);
    crate::diag::ui_rect_visible(region, response.rect, ui.clip_rect());
    if response.on_hover_text(hover).changed() {
        actions.push(
            FieldAction::EditProperties {
                field: fqn.to_owned(),
                edit: set(new(), on),
                touched,
            }
            .into(),
        );
    }
}

/// `/MaxLen` — the outer option is *touched or not*, the inner *present or
/// absent*.
///
/// ★ Zero means **absent**, and it is spelled that way rather than with a
/// separate "limit the length" checkbox, because a spinner at zero and an
/// unchecked box beside a greyed spinner say the same thing and the second
/// costs a control. `/MaxLen` of zero is not meaningful in a file — a field
/// that accepts no characters is not a field — so the value is free to carry
/// the absence.
fn max_len_row(
    ui: &mut Ui,
    field: &Field,
    fqn: &str,
    state: &mut PanelsState,
    actions: &mut Vec<Action>,
) {
    let draft = state.field_props_mut();
    ui.horizontal(|ui| {
        ui.label(t::label_max_len());
        let response = ui.add(
            egui::DragValue::new(&mut draft.max_len)
                .speed(1.0)
                .range(0..=32_767),
        );
        crate::diag::ui_rect_visible(MAX_LEN_REGION, response.rect, ui.clip_rect());
        let response = response.on_hover_text(t::label_max_len_hover());
        // ★ Committed on release or on losing focus, never on `.changed()`.
        // Each commit is one `edit_field` and one undo entry, so a drag across
        // the spinner would author one edit per pixel — the same rule the text
        // style and markup width rows follow.
        if response.drag_stopped() || response.lost_focus() {
            let want = (draft.max_len > 0).then_some(draft.max_len);
            let have = field.max_len;
            if want != have {
                actions.push(
                    FieldAction::EditProperties {
                        field: fqn.to_owned(),
                        edit: FieldEdit::new().with_max_len(want),
                        // ui-text-exempt: a control name carried for a refusal message.
                        touched: "maximum length",
                    }
                    .into(),
                );
            }
        }
    });
}

/// Comb — the one control whose edit is **two** properties, because the
/// standard makes it indivisible.
///
/// ★★ Table 228 permits `Comb` only when `/MaxLen` is present, and the engine
/// checks its gates **against the resulting field** rather than against the
/// request. So turning comb on for a field with no max-length must send both or
/// be refused with `CombPreconditionUnmet` — a refusal naming a property the
/// operator never touched.
///
/// The pane sends both. `.with_comb(true).with_max_len(Some(n))` is explicitly
/// accepted by the engine, and `n` is whatever the spinner above holds, or a
/// default when it holds nothing: a comb field needs a cell count and there is
/// no honest way to have one without it.
///
/// ★ This is **not** a violation of "one press, one undo entry". It is one act
/// that the standard defines as two writes, which is a different thing from a
/// pane choosing to batch two acts.
fn comb_row(
    ui: &mut Ui,
    field: &Field,
    fqn: &str,
    state: &mut PanelsState,
    actions: &mut Vec<Action>,
) {
    let held = state.field_props_mut().max_len;
    let mut on = field.flags.has(FieldFlags::COMB);
    let response = ui.checkbox(&mut on, t::flag_comb());
    crate::diag::ui_rect_visible(
        // ui-text-exempt: trace region name, never displayed
        "properties.field_edit.comb",
        response.rect,
        ui.clip_rect(),
    );
    if response.on_hover_text(t::flag_comb_hover()).changed() {
        let mut edit = FieldEdit::new().with_comb(on);
        if on && field.max_len.is_none() {
            // ★ The spinner's value when it has one, and `DEFAULT_COMB_CELLS`
            // when it does not. Never zero: `.with_comb(true)` plus
            // `.with_max_len(None)` is the refusal this branch exists to avoid,
            // and it would be a press that always fails.
            let cells = if held > 0 { held } else { DEFAULT_COMB_CELLS };
            edit = edit.with_max_len(Some(cells));
        }
        actions.push(
            FieldAction::EditProperties {
                field: fqn.to_owned(),
                edit,
                // ui-text-exempt: a control name carried for a refusal message.
                touched: "equal cells",
            }
            .into(),
        );
    }
}

/// The cell count a comb field gets when it is turned on with no `/MaxLen`.
///
/// ★ Ten, and the number is a **disclosed guess** rather than a right answer:
/// there is no way to know how many cells the operator wants, `/MaxLen` is
/// mandatory for comb, and refusing the press would mean a checkbox that
/// cannot be ticked on the majority of text fields. Ten is a postcode, a phone
/// number and a part number, and it is immediately editable in the spinner
/// directly above the control that set it — which is what makes a guess
/// acceptable here and not elsewhere.
const DEFAULT_COMB_CELLS: i64 = 10;

/// `/TU`, the accessibility name.
///
/// ★★ A draft and a button, not a live write — `super::formfield`'s rename row
/// makes the argument and it is identical here: a `TextEdit` bound straight to
/// the field would author one `edit_field` per keystroke, each one a real,
/// separately undoable change.
///
/// ★ **Empty commits `TooltipChoice::Declined`, which REMOVES `/TU`**, and that
/// is the engine's instruction rather than this pane's choice: *"an empty `/TU`
/// would be worse than none, because a screen reader announces the empty name
/// instead of falling back to the field's."*
fn tooltip_row(ui: &mut Ui, fqn: &str, state: &mut PanelsState, actions: &mut Vec<Action>) {
    ui.label(t::label_tooltip());
    let draft = state.field_props_mut();
    let response = ui.add(
        egui::TextEdit::singleline(&mut draft.tooltip)
            .desired_width(f32::INFINITY)
            .hint_text(t::label_tooltip_hint()),
    );
    let typed = draft.tooltip.trim().to_owned();
    let stored = draft.tooltip_stored.clone();
    let lost = response.lost_focus();
    if lost && typed != stored {
        let choice = if typed.is_empty() {
            pdfcer_core::edit::TooltipChoice::Declined
        } else {
            pdfcer_core::edit::TooltipChoice::Text(typed)
        };
        actions.push(
            FieldAction::EditProperties {
                field: fqn.to_owned(),
                edit: FieldEdit::new().with_tooltip(choice),
                // ui-text-exempt: a control name carried for a refusal message.
                touched: "tooltip",
            }
            .into(),
        );
    }
}

/// What the two typed controls hold, and the field they were read for.
///
/// # ★ Why only two properties have a draft
///
/// Because only two take typing. Every checkbox reads `field.flags` straight
/// from the session each frame, which is what makes a refused press leave the
/// box where it was — see [`flag_row`]. A draft for a boolean would show the
/// operator's intent while the document disagreed with it, silently.
#[derive(Default)]
pub struct FieldPropsDraft {
    /// `(fully-qualified name, edit epoch)` the values below were read at.
    stamp: Option<(String, u64)>,
    /// `/MaxLen`, with **0 meaning absent** — see [`max_len_row`].
    max_len: i64,
    /// The tooltip being typed.
    tooltip: String,
    /// The tooltip as the document holds it, so a commit can tell whether the
    /// operator changed anything.
    ///
    /// ★ Its own field rather than a re-read, because the commit happens on
    /// `lost_focus` — a frame in which the draft has already been typed into
    /// and the document has not changed. Comparing the draft against a fresh
    /// read would work; keeping the read is what makes the comparison
    /// obviously against *the value this draft was seeded from*.
    tooltip_stored: String,
}

impl FieldPropsDraft {
    /// Re-read from the document when the stamp has moved.
    ///
    /// # ★★ The stamp is the name AND the epoch, and both are load-bearing
    ///
    /// **The name**, because clicking a second field must not leave the first
    /// field's tooltip sitting in the box waiting to be applied to the wrong
    /// one — the failure `super::formfield`'s rename draft stores a key to
    /// prevent, and the reason that draft stores a key at all.
    ///
    /// **The epoch**, because a property edit is an edit: after committing a
    /// max-length the document holds a new value, and a draft that did not
    /// re-read would show the pre-edit number for ever after the first change.
    /// That is the failure that makes a properties panel untrustworthy, and it
    /// is the same term `super::text`'s `TextStyleDraft` carries for the same
    /// reason.
    ///
    /// ★ Note what a re-read costs here and does not cost there: this is a
    /// field lookup in an already-parsed `AcroForm`, not a text extraction with
    /// provenance. The stamp is for **correctness** — not leaking one field's
    /// draft onto another — rather than for the 392 ms the text draft is
    /// avoiding.
    /// ★★ It takes the two VALUES, not the `&Field` it used to.
    ///
    /// The refactor was forced by a test and is right on its own: `Field` has
    /// no `Default`, so a unit test could not build one without a document —
    /// and this function reads exactly two things off it. Taking those two
    /// makes the dependency honest and the staleness rule testable in
    /// isolation, which is the same move `app::actions::forms::disclosures`
    /// made and for the same stated reason.
    ///
    /// [`Self::read`] is the one place the two are pulled off a real field, so
    /// there is still exactly one statement of *where a tooltip lives*.
    fn sync(&mut self, max_len: Option<i64>, tooltip: String, fqn: &str, epoch: u64) {
        let stamp = (fqn.to_owned(), epoch);
        if self.stamp.as_ref() == Some(&stamp) {
            return;
        }
        self.stamp = Some(stamp);
        self.max_len = max_len.unwrap_or(0);
        self.tooltip_stored = tooltip;
        self.tooltip.clone_from(&self.tooltip_stored);
    }

    /// Pull the two typed values off a real field, and sync.
    ///
    /// ★ `/TU` is `alternate_name`, **not** a field called `tooltip`. The
    /// standard's own name for it is *alternate field name*; every application
    /// calls it the tooltip because that is where a reader shows it and what a
    /// screen reader announces. Raw bytes (§7.9.2), decoded the way every other
    /// operator-visible name in this crate is.
    fn read(&mut self, field: &Field, fqn: &str, epoch: u64) {
        let tooltip = field
            .alternate_name
            .as_deref()
            .map(|raw| String::from_utf8_lossy(raw).into_owned())
            .unwrap_or_default();
        self.sync(field.max_len, tooltip, fqn, epoch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★ **A draft seeded from one field does not survive onto another.**
    ///
    /// The failure this stamp exists for, and it is the expensive one: the
    /// operator types a tooltip, clicks a different field without committing,
    /// and the pane is now holding the first field's text over the second
    /// field's name. Pressing Enter would write it to the wrong field, and
    /// nothing on screen would have said so.
    #[test]
    fn a_draft_is_reseeded_when_the_selection_moves() {
        let mut draft = FieldPropsDraft::default();

        draft.sync(Some(8), "first".to_owned(), "A", 0);
        assert_eq!(draft.tooltip, "first");
        assert_eq!(draft.max_len, 8);

        // The operator types without committing…
        draft.tooltip = "half typed".to_owned();
        // …and clicks a different field.
        draft.sync(None, "second".to_owned(), "B", 0);
        assert_eq!(
            draft.tooltip, "second",
            "the half-typed tooltip must not survive onto another field"
        );
        assert_eq!(draft.max_len, 0, "absent /MaxLen reads as zero");
    }

    /// ★★ **An edit to the same field re-reads it**, which is the term that
    /// stops the pane showing a stale value for ever.
    ///
    /// Without the epoch in the stamp, committing a max-length would leave the
    /// spinner showing the number it had *before* the commit — and it would
    /// stay there, because the name has not changed. The operator would see
    /// their own edit not take.
    #[test]
    fn an_edit_to_the_same_field_reseeds_the_draft() {
        let mut draft = FieldPropsDraft::default();
        draft.sync(Some(8), String::new(), "A", 0);
        assert_eq!(draft.max_len, 8);

        // Same name, same epoch — the pane has not been told anything changed.
        draft.sync(Some(12), String::new(), "A", 0);
        assert_eq!(draft.max_len, 8, "no epoch change, no re-read");

        draft.sync(Some(12), String::new(), "A", 1);
        assert_eq!(
            draft.max_len, 12,
            "the epoch moved, so the value is re-read"
        );
    }
}
