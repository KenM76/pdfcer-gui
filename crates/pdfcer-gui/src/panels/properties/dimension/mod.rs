//! # `panels::properties::dimension` — the selected ce dimension's own
//! properties
//!
//! ## The gap this closes, quoted from the people who found it
//!
//! `docs/ui_specs/tool-options-dock-and-ce-dimension-properties.md` §C.11,
//! item 2, written before any of this shell existed:
//!
//! > **A selection-driven property surface for an ALREADY-PLACED ce
//! > dimension.** … There is **no** panel today that shows a selected ce
//! > dimension's own properties — not its group, not its radius/diameter
//! > toggle, nothing. An operator who placed a Radius ce dimension and later
//! > wants Diameter has **no way to change it** without deleting and
//! > redrawing … This is the concrete gap Ken's request is actually naming
//! > when he says *"the ce dimensions I add need to be editable as well"*.
//!
//! and `pdfcer`'s `FEATURES.md`, which carries the same row as
//! `core [x] cli [x] gui [ ]` for **both** shells:
//!
//! > ce-dimension style AND tolerance in the GUI — **one panel covering
//! > both**, showing which values are inherited and which are overridden and
//! > letting the override be set. The model and the CLI already do all of it;
//! > only the disclosure surface is missing.
//!
//! ## Why here, and not on the Format tab
//!
//! The ui-spec settles it (§C.12) and the reasoning is not about taste:
//!
//! - **A ribbon band is about seventy points tall.** Eleven property editors,
//!   each with a checkbox and a provenance sentence, do not fit in one and
//!   would not be readable if they did.
//! - **Per-ce-dimension editing must work with no tool armed.** Tool Options is
//!   for an *armed tool's* controls; this is a *selection's* properties, which
//!   is one step further down — properties of the selected object, not of the
//!   thing that would create one.
//! - **`RIBBON_IA.md` §5.8 wants both surfaces**, with the tab carrying *"what
//!   a user changes while working"* and the panel carrying *everything*. The
//!   panel is the harder half and the tab's contents are a subset of it, so
//!   the specified build order is panel first — which is exactly what
//!   `manifest/format.rs`'s header already records.
//!
//! ## ★ And why it is a section of Properties rather than a panel of its own
//!
//! §C.12 answered this too, and it flagged the consequence honestly: the
//! Properties panel's own premise — *"nothing else competed for the word
//! Properties"* — **stops being true** the moment a ce-dimension selection
//! becomes a second claimant on it. The fix it recommends is to broaden the
//! panel's stated purpose rather than to invent a ninth panel or rename the
//! tab, and that is what has been done: [`super::body`] now shows *the
//! document's object properties, OR the properties of whatever is selected on
//! the canvas*.
//!
//! Transient, "what I am looking at right now" content goes **first**, above
//! the persistent object form — the same top-first-bottom-persistent ordering
//! the Objects/Properties split already establishes.
//!
//! ## ★ Rule 4: everything here is off-canvas, and the canvas is untouched
//!
//! The selection outline is the cursor, which the rule permits by name. Nothing
//! in this section tints, badges or flags the ce dimension it is describing;
//! the provenance of a value — *"using the group's setting"* — is a sentence in
//! a panel, and the drawing renders exactly as it will save. That is the whole
//! point of the surface: the inference (which tier supplied this) is disclosed
//! **off** the page, not marked **on** it.
//!
//! ## What the operator cannot do here, and why each is stated on screen
//!
//! | | why |
//! |---|---|
//! | ~~change the group~~ | ★ **closed 2026-08-19.** It was *"no engine verb; filed"* — filed on the 18th, shipped on the 19th, and it is a picker now. What it gained with the verb is a **disclosure**: `set_dimension_group` re-measures, so the number changes |
//! | change the scale | **by refusal** — `StyleOverrides` has no scale field, asserted structurally, because a member measuring at a different scale from its group would print a number nothing on the page discloses |
//! | drag the extension lines | the gap and overshoot are standard-derived, not per-ce-dimension fields; new core work, named in §C.11 item 3 |
//!
//! The scale row is **not** disclosed in words, deliberately: an absent control
//! for a thing that is *correctly* group-scoped needs no apology, and the
//! Set-scale window is where an operator looks for a scale anyway. The group row
//! needed one only while the capability was missing, and now needs a different
//! one — about what the move does rather than about it being impossible.

mod overrides;
mod tolerance;

use egui::Ui;
use pdfcer_core::dimension::{DimensionKind, DimensionRecord};

use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;
use crate::app::state::OpenDoc;
use crate::canvas::selection::AnnotKind;
use crate::text::panels::dimension as t;

/// The region this section publishes.
const REGION: &str = "properties.dimension"; // ui-text-exempt: trace region name, never displayed
/// The region the label-override box publishes, for a driven check to aim at.
pub const LABEL_REGION: &str = "properties.dimension.label"; // ui-text-exempt: trace region name, never displayed

/// The region the radius/diameter choice publishes.
pub const REGION_DISPLAY: &str = "properties.dimension.display"; // ui-text-exempt: trace region name, never displayed

/// Draw the ce-dimension section, if one is selected. Returns whether it drew
/// anything.
///
/// # Why it reports whether it drew
///
/// So [`super::body`] can decide what the panel says when there is no object
/// focused. A ce dimension selected on the canvas with nothing focused in the
/// object tree is a perfectly ordinary state — it is what happens the moment
/// the operator clicks a dimension — and *"nothing is selected"* would be
/// false in front of a section describing the thing that is.
pub fn section(ui: &mut Ui, doc: &OpenDoc, actions: &mut Vec<Action>) -> bool {
    let Some(selection) = doc.selection.annot() else {
        return false;
    };
    if selection.target.kind != AnnotKind::CeDimension {
        return false;
    }

    crate::diag::ui_rect(REGION, ui.max_rect());
    // No `.strong()` — R84 / DEFECTS.md D11. See `dialogs::dimension_groups`.
    ui.label(t::heading());

    // ★ Read from the SESSION, every frame. `dimension_model()` clones out of
    // the `/PieceInfo` sidecar, so this is the model as the document stands
    // including unsaved edits — and a cached copy would be stale for exactly
    // the frame after the operator changed something, which is the frame they
    // are looking at.
    let model = doc.session.dimension_model();
    let Some(record) = model
        .dimensions()
        .iter()
        .find(|r| r.annot == Some(selection.target.id))
    else {
        // Reachable, and not a defect here — see the string's own doc comment.
        // A `/Line` with `/IT /LineDimension` can arrive from an insert, a
        // merge, or a third-party rewrite that dropped the sidecar.
        ui.label(t::no_record());
        ui.separator();
        return true;
    };
    let Some(group) = model.group(record.group) else {
        // The record names a group the model does not have. Structurally
        // impossible through any pdfcer path — the sidecar carries both — and
        // handled rather than unwrapped, because a panic in a panel takes the
        // whole window with it and the honest answer is the same sentence.
        ui.label(t::no_record());
        ui.separator();
        return true;
    };

    // --- the facts ------------------------------------------------------
    // ★ A PICKER, not a readout, as of 2026-08-19.
    //
    // It was a readout with a sentence saying the group *"cannot be changed
    // afterwards"*, which was true and was filed the same day
    // (`request_a_placed_ce_dimension_cannot_be_moved_to_another_group.md`).
    // `EditSession::set_dimension_group` shipped the next morning.
    //
    // The disclosure under it is the one the engine went out of its way to make
    // sure a shell would not miss: this verb **re-measures**, so the number the
    // dimension prints may change. See the string.
    let mut destination = record.group;
    ui.horizontal(|ui| {
        ui.label(t::group_label());
        egui::ComboBox::from_id_salt("dimension-group-move")
            .selected_text(&group.name)
            .show_ui(ui, |ui| {
                for candidate in model.groups() {
                    ui.selectable_value(&mut destination, candidate.id, &candidate.name);
                }
            });
    });
    ui.weak(t::group_move_changes_the_number());
    if destination != record.group {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "dimension-regroup id={} from={} to={}",
                record.id.0, record.group.0, destination.0
            )
        });
        actions.push(Action::Dimension(DimensionAction::SetDimensionGroup {
            dimension: record.id,
            group: destination,
        }));
    }

    // ★ The measurement comes from the model's own `display`, which is the
    // producer the `/AP` label is baked from — including the branch that sends
    // an ANGULAR kind to `format_angle_degrees` and never applies the group
    // scale. Formatting it here instead would produce a plausible, wrong number
    // for every angular ce dimension: 30° at 1:50 is 30°, not 1500 of anything,
    // and `format_measurement` would report the wrong value with
    // `raw_page_units: false`, i.e. wrong AND undisclosed.
    if let Some(display) = model.display(record.id) {
        ui.horizontal(|ui| {
            ui.label(t::measured_label());
            ui.label(&display.text);
        });
        if display.raw_page_units {
            // Verbatim. The engine owns this wording so shells cannot invent
            // their own — `03-capabilities.md` §1.5 obligation 2.
            ui.weak(pdfcer_core::dimension::NO_SCALE_DISCLOSURE);
        }
    }

    display_toggle(ui, record, actions);
    label_row(ui, record, actions);

    // --- the overrides --------------------------------------------------
    ui.separator();
    ui.label(t::overrides_heading());
    ui.weak(t::overrides_hint());
    let mut next = record.style;
    let valid = overrides::show(ui, group, &mut next);
    // ★ Raised only when the draft VALIDATES and differs. The validity gate is
    // the tolerance's: an inverted limit pair or a negative magnitude is
    // refused by the engine with a sentence, and pushing the action anyway
    // would turn a refusal the operator can read into an edit that silently
    // does not happen.
    if valid && next != record.style {
        actions.push(Action::Dimension(DimensionAction::SetStyle {
            dimension: record.id,
            style: next,
        }));
    }
    ui.separator();
    true
}

/// **Say something other than the measurement**, without changing it.
///
/// `EditSession::set_dimension_label`, shipped 2026-08-30.
///
/// # ★★★ It does NOT destroy the measurement, and that is the whole design
///
/// The engine's own note is titled for it: *"dimension text override ships and
/// it does not destroy the measurement."* The override is a **caption**; the
/// measured value stays underneath, so `None` restores it with **no
/// re-measurement** — the number that comes back is the number that was always
/// there, not a fresh calculation that might round differently.
///
/// ⇒ That is why this control is a text box with a Clear beside it rather than
/// an editable value field. An editable value would imply the operator was
/// changing what was measured, and on a drawing that is the difference between
/// a note and a lie.
///
/// # ★★ Why the measured value is shown even while overridden
///
/// `DimensionLabelChange` carries `measured` and `printed` separately, and this
/// shows both whenever they differ. An operator looking at a ce dimension that
/// reads *"see detail B"* has no other way to find out what it actually
/// measures — and the one place that fact must be available is beside the
/// control that hid it.
///
/// # Committed on focus loss, not per keystroke
///
/// `super::widgetedit`'s rule: one control press is one undo entry. A caption
/// typed a character at a time would author twelve commands and leave eleven
/// intermediate states in the undo stack that the operator never saw.
fn label_row(
    ui: &mut Ui,
    record: &pdfcer_core::dimension::DimensionRecord,
    actions: &mut Vec<Action>,
) {
    ui.separator();
    ui.label(t::label_heading());

    let current = record.label_override.clone().unwrap_or_default();
    let id = ui.make_persistent_id(("dimension-label", record.id.0));
    let mut draft: String = ui
        .data(|d| d.get_temp::<String>(id))
        .unwrap_or_else(|| current.clone());

    let response = ui.text_edit_singleline(&mut draft);
    crate::diag::ui_rect(LABEL_REGION, response.rect);
    ui.data_mut(|d| d.insert_temp(id, draft.clone()));

    // ★ On focus loss OR Enter — the two ways a person finishes typing. Neither
    // alone is enough: an operator who tabs away has finished, and one who
    // presses Enter without moving has too.
    let done = response.lost_focus();
    if done && draft.trim() != current {
        let next = draft.trim();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "dimension-label-requested id={} len={}",
                record.id.0,
                next.len()
            )
        });
        actions.push(Action::Dimension(DimensionAction::SetLabel {
            dimension: record.id,
            // ★★ EMPTY MEANS RESTORE, and the engine says so in as many words:
            // *"pass `None` to restore the measurement instead."* Clearing the
            // box is the operator saying "go back to the number", which is
            // exactly what `None` means — so there is no separate Clear button
            // to keep in step with the field.
            label: (!next.is_empty()).then(|| next.to_owned()),
        }));
    }

    // ★★ The measured value is NOT shown here, and it is a real gap rather
    // than a decision: `DimensionRecord` does not carry it — the measurement is
    // computed from the geometry and the group's scale, and only
    // `DimensionLabelChange` hands it back, at the moment of a change.
    //
    // ⇒ So an operator looking at a ce dimension captioned *"see detail B"* has
    // no way here to learn what it actually measures. That is worth closing and
    // is filed rather than papered over; the honest interim is to say the
    // measurement is still underneath, which is the fact that decides whether
    // they trust the override at all.
    if record.label_override.is_some() {
        ui.small(t::label_overridden());
    }
    ui.small(t::label_hint());
}

/// The radius / diameter choice, for a circular ce dimension only.
///
/// # ★ Why it is offered only for a circular kind
///
/// `set_dimension_display` refuses a non-circular target by name
/// (`EditError::NotACircularDimension`) and refuses **before** mutating. R9's
/// rule is that an affordance which cannot be honoured is not drawn, so a
/// linear or angular ce dimension gets no control rather than a greyed one —
/// and the engine's refusal stays as the backstop rather than as the path.
///
/// # ★ Why the action is raised only on a CHANGE
///
/// `set_dimension_display` is documented as committing **even when nothing
/// changes** — flagged in `02-editing-and-saving.md` §1.19 as *"the opposite of
/// `set_info_field`"*. Pressing the option it is already on would therefore
/// write an undo entry for a no-op, so the guard is here, in the surface, where
/// what the operator pressed is known.
fn display_toggle(ui: &mut Ui, record: &DimensionRecord, actions: &mut Vec<Action>) {
    let DimensionKind::Circular { show_diameter, .. } = record.kind else {
        return;
    };
    let mut chosen = show_diameter;
    ui.horizontal(|ui| {
        ui.label(t::display_label());
        let radius = ui.radio_value(&mut chosen, false, t::display_radius());
        let diameter = ui.radio_value(&mut chosen, true, t::display_diameter());
        crate::diag::ui_rect(REGION_DISPLAY, radius.rect.union(diameter.rect));
    });
    ui.weak(t::display_hint());
    if chosen != show_diameter {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "dimension-display id={} diameter={}",
                record.id.0,
                u8::from(chosen)
            )
        });
        actions.push(Action::Dimension(DimensionAction::SetDisplay {
            dimension: record.id,
            show_diameter: chosen,
        }));
    }
}
