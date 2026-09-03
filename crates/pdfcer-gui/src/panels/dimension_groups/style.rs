//! # `panels::dimension_groups::style` — a group's appearance defaults, and
//! the count that stops them being a surprise
//!
//! ## What this draws
//!
//! The **middle tier** of `pdfcer-core`'s three-tier style cascade
//! (factory → group → ce dimension). Seven properties, each an `Option` on
//! `GroupStyle`, and **the `Option` is the operator's checkbox**: clear means
//! *this group has not spoken, so pdfcer's own default applies*, ticked means
//! *this group says this*.
//!
//! Two of the seven — `tolerance` and `tolerance_places` — are not drawn here
//! and the reason is at [`show`].
//!
//! ## ★ The count beside every control, and why it is not the engine's
//!
//! The operator's own words, quoted in
//! `docs/ui_specs/tool-options-dock-and-ce-dimension-properties.md` §C.11.1:
//!
//! > *"cannot change one and be surprised 40 others changed or didn't."*
//!
//! `EditSession::set_group_style` returns a count, and it is **the wrong
//! number to show**. `docs/core-api/03-capabilities.md` §1.6 trap (a) says so
//! outright: the return is the number of members *regenerated*, which is every
//! wired member including the ones that override the very property being
//! changed — because regenerating an overrider is byte-identical and free in
//! the diff, so the engine does not bother to exclude them.
//!
//! The number that will visibly **move** is the members whose
//! `StyleProvenance` for that property reports `follows_group() == true`, and
//! it has to be computed **before** the edit if it is to be shown before the
//! edit. [`will_move`] is that computation, and it is called every frame for
//! every drawn property so the sentence under a control is always about the
//! model as it stands.
//!
//! ## ★ `Factory` counts as following the group, and this is the easy thing to
//! get wrong
//!
//! `StyleSource::follows_group()` is `true` for **both** `Factory` and
//! `Group`. A property nobody has set yet *will* move when the group sets one
//! — the group simply has not spoken. A panel that derives the predicate by
//! hand and tests only for `Group` greys out, or under-counts, exactly the rows
//! that are about to change.
//!
//! `pdfcer-core` pins that with a test named for the trap
//! (`style.rs:643 factory_sourced_properties_still_follow_a_group_edit`), and
//! this module never re-derives the predicate: it calls the engine's.

use egui::Ui;
use pdfcer_core::dimension::{
    ArrowForm, DimensionModel, Group, StyleDefaults, StyleSource, style_provenance,
};
use pdfcer_core::vector::Rgb;

use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;
use crate::text::dimension_groups as t;

/// The region the appearance section publishes, so a driven check can find it.
pub const REGION: &str = "dimension-groups.appearance"; // ui-text-exempt: trace region name, never displayed
/// The region the text-height control publishes.
///
/// ★ One driveable, non-popup control is named deliberately.
/// `NO_SURFACE.md` §4 records what a colour picker costs a harness: *"the
/// picker's popup publishes no regions, so a check can assert the swatch was
/// drawn and driven but cannot aim at a hue inside it."* Any group of controls
/// containing a colour picker therefore needs a numeric neighbour the harness
/// **can** move, and this is it.
pub const REGION_TEXT_HEIGHT: &str = "dimension-groups.text_height"; // ui-text-exempt: trace region name, never displayed

/// The drag speed for the three point-valued properties.
///
/// Slow enough that a drag lands on a tenth rather than skating past it. These
/// are typographic sizes — 10 pt text, a 0.75 pt line — where the difference
/// between 0.7 and 0.8 is visible on a plot.
const POINT_SPEED: f64 = 0.05;

/// The legal range for a text height, in points.
///
/// ★ Bounded here rather than in the engine because the engine does not bound
/// it: `GroupStyle::text_height` is a bare `Option<f64>`. The floor is the
/// smallest size that survives a 1:100 plot; the ceiling is where a label stops
/// fitting between its own witness lines on an A3 sheet. Neither is a hard
/// refusal — an operator who needs 60 pt can set it from the CLI, which is the
/// right place for a value outside what a drawing normally uses.
const TEXT_HEIGHT_RANGE: std::ops::RangeInclusive<f64> = 1.0..=48.0;
/// The legal range for a stroke width, in points. Hairline to heavy.
const LINE_WIDTH_RANGE: std::ops::RangeInclusive<f64> = 0.05..=6.0;
/// The legal range for an arrowhead length, in points.
const ARROW_LENGTH_RANGE: std::ops::RangeInclusive<f64> = 1.0..=30.0;

/// Draw the appearance-defaults section for `group`, raising at most one
/// [`DimensionAction::SetGroupStyle`].
///
/// # Read-modify-write, once per frame at most
///
/// The section starts from the group's **live** style, mutates a copy as the
/// operator touches controls, and raises one action if the copy differs at the
/// end. That is the CLI's own convention (`pdfcer group-style`: setting one
/// property leaves the others alone) and it is what keeps a click here and a
/// command-line invocation the same edit.
///
/// One action per frame rather than one per control is not an optimisation: two
/// `SetGroupStyle` actions raised in the same frame would each carry a *whole*
/// tier computed from the same starting point, so the second would silently
/// undo the first. The queue drains in order and the last writer would win.
///
/// # ★ Why tolerance is not drawn here
///
/// `GroupStyle` carries `tolerance` and `tolerance_places`, so a group *can*
/// default them — and a group-level tolerance is the rarer half of the feature.
/// A tolerance is a statement about **one manufactured feature**: two holes on
/// the same drawing routinely carry different ones even though they share the
/// drawing's units and precision, which is the ui-spec's own reasoning
/// (§C.11.1) and the reference tool's.
///
/// So tolerance belongs on the **per-ce-dimension** surface, where it is drawn,
/// and putting a second control for it here would invite an operator to set a
/// group default that almost every member then overrides — the shape that makes
/// the moving-count read *"no change on screen"* on nearly every press. It is
/// reachable from the CLI for the drawing that genuinely wants one, and this
/// paragraph is the record of that being a decision rather than an omission.
pub fn show(ui: &mut Ui, model: &DimensionModel, group: &Group, actions: &mut Vec<Action>) {
    crate::diag::ui_rect(REGION, ui.max_rect());
    // ★ The heading itself is the FOLD's caption, not a label here — see
    // `super::section`. Drawing it twice was the first draft and it read as a
    // section inside a section. The hint stays: a fold caption is four words
    // and the sentence under it is what says these are *defaults*.
    ui.label(t::appearance_hint());
    ui.add_space(4.0);

    let mut next = group.style;
    let total = model.member_count(group.id);

    // --- text height ----------------------------------------------------
    let moving = will_move(model, group, |p| p.text_height);
    property_row(
        ui,
        t::prop_text_height(),
        &mut next.text_height,
        StyleDefaults::FACTORY.text_height,
        (moving, total),
        |ui, value| {
            let r = ui.add(
                egui::DragValue::new(value)
                    .speed(POINT_SPEED)
                    .range(TEXT_HEIGHT_RANGE)
                    .suffix(t::points_suffix()),
            );
            crate::diag::ui_rect(REGION_TEXT_HEIGHT, r.rect);
        },
        t::points_value,
    );

    // --- line width -----------------------------------------------------
    let moving = will_move(model, group, |p| p.line_width);
    property_row(
        ui,
        t::prop_line_width(),
        &mut next.line_width,
        StyleDefaults::FACTORY.line_width,
        (moving, total),
        |ui, value| {
            ui.add(
                egui::DragValue::new(value)
                    .speed(POINT_SPEED)
                    .range(LINE_WIDTH_RANGE)
                    .suffix(t::points_suffix()),
            );
        },
        t::points_value,
    );

    // --- arrow length ---------------------------------------------------
    let moving = will_move(model, group, |p| p.arrow_length);
    property_row(
        ui,
        t::prop_arrow_length(),
        &mut next.arrow_length,
        StyleDefaults::FACTORY.arrow_length,
        (moving, total),
        |ui, value| {
            ui.add(
                egui::DragValue::new(value)
                    .speed(POINT_SPEED)
                    .range(ARROW_LENGTH_RANGE)
                    .suffix(t::points_suffix()),
            );
        },
        t::points_value,
    );

    // --- arrow form -----------------------------------------------------
    let moving = will_move(model, group, |p| p.arrow_form);
    property_row(
        ui,
        t::prop_arrow_form(),
        &mut next.arrow_form,
        StyleDefaults::FACTORY.arrow_form,
        (moving, total),
        |ui, value| {
            egui::ComboBox::from_id_salt("dimension-group-arrow-form")
                .selected_text(t::arrow_form_name(*value))
                .show_ui(ui, |ui| {
                    // ★ `ArrowForm::ALL` rather than a local list. Its own doc
                    // comment says why it exists — *"must not drift from the
                    // enum"* — and a form the engine gains appears here without
                    // a shell change, which is the same reason the New-document
                    // window iterates `PaperSize::ALL`.
                    for form in ArrowForm::ALL {
                        ui.selectable_value(value, form, t::arrow_form_name(form));
                    }
                });
        },
        |v| t::arrow_form_name(v).to_owned(),
    );

    // --- colour ---------------------------------------------------------
    let moving = will_move(model, group, |p| p.color);
    property_row(
        ui,
        t::prop_color(),
        &mut next.color,
        StyleDefaults::FACTORY.color,
        (moving, total),
        |ui, value| {
            let mut screen = color32_of(*value);
            if ui.color_edit_button_srgba(&mut screen).changed() {
                *value = rgb_of(screen);
            }
        },
        |_| String::new(),
    );

    if next != group.style {
        actions.push(Action::Dimension(DimensionAction::SetGroupStyle {
            group: group.id,
            style: next,
        }));
    }
}

/// One property: the override checkbox, the editor it gates, and the sentence
/// saying what pressing it will move.
///
/// # Why the checkbox and the editor are one function
///
/// Because the pair is the *representation of an `Option`*, and splitting them
/// would let a future property draw one without the other. Ticking the box has
/// to seed a value — you cannot edit `None` — and the seed is the factory
/// default rather than zero, so the first frame after ticking shows the value
/// that was already in force. Clearing it restores **inheritance**, not the
/// previously-inherited value frozen in place, which is the engine's
/// deliberate divergence from the reference tool (`style.rs:269-278`).
///
/// # Why the editor is drawn only when the box is ticked
///
/// R9: greying is for *temporarily* unavailable, and an inherited property is
/// not unavailable — it has a value, supplied by a tier above. A greyed
/// spinner showing the factory number would invite the operator to drag it and
/// then decline, which is the affordance-that-cannot-be-honoured shape. The
/// caption in its place states the inherited value in words instead, so the
/// information is not lost with the control.
///
/// # Why `reach` is one parameter and not two
///
/// `(moving, total)` travel together into one sentence and are meaningless
/// apart — *"3"* says nothing without *"of 40"*, and that is the whole point of
/// the disclosure. Passing them as a pair also keeps this function inside
/// clippy's argument budget without dropping the `describe` closure, which is
/// what renders the inherited value in words when the editor is absent.
fn property_row<T: Copy + PartialEq>(
    ui: &mut Ui,
    label: &str,
    slot: &mut Option<T>,
    factory: T,
    reach: (usize, usize),
    editor: impl FnOnce(&mut Ui, &mut T),
    describe: impl FnOnce(T) -> String,
) {
    let (moving, total) = reach;
    ui.horizontal_wrapped(|ui| {
        let mut set = slot.is_some();
        if ui.checkbox(&mut set, t::set_by_group()).changed() {
            *slot = if set { Some(factory) } else { None };
        }
        ui.label(label);
        match slot.as_mut() {
            Some(value) => editor(ui, value),
            None => {
                let described = describe(factory);
                if !described.is_empty() {
                    ui.weak(t::using_factory(&described));
                }
            }
        }
    });
    // The disclosure sits under the row rather than beside it: it is a sentence,
    // and a sentence on the same line as a spinner is a sentence nobody reads.
    ui.weak(t::members_that_will_move(moving, total));
    ui.add_space(6.0);
}

/// **How many of `group`'s members will visibly change** if the group's value
/// for one property moves.
///
/// `pick` selects the property out of the engine's own
/// `StyleProvenance` — one field per property, and the struct is what
/// `style_provenance` returns, so nothing here re-derives which tier supplied
/// a value.
///
/// # Why the predicate is the engine's and not `== StyleSource::Group`
///
/// Because `StyleSource::follows_group()` is `true` for `Factory` as well, and
/// that is the whole trap. A member that has never had the property set
/// anywhere follows the group the moment the group speaks. Testing for `Group`
/// alone would report zero on a fresh document — every member `Factory` — which
/// is the case where the count matters most, because that is the press that
/// changes everything on the sheet.
fn will_move(
    model: &DimensionModel,
    group: &Group,
    pick: impl Fn(&pdfcer_core::dimension::StyleProvenance) -> StyleSource,
) -> usize {
    model
        .members(group.id)
        .filter(|record| pick(&style_provenance(group, &record.style)).follows_group())
        .count()
}

/// A screen colour from PDF components.
///
/// Opaque, because `/C` on an annotation has no alpha and a picker offering one
/// would be a channel pdfcer silently ignores — the argument
/// `canvas::markup::pen` already makes for the markup swatches, applied to the
/// engine's `Rgb` rather than to the pen's own triple.
fn color32_of(rgb: Rgb) -> egui::Color32 {
    let to_byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    egui::Color32::from_rgb(to_byte(rgb.r), to_byte(rgb.g), to_byte(rgb.b)) // DOCUMENT COLOUR: the ce dimension's own `/C`, chosen by the operator — not a theme colour
}

/// PDF components from a screen colour — the inverse of [`color32_of`].
///
/// Divides by `255.0` rather than `256.0`: the component range is inclusive at
/// both ends, so `255` must map to exactly `1.0` or a pure red chosen in the
/// picker would be written as `0.996` and round-trip to a slightly different
/// swatch.
fn rgb_of(c: egui::Color32) -> Rgb {
    Rgb {
        r: f32::from(c.r()) / 255.0,
        g: f32::from(c.g()) / 255.0,
        b: f32::from(c.b()) / 255.0,
    }
}

#[cfg(test)]
#[allow(
    clippy::float_cmp,
    reason = "exact round-trip is the property under test"
)] // ui-text-exempt: lint justification, never displayed
mod tests {
    use super::*;
    use pdfcer_core::dimension::{DEFAULT_GROUP_ID, DimensionKind, StyleOverrides, Unit};
    use pdfcer_core::vector::{AxisConstraint, Point};

    fn a_linear() -> DimensionKind {
        DimensionKind::Linear {
            a: Point::new(0.0, 0.0),
            b: Point::new(100.0, 0.0),
            constraint: AxisConstraint::Aligned,
            offset: 10.0,
            text_along: 0.0,
        }
    }

    /// ★ **The trap, asserted.** A member that has never been given a value
    /// anywhere still follows the group.
    ///
    /// This is the test that would have caught a hand-rolled
    /// `== StyleSource::Group` predicate, and it is written against a *fresh*
    /// model precisely because that is the case where the wrong predicate
    /// reports zero and the right one reports everything.
    #[test]
    fn a_member_that_overrides_nothing_is_counted_as_moving() {
        let mut model = DimensionModel::new();
        let group = model.add_group("Plan", Unit::Millimeter);
        for _ in 0..3 {
            model.add_dimension(group, a_linear());
        }
        let g = model.group(group).expect("the group was just added");
        assert_eq!(
            will_move(&model, g, |p| p.text_height),
            3,
            "three members inherit from the factory, and all three move when \
             the group speaks — counting only StyleSource::Group would say 0"
        );
    }

    /// A member that overrides the property is not counted, and only that one
    /// property is excluded.
    ///
    /// The second half is the one worth having: a member overriding the text
    /// height still follows the group for the line width, and a count that
    /// excluded it from both would under-report the wider edit.
    #[test]
    fn an_override_is_excluded_from_its_own_property_and_no_other() {
        let mut model = DimensionModel::new();
        let group = model.add_group("Plan", Unit::Millimeter);
        let a = model.add_dimension(group, a_linear());
        model.add_dimension(group, a_linear());
        model.dimension_mut(a).expect("just added").style = StyleOverrides {
            text_height: Some(3.0),
            ..StyleOverrides::default()
        };

        let g = model.group(group).expect("just added");
        assert_eq!(
            will_move(&model, g, |p| p.text_height),
            1,
            "the overriding member does not move"
        );
        assert_eq!(
            will_move(&model, g, |p| p.line_width),
            2,
            "it still follows the group for every property it did not override"
        );
    }

    /// A group with no members reports zero, and the sentence for it is the
    /// "nothing to redraw" one rather than the "all of them override" one.
    #[test]
    fn an_empty_group_moves_nothing() {
        let model = DimensionModel::new();
        let g = model
            .group(DEFAULT_GROUP_ID)
            .expect("the default group is always present");
        assert_eq!(will_move(&model, g, |p| p.color), 0);
        assert!(
            crate::text::dimension_groups::members_that_will_move(0, 0)
                .contains("no dimensions yet")
        );
    }

    /// The colour round-trips exactly, including both ends of the range.
    ///
    /// ★ `255 → 1.0 → 255` is the case the `/ 255.0` divisor exists for. With
    /// `256.0` a pure red would be written as `0.996` and the swatch an operator
    /// reopened would not be the one they chose — a difference small enough to
    /// dismiss and permanent once it is in the file.
    #[test]
    fn a_colour_survives_the_round_trip_at_both_ends() {
        for c in [
            egui::Color32::from_rgb(255, 0, 0), // DOCUMENT COLOUR: a test operand, not a theme colour
            egui::Color32::from_rgb(0, 0, 0), // DOCUMENT COLOUR: a test operand, not a theme colour
            egui::Color32::from_rgb(17, 128, 240), // DOCUMENT COLOUR: a test operand, not a theme colour
        ] {
            assert_eq!(color32_of(rgb_of(c)), c, "{c:?} did not survive");
        }
        assert_eq!(rgb_of(egui::Color32::from_rgb(255, 255, 255)).r, 1.0); // DOCUMENT COLOUR: a test operand, not a theme colour
    }
}
