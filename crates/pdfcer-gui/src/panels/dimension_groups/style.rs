//! # `panels::dimension_groups::style` — a group's appearance defaults, and
//! the count that stops them being a surprise
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/dimension_groups/style.md`.

use egui::Ui;
use pdfcer_core::dimension::{
    ArrowForm, DimensionModel, Group, GroupStyle, StyleDefaults, StyleProvenance, StyleSource,
    style_provenance,
};

use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;
use crate::text::dimension_groups as t;

/// The region the appearance section publishes, so a driven check can find it.
pub const REGION: &str = "dimension-groups.appearance"; // ui-text-exempt: trace region name, never displayed
/// The region the text-height control publishes.
pub const REGION_TEXT_HEIGHT: &str = "dimension-groups.text_height"; // ui-text-exempt: trace region name, never displayed
/// The region the colour swatch publishes; its picker publishes beneath it.
pub const REGION_COLOUR: &str = "dimension-groups.colour"; // ui-text-exempt: trace region name, never displayed
/// The line-style row's *Set by group* checkbox.
pub const REGION_DASH_SET: &str = "dimension-groups.dash.set"; // ui-text-exempt: trace region name, never displayed
/// The line-style chooser: `.combo`, and `.option.{token}` while open.
pub const REGION_DASH: &str = "dimension-groups.dash"; // ui-text-exempt: trace region name, never displayed
/// The opacity row's *Set by group* checkbox.
pub const REGION_OPACITY_SET: &str = "dimension-groups.opacity.set"; // ui-text-exempt: trace region name, never displayed
/// The opacity percentage box.
pub const REGION_OPACITY: &str = "dimension-groups.opacity"; // ui-text-exempt: trace region name, never displayed

/// The drag speed for the three point-valued properties.
const POINT_SPEED: f64 = 0.05;

/// The legal range for a text height, in points.
const TEXT_HEIGHT_RANGE: std::ops::RangeInclusive<f64> = 1.0..=48.0;
/// The legal range for a stroke width, in points. Hairline to heavy.
pub(super) const LINE_WIDTH_RANGE: std::ops::RangeInclusive<f64> = 0.05..=6.0;
/// The legal range for an arrowhead length, in points.
const ARROW_LENGTH_RANGE: std::ops::RangeInclusive<f64> = 1.0..=30.0;

/// Draw the appearance-defaults section for `group`, raising at most one
/// [`DimensionAction::SetGroupStyle`].
pub fn show(ui: &mut Ui, model: &DimensionModel, group: &Group, actions: &mut Vec<Action>) {
    crate::diag::ui_rect(REGION, ui.max_rect());
    ui.label(t::appearance_hint());
    ui.add_space(4.0);

    let mut next = group.style;
    let rows = Rows {
        model,
        group,
        total: model.member_count(group.id),
    };
    size_rows(ui, &rows, &mut next);
    shape_rows(ui, &rows, &mut next);
    stroke_rows(ui, &rows, &mut next);
    super::parts::show(ui, &rows, &mut next);

    if next != group.style {
        actions.push(Action::Dimension(DimensionAction::SetGroupStyle {
            group: group.id,
            style: Box::new(next),
        }));
    }
}

/// What every row reads: the model, the group, and its member count.
pub(super) struct Rows<'a> {
    model: &'a DimensionModel,
    group: &'a Group,
    total: usize,
}

impl Rows<'_> {
    /// The row's `(moving, total)` pair for the property `pick` names.
    pub(super) fn reach(&self, pick: impl Fn(&StyleProvenance) -> StyleSource) -> (usize, usize) {
        (will_move(self.model, self.group, pick), self.total)
    }
}

/// Text height, line width and arrow length: the three point-valued rows.
fn size_rows(ui: &mut Ui, rows: &Rows<'_>, next: &mut GroupStyle) {
    property_row(
        ui,
        t::prop_text_height(),
        &mut next.text_height,
        (StyleDefaults::FACTORY.text_height, None),
        rows.reach(|p| p.text_height),
        |ui, value| {
            let r = point_box(ui, value, TEXT_HEIGHT_RANGE);
            crate::diag::ui_rect(REGION_TEXT_HEIGHT, r.rect);
        },
        t::points_value,
    );
    property_row(
        ui,
        t::prop_line_width(),
        &mut next.line_width,
        (StyleDefaults::FACTORY.line_width, None),
        rows.reach(|p| p.line_width),
        |ui, value| {
            point_box(ui, value, LINE_WIDTH_RANGE);
        },
        t::points_value,
    );
    property_row(
        ui,
        t::prop_arrow_length(),
        &mut next.arrow_length,
        (StyleDefaults::FACTORY.arrow_length, None),
        rows.reach(|p| p.arrow_length),
        |ui, value| {
            point_box(ui, value, ARROW_LENGTH_RANGE);
        },
        t::points_value,
    );
}

/// Arrow form and colour.
fn shape_rows(ui: &mut Ui, rows: &Rows<'_>, next: &mut GroupStyle) {
    property_row(
        ui,
        t::prop_arrow_form(),
        &mut next.arrow_form,
        (StyleDefaults::FACTORY.arrow_form, None),
        rows.reach(|p| p.arrow_form),
        |ui, value| {
            egui::ComboBox::from_id_salt("dimension-group-arrow-form")
                .selected_text(t::arrow_form_name(*value))
                .show_ui(ui, |ui| {
                    // `ArrowForm::ALL`, so a form the engine gains appears
                    // here without a shell change.
                    for form in ArrowForm::ALL {
                        ui.selectable_value(value, form, t::arrow_form_name(form));
                    }
                });
        },
        |v| t::arrow_form_name(v).to_owned(),
    );
    property_row(
        ui,
        t::prop_color(),
        &mut next.color,
        (StyleDefaults::FACTORY.color, None),
        rows.reach(|p| p.color),
        |ui, value| {
            pdfcer_gui_base::colourpick::edit_color(
                ui,
                "dimension-group-colour",
                value,
                REGION_COLOUR,
            );
        },
        |_| String::new(),
    );
}

/// Line style, opacity and centre-mark size, through the same editors as a single ce
/// dimension's overrides.
fn stroke_rows(ui: &mut Ui, rows: &Rows<'_>, next: &mut GroupStyle) {
    property_row(
        ui,
        t::prop_dash(),
        &mut next.dash,
        (StyleDefaults::FACTORY.dash, Some(REGION_DASH_SET)),
        rows.reach(|p| p.dash),
        |ui, value| {
            pdfcer_gui_base::dimlinetype::line_type_editor(
                ui,
                "dimension-group-dash",
                Some(REGION_DASH),
                value,
            );
        },
        |d| {
            if d.line_type() == Some(pdfcer_core::dimension::LineType::Continuous) {
                t::dash_factory_value().to_owned()
            } else {
                String::new()
            }
        },
    );
    property_row(
        ui,
        t::prop_opacity(),
        &mut next.opacity,
        (StyleDefaults::FACTORY.opacity, Some(REGION_OPACITY_SET)),
        rows.reach(|p| p.opacity),
        |ui, value| {
            let r = pdfcer_gui_base::dimensionoverrides::opacity_editor(ui, value);
            crate::diag::ui_rect(REGION_OPACITY, r.rect);
        },
        t::percent_value,
    );
    let text_height = next
        .text_height
        .unwrap_or(StyleDefaults::FACTORY.text_height);
    property_row(
        ui,
        t::prop_centre_mark_size(),
        &mut next.centre_mark_size,
        (StyleDefaults::FACTORY.centre_mark_size, None),
        rows.reach(|p| p.centre_mark_size),
        |ui, value| {
            pdfcer_gui_base::dimensionoverrides::centre_mark_size_editor(
                ui,
                "dimension-group-centre-mark",
                value,
                text_height,
            );
        },
        |_| t::centre_mark_factory_value().to_owned(),
    );
}

/// One property: the override checkbox, the editor it gates, and the sentence
/// saying what pressing it will move.
///
/// `seed` is the factory value a tick starts from, and the region the
/// checkbox publishes, if any.
pub(super) fn property_row<T: Copy + PartialEq>(
    ui: &mut Ui,
    label: &str,
    slot: &mut Option<T>,
    seed: (T, Option<&str>),
    reach: (usize, usize),
    editor: impl FnOnce(&mut Ui, &mut T),
    describe: impl FnOnce(T) -> String,
) {
    let (moving, total) = reach;
    let (factory, set_region) = seed;
    ui.horizontal_wrapped(|ui| {
        let mut set = slot.is_some();
        let tick = ui.checkbox(&mut set, t::set_by_group());
        if let Some(region) = set_region {
            crate::diag::ui_rect(region, tick.rect);
        }
        if tick.changed() {
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
    pick: impl Fn(&StyleProvenance) -> StyleSource,
) -> usize {
    model
        .members(group.id)
        .filter(|record| pick(&style_provenance(group, &record.style)).follows_group())
        .count()
}

/// A point-valued box: shows `pt`, reads a typed unit or arithmetic.
pub(super) fn point_box(
    ui: &mut Ui,
    value: &mut f64,
    range: std::ops::RangeInclusive<f64>,
) -> egui::Response {
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
}
