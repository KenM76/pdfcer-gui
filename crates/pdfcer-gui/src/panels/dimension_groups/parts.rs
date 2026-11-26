//! A group's style for the parts its ce dimensions draw besides their own
//! lines: the line-type scale, the radius or diameter circle, the centre line
//! and the centre mark. Each row is *Set by group* like the others in
//! [`super::style`], and says how many members it will move.
//!
//! Publishes `dimension-groups.<part>.<dash|width|colour>.set`, `<part>` being
//! `circle`, `centre-line` or `centre-mark`, and
//! `dimension-groups.line-type-scale.set`.

use egui::Ui;
use pdfcer_core::dimension::{
    CentreMarkSize, GroupStyle, LineType, LineTypeRef, PartStroke, StyleDefaults, StyleProvenance,
    StyleSource,
};
use pdfcer_gui_base::text::panels::dimension as dt;

use super::style::{LINE_WIDTH_RANGE, Rows, point_box, property_row};
use crate::text::dimension_groups as t;

/// The legal line-type scale.
const SCALE_RANGE: std::ops::RangeInclusive<f64> = 0.1..=10.0;

/// The provenance fields of one part's dash, width and colour.
type Pick = fn(&StyleProvenance) -> StyleSource;

/// One part: its region key, heading, the type its unset dash draws in, and
/// where its three properties are read from.
struct Part {
    key: &'static str,
    heading: &'static str,
    dash: LineTypeRef,
    picks: [Pick; 3],
}

/// Draw the line-type scale, then a section per part.
pub(super) fn show(ui: &mut Ui, rows: &Rows<'_>, next: &mut GroupStyle) {
    property_row(
        ui,
        dt::prop_line_type_scale(),
        &mut next.line_type_scale,
        (
            StyleDefaults::FACTORY.line_type_scale,
            Some("dimension-groups.line-type-scale.set"), // ui-text-exempt: trace region name
        ),
        rows.reach(|p| p.line_type_scale),
        |ui, value| {
            ui.add(egui::DragValue::new(value).speed(0.01).range(SCALE_RANGE));
        },
        |v| format!("{v}"),
    );
    let circle = Part {
        key: "circle", // ui-text-exempt: a trace region segment
        heading: dt::part_circle(),
        dash: StyleDefaults::FACTORY.dash,
        picks: [|p| p.circle_dash, |p| p.circle_width, |p| p.circle_color],
    };
    part_rows(ui, rows, &circle, &mut next.circle);
    let line = Part {
        key: "centre-line", // ui-text-exempt: a trace region segment
        heading: dt::part_centre_line(),
        dash: LineTypeRef::Named(LineType::Center),
        picks: [
            |p| p.centre_line_dash,
            |p| p.centre_line_width,
            |p| p.centre_line_color,
        ],
    };
    part_rows(ui, rows, &line, &mut next.centre.line);
    let mark = Part {
        key: "centre-mark", // ui-text-exempt: a trace region segment
        heading: dt::part_centre_mark(),
        dash: LineTypeRef::Named(LineType::Continuous),
        picks: [
            |p| p.centre_mark_dash,
            |p| p.centre_mark_width,
            |p| p.centre_mark_color,
        ],
    };
    part_rows(ui, rows, &mark, &mut next.centre.mark);
    mark_rows(ui, rows, next);
}

/// A part's dash, width and colour, each *Set by group*.
fn part_rows(ui: &mut Ui, rows: &Rows<'_>, part: &Part, stroke: &mut PartStroke) {
    let key = part.key;
    ui.add_space(4.0);
    ui.label(part.heading);
    let dash_set = format!("dimension-groups.{key}.dash.set"); // ui-text-exempt: trace region name
    let dash_region = format!("dimension-groups.{key}.dash"); // ui-text-exempt: trace region name
    property_row(
        ui,
        dt::prop_dash(),
        &mut stroke.dash,
        (part.dash, Some(&dash_set)),
        rows.reach(part.picks[0]),
        |ui, value| {
            pdfcer_gui_base::dimlinetype::line_type_editor(
                ui,
                &dash_region,
                Some(&dash_region),
                value,
            );
        },
        |_| dt::part_follows().to_owned(),
    );
    let width_set = format!("dimension-groups.{key}.width.set"); // ui-text-exempt: trace region name
    property_row(
        ui,
        dt::prop_line_width(),
        &mut stroke.width,
        (StyleDefaults::FACTORY.line_width, Some(&width_set)),
        rows.reach(part.picks[1]),
        |ui, value| {
            point_box(ui, value, LINE_WIDTH_RANGE);
        },
        |_| dt::part_follows().to_owned(),
    );
    let colour_set = format!("dimension-groups.{key}.colour.set"); // ui-text-exempt: trace region name
    let colour = format!("dimension-groups.{key}.colour"); // ui-text-exempt: trace region name
    property_row(
        ui,
        dt::prop_color(),
        &mut stroke.color,
        (StyleDefaults::FACTORY.color, Some(&colour_set)),
        rows.reach(part.picks[2]),
        |ui, value| {
            pdfcer_gui_base::colourpick::edit_color(ui, &colour, value, &colour);
        },
        |_| dt::part_follows().to_owned(),
    );
}

/// A centre mark's gap, extension and line font.
fn mark_rows(ui: &mut Ui, rows: &Rows<'_>, next: &mut GroupStyle) {
    let size: CentreMarkSize = next
        .centre_mark_size
        .unwrap_or(StyleDefaults::FACTORY.centre_mark_size);
    let text_height = next
        .text_height
        .unwrap_or(StyleDefaults::FACTORY.text_height);
    property_row(
        ui,
        dt::prop_centre_mark_gap(),
        &mut next.centre.mark_gap,
        (size, Some("dimension-groups.centre-mark.gap.set")), // ui-text-exempt: trace region name
        rows.reach(|p| p.centre_mark_gap),
        |ui, value| {
            pdfcer_gui_base::dimensionoverrides::centre_mark_size_editor(
                ui,
                "dimension-group-mark-gap",
                value,
                text_height,
            );
        },
        |_| t::centre_mark_factory_value().to_owned(),
    );
    property_row(
        ui,
        dt::prop_centre_mark_extension(),
        &mut next.centre.mark_extension,
        (size, Some("dimension-groups.centre-mark.extension.set")), // ui-text-exempt: trace region name
        rows.reach(|p| p.centre_mark_extension),
        |ui, value| {
            pdfcer_gui_base::dimensionoverrides::centre_mark_size_editor(
                ui,
                "dimension-group-mark-reach",
                value,
                text_height,
            );
        },
        |_| t::centre_mark_factory_value().to_owned(),
    );
    property_row(
        ui,
        dt::prop_centre_mark_line_font(),
        &mut next.centre.mark_line_font,
        (false, Some("dimension-groups.centre-mark.line-font.set")), // ui-text-exempt: trace region name
        rows.reach(|p| p.centre_mark_line_font),
        |ui, value| {
            ui.checkbox(value, dt::centre_mark_line_font_on());
        },
        |_| String::new(),
    );
}
