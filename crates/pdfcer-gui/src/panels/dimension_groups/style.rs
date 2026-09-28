//! # `panels::dimension_groups::style` — a group's appearance defaults, and
//! the count that stops them being a surprise
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/dimension_groups/style.md`.

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
pub const REGION_TEXT_HEIGHT: &str = "dimension-groups.text_height"; // ui-text-exempt: trace region name, never displayed

/// The drag speed for the three point-valued properties.
const POINT_SPEED: f64 = 0.05;

/// The legal range for a text height, in points.
const TEXT_HEIGHT_RANGE: std::ops::RangeInclusive<f64> = 1.0..=48.0;
/// The legal range for a stroke width, in points. Hairline to heavy.
const LINE_WIDTH_RANGE: std::ops::RangeInclusive<f64> = 0.05..=6.0;
/// The legal range for an arrowhead length, in points.
const ARROW_LENGTH_RANGE: std::ops::RangeInclusive<f64> = 1.0..=30.0;

/// Draw the appearance-defaults section for `group`, raising at most one
/// [`DimensionAction::SetGroupStyle`].
pub fn show(ui: &mut Ui, model: &DimensionModel, group: &Group, actions: &mut Vec<Action>) {
    crate::diag::ui_rect(REGION, ui.max_rect());
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
            let r = point_box(ui, value, TEXT_HEIGHT_RANGE);
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
            point_box(ui, value, LINE_WIDTH_RANGE);
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
            point_box(ui, value, ARROW_LENGTH_RANGE);
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
                    // `ArrowForm::ALL` rather than a local list. Its own doc
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
fn color32_of(rgb: Rgb) -> egui::Color32 {
    let to_byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    egui::Color32::from_rgb(to_byte(rgb.r), to_byte(rgb.g), to_byte(rgb.b)) // DOCUMENT COLOUR: the ce dimension's own `/C`, chosen by the operator — not a theme colour
}

/// PDF components from a screen colour — the inverse of [`color32_of`].
fn rgb_of(c: egui::Color32) -> Rgb {
    Rgb {
        r: f32::from(c.r()) / 255.0,
        g: f32::from(c.g()) / 255.0,
        b: f32::from(c.b()) / 255.0,
    }
}

/// A point-valued box: shows `pt`, reads a typed unit or arithmetic.
fn point_box(ui: &mut Ui, value: &mut f64, range: std::ops::RangeInclusive<f64>) -> egui::Response {
    let (widget, refusal) = pdfcer_gui_base::entry::drag_value(
        ui,
        value,
        pdfcer_gui_base::entry::Kind::Length(pdfcer_gui_base::entry::LengthUnit::Point),
    );
    refusal.show(ui.add(widget.speed(POINT_SPEED).range(range)))
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
            extension_gap: [None; 2],
        }
    }

    /// **The trap, asserted.** A member that has never been given a value
    /// anywhere still follows the group.
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
