//! A ce dimension's per-part style rows: the radius or diameter circle's
//! stroke, a centre line's stroke, a centre mark's stroke, gap, extension and
//! line font, and the line-type scale every named type is drawn at.
//!
//! Each part's rows appear only on a kind that draws the part. An unset row
//! follows the dimension's own lines, as the engine's cascade resolves it, and
//! says which tier it came from like every other override row.
//!
//! Publishes `dimension-properties.<part>.<property>` for each part row's
//! *Set here* checkbox, `<part>` being `circle`, `centre-line` or
//! `centre-mark`, and `dimension-properties.line-type-scale.set`.

use egui::Ui;
use pdfcer_core::dimension::{
    DimensionKind, DimensionStyle, LineType, LineTypeRef, PartStroke, StyleProvenance, StyleSource,
};

use crate::dimensionoverrides::{LINE_WIDTH_RANGE, centre_mark_size_editor, point_box, row};
use crate::text::panels::dimension as t;

/// The legal line-type scale.
const SCALE_RANGE: std::ops::RangeInclusive<f64> = 0.1..=10.0;

/// The parts a ce dimension of one kind draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Parts {
    /// The radius or diameter circle.
    pub circle: bool,
    /// A centre line.
    pub centre_line: bool,
    /// A centre mark: its cross and extension lines.
    pub centre_mark: bool,
}

impl Parts {
    /// What `kind` draws.
    #[must_use]
    pub const fn of(kind: &DimensionKind) -> Self {
        match kind {
            DimensionKind::Circular { centre_mark, .. } => Self {
                circle: true,
                centre_line: false,
                centre_mark: centre_mark.is_some(),
            },
            DimensionKind::CentreLine { .. } => Self {
                circle: false,
                centre_line: true,
                centre_mark: false,
            },
            DimensionKind::CentreMark { .. } => Self {
                circle: false,
                centre_line: false,
                centre_mark: true,
            },
            _ => Self {
                circle: false,
                centre_line: false,
                centre_mark: false,
            },
        }
    }
}

/// One part's three stroke cascades and the values an unset row resolves to.
struct Stroke<'a> {
    /// `circle`, `centre-line` or `centre-mark`, for the regions.
    key: &'a str,
    sources: [StyleSource; 3],
    dash: LineTypeRef,
    width: f64,
    color: pdfcer_core::annot_author::Color,
}

/// Draw the rows for the parts `kind` draws, then the line-type scale.
pub fn show(
    ui: &mut Ui,
    provenance: &StyleProvenance,
    resolved: &DimensionStyle,
    kind: &DimensionKind,
    overrides: &mut pdfcer_core::dimension::StyleOverrides,
) {
    let parts = Parts::of(kind);
    if parts.circle {
        let drawn = resolved.circle_stroke();
        let stroke = Stroke {
            key: "circle", // ui-text-exempt: a trace region segment
            sources: [
                provenance.circle_dash,
                provenance.circle_width,
                provenance.circle_color,
            ],
            dash: resolved.circle.dash.unwrap_or(resolved.line_type),
            width: drawn.width,
            color: drawn.color,
        };
        heading(ui, t::part_circle());
        stroke_rows(ui, &stroke, &mut overrides.circle);
    }
    if parts.centre_line {
        let drawn = resolved.centre_line_stroke();
        let stroke = Stroke {
            key: "centre-line", // ui-text-exempt: a trace region segment
            sources: [
                provenance.centre_line_dash,
                provenance.centre_line_width,
                provenance.centre_line_color,
            ],
            dash: resolved
                .centre
                .line
                .dash
                .unwrap_or(LineTypeRef::Named(LineType::Center)),
            width: drawn.width,
            color: drawn.color,
        };
        heading(ui, t::part_centre_line());
        stroke_rows(ui, &stroke, &mut overrides.centre.line);
    }
    if parts.centre_mark {
        centre_mark_rows(ui, provenance, resolved, overrides);
    }
    row(
        ui,
        t::prop_line_type_scale(),
        provenance.line_type_scale,
        &mut overrides.line_type_scale,
        || resolved.line_type_scale,
    )
    .published(ui, "dimension-properties.line-type-scale.set") // ui-text-exempt: trace region name
    .edit(ui, |ui, value| {
        let (widget, refusal) =
            crate::entry::drag_value(ui, value, crate::entry::Kind::Number(&[]));
        refusal.show(ui.add(widget.speed(0.01).range(SCALE_RANGE)));
    });
}

fn heading(ui: &mut Ui, name: &str) {
    ui.add_space(4.0);
    ui.label(name);
}

/// A part's dash, width and colour rows.
fn stroke_rows(ui: &mut Ui, stroke: &Stroke<'_>, part: &mut PartStroke) {
    let key = stroke.key;
    if part.is_follow() {
        ui.weak(t::part_follows());
    }
    row(
        ui,
        t::prop_dash(),
        stroke.sources[0],
        &mut part.dash,
        || stroke.dash,
    )
    .published(ui, &format!("dimension-properties.{key}.dash.set")) // ui-text-exempt: trace region name
    .edit(ui, |ui, value| {
        let region = format!("dimension-properties.{key}.dash"); // ui-text-exempt: trace region name
        crate::dimlinetype::line_type_editor(ui, &region, Some(&region), value);
    });
    row(
        ui,
        t::prop_line_width(),
        stroke.sources[1],
        &mut part.width,
        || stroke.width,
    )
    .published(ui, &format!("dimension-properties.{key}.width.set")) // ui-text-exempt: trace region name
    .edit(ui, |ui, value| {
        point_box(ui, value, LINE_WIDTH_RANGE);
    });
    row(
        ui,
        t::prop_color(),
        stroke.sources[2],
        &mut part.color,
        || stroke.color,
    )
    .published(ui, &format!("dimension-properties.{key}.colour.set")) // ui-text-exempt: trace region name
    .edit(ui, |ui, value| {
        let region = format!("dimension-properties.{key}.colour"); // ui-text-exempt: trace region name
        crate::colourpick::edit_color(ui, &region, value, &region);
    });
}

/// A centre mark's stroke rows, then its gap, extension and line font.
fn centre_mark_rows(
    ui: &mut Ui,
    provenance: &StyleProvenance,
    resolved: &DimensionStyle,
    overrides: &mut pdfcer_core::dimension::StyleOverrides,
) {
    let drawn = resolved.centre_mark_stroke();
    let stroke = Stroke {
        key: "centre-mark", // ui-text-exempt: a trace region segment
        sources: [
            provenance.centre_mark_dash,
            provenance.centre_mark_width,
            provenance.centre_mark_color,
        ],
        dash: resolved
            .centre
            .mark
            .dash
            .unwrap_or(LineTypeRef::Named(LineType::Continuous)),
        width: drawn.width,
        color: drawn.color,
    };
    heading(ui, t::part_centre_mark());
    stroke_rows(ui, &stroke, &mut overrides.centre.mark);
    let text_height = resolved.text_height;
    let arm = resolved
        .centre
        .mark_gap
        .unwrap_or(resolved.centre_mark_size);
    row(
        ui,
        t::prop_centre_mark_gap(),
        provenance.centre_mark_gap,
        &mut overrides.centre.mark_gap,
        || arm,
    )
    .published(ui, "dimension-properties.centre-mark.gap.set") // ui-text-exempt: trace region name
    .edit(ui, |ui, value| {
        centre_mark_size_editor(ui, "dimension-override-mark-gap", value, text_height);
    });
    let reach = resolved
        .centre
        .mark_extension
        .unwrap_or(resolved.centre_mark_size);
    row(
        ui,
        t::prop_centre_mark_extension(),
        provenance.centre_mark_extension,
        &mut overrides.centre.mark_extension,
        || reach,
    )
    .published(ui, "dimension-properties.centre-mark.extension.set") // ui-text-exempt: trace region name
    .edit(ui, |ui, value| {
        centre_mark_size_editor(ui, "dimension-override-mark-reach", value, text_height);
    });
    let font = resolved.centre_mark_line_font();
    row(
        ui,
        t::prop_centre_mark_line_font(),
        provenance.centre_mark_line_font,
        &mut overrides.centre.mark_line_font,
        || font,
    )
    .published(ui, "dimension-properties.centre-mark.line-font.set") // ui-text-exempt: trace region name
    .edit(ui, |ui, value| {
        ui.checkbox(value, t::centre_mark_line_font_on());
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::dimension::{CentreMarkSpec, FitCircle};

    fn fit() -> FitCircle {
        FitCircle {
            center: pdfcer_core::vector::Point::new(0.0, 0.0),
            radius: 10.0,
            residual: 0.0,
        }
    }

    #[test]
    fn a_circle_draws_its_mark_only_when_it_carries_one() {
        let bare = DimensionKind::Circular {
            fit: fit(),
            show_diameter: false,
            show_circle: true,
            leader_angle: 0.0,
            text_distance: None,
            centre_mark: None,
        };
        assert_eq!(
            Parts::of(&bare),
            Parts {
                circle: true,
                ..Parts::default()
            }
        );
        let marked = DimensionKind::Circular {
            fit: fit(),
            show_diameter: false,
            show_circle: true,
            leader_angle: 0.0,
            text_distance: None,
            centre_mark: Some(CentreMarkSpec::default()),
        };
        assert!(Parts::of(&marked).centre_mark);
    }

    #[test]
    fn a_centre_mark_alone_draws_no_circle_rows() {
        let mark = DimensionKind::CentreMark {
            fit: fit(),
            mark: CentreMarkSpec::default(),
        };
        let parts = Parts::of(&mark);
        assert!(parts.centre_mark && !parts.circle && !parts.centre_line);
    }
}
