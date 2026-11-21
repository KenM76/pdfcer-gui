//! How the *Face, edge, corner* measure says its figures: the unit and places
//! lengths and areas are said in, the coordinate system corners and offsets
//! are said in, and the plane distances and a face's area are projected on.
//!
//! The engine converts and projects (`LengthUnit::mm_per_unit`,
//! `CoordinateFrame`, `Span::projected`, `Span::along`,
//! `face_projected_area`); this module holds the operator's choices and asks
//! it. A unit other than the file's is offered only when the file states its
//! own, since converting an unstated unit would invent one. The choices
//! outlive the picks; the unit and places are kept with the viewer's
//! defaults, the system and the projection are this session's.

use egui::Ui;
use pdfcer_3d::vec3::{length, sub};
use pdfcer_3d::{CoordinateFrame, EdgeFigures, EdgeShape, LengthUnit, ModelUnits, Span};

use super::measure::units_word;
use crate::text::model3dreadout::{self as t, Lengths};

/// One unit choice; the suffix is 0 for the file's own, else one more than
/// its place in [`LengthUnit::ALL`].
pub const REGION_UNIT_PREFIX: &str = "model3d.select.unit."; // ui-text-exempt: trace region name, never displayed
/// The places chooser.
pub const REGION_SECOND_PREFIX: &str = "model3d.select.second."; // ui-text-exempt: trace region name, never displayed
/// The places chooser.
pub const REGION_DECIMALS: &str = "model3d.select.decimals"; // ui-text-exempt: trace region name, never displayed
/// *Set from the two edges*.
pub const REGION_SET_FRAME: &str = "model3d.select.frame.set"; // ui-text-exempt: trace region name, never displayed
/// *Use the model's*.
pub const REGION_MODEL_FRAME: &str = "model3d.select.frame.model"; // ui-text-exempt: trace region name, never displayed
/// One projection choice; the suffix is its place in [`PROJECTIONS`].
pub const REGION_PROJECTION_PREFIX: &str = "model3d.select.projection."; // ui-text-exempt: trace region name, never displayed

/// The most places after the point.
pub(super) const MAX_DECIMALS: usize = 6;
/// The model's own axes.
const MODEL_AXES: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// What distances and a face's area are flattened onto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Projection {
    None,
    /// Square to the line from the target to the eye.
    Screen,
    Xy,
    Yz,
    Zx,
}

/// The projection choices, in [`t::PROJECTIONS`]' order.
pub(super) const PROJECTIONS: [Projection; 5] = [
    Projection::None,
    Projection::Screen,
    Projection::Xy,
    Projection::Yz,
    Projection::Zx,
];

/// The trace's word for each projection, in [`PROJECTIONS`]' order.
const PLANE_WORDS: [&str; 5] = ["none", "screen", "xy", "yz", "zx"];

/// The operator's readout choices.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Readout {
    /// `None` says lengths in the file's own unit.
    pub unit: Option<LengthUnit>,
    /// A second unit lengths and areas are also said in.
    pub second: Option<LengthUnit>,
    /// Places after the point, at most [`MAX_DECIMALS`].
    pub decimals: usize,
    /// `None` is the model's own system.
    pub frame: Option<CoordinateFrame>,
    /// A place in [`PROJECTIONS`].
    pub projection: usize,
}

impl Default for Readout {
    fn default() -> Self {
        Readout {
            unit: None,
            second: None,
            decimals: 3,
            frame: None,
            projection: 0,
        }
    }
}

impl Readout {
    /// How lengths are written for a model in `units`: converted only when
    /// the file states its unit and another is chosen.
    pub(super) fn lengths(&self, units: Option<ModelUnits>) -> Lengths<'static> {
        let stated = units.filter(|u| u.from_cad_file).map(|u| u.mm_per_unit);
        self.lengths_from(stated, units_word(units))
    }

    /// [`Readout::lengths`] for a file stating `stated` millimetres per unit,
    /// whose own unit is written `own`.
    fn lengths_from(&self, stated: Option<f64>, own: &'static str) -> Lengths<'static> {
        let decimals = self.decimals.min(MAX_DECIMALS);
        let also = stated
            .zip(self.second)
            .map(|(from, s)| (from / s.mm_per_unit(), t::unit_word(s.symbol())));
        match (stated, self.unit) {
            (Some(from), Some(to)) => Lengths {
                factor: from / to.mm_per_unit(),
                unit: t::unit_word(to.symbol()),
                decimals,
                also,
            },
            _ => Lengths {
                factor: 1.0,
                unit: own,
                decimals,
                also,
            },
        }
    }

    pub(super) fn projection(&self) -> Projection {
        PROJECTIONS[self.projection.min(PROJECTIONS.len() - 1)]
    }

    /// The trace's word for the chosen projection.
    pub(super) fn plane_word(&self) -> &'static str {
        PLANE_WORDS[self.projection.min(PLANE_WORDS.len() - 1)]
    }

    /// The chosen plane's normal in model coordinates: `toward_eye` for the
    /// screen, the system's axis square to the plane otherwise. `None` when
    /// nothing is projected or the view gave no direction.
    pub(super) fn normal(&self, toward_eye: Option<[f64; 3]>) -> Option<[f64; 3]> {
        let axes = self.frame.map_or(MODEL_AXES, |f| f.axes());
        match self.projection() {
            Projection::None => None,
            Projection::Screen => toward_eye,
            Projection::Xy => Some(axes[2]),
            Projection::Yz => Some(axes[0]),
            Projection::Zx => Some(axes[1]),
        }
    }

    /// A model point in the chosen system.
    pub(super) fn point(&self, p: [f64; 3]) -> [f64; 3] {
        self.frame.map_or(p, |f| f.point_to_local(p))
    }

    /// A model offset in the chosen system's axes.
    pub(super) fn vector(&self, v: [f64; 3]) -> [f64; 3] {
        self.frame.map_or(v, |f| f.vector_to_local(v))
    }

    /// The span's projected length and the signed part removed, on the
    /// plane square to `normal`.
    pub(super) fn flattened(span: &Span, normal: [f64; 3]) -> Option<(f64, f64)> {
        Some((span.projected(normal)?.length, span.along(normal)?))
    }
}

/// The system two straight edges set: the origin on the first where the two
/// come nearest (`between.from`), X along the first edge toward its far end
/// from there, and Y toward the second's far end from `between.to`. `None`
/// unless both are straight and not parallel.
pub(super) fn frame_from(
    first: &EdgeFigures,
    second: &EdgeFigures,
    between: &Span,
) -> Option<CoordinateFrame> {
    frame_at(between.from, line_of(first)?, between.to, line_of(second)?)
}

/// A straight edge's direction and its two ends.
type Line = ([f64; 3], [f64; 3], [f64; 3]);

fn line_of(edge: &EdgeFigures) -> Option<Line> {
    let EdgeShape::Line { direction } = edge.shape else {
        return None;
    };
    Some((direction, *edge.points.first()?, *edge.points.last()?))
}

/// The system at `origin` on `first`, X along it away from `origin` and Y
/// along `second` away from `on_second`.
fn frame_at(
    origin: [f64; 3],
    first: Line,
    on_second: [f64; 3],
    second: Line,
) -> Option<CoordinateFrame> {
    CoordinateFrame::new(origin, outward(first, origin), outward(second, on_second))
}

/// A line's direction, turned to point from `at` toward its farther end.
fn outward((direction, first, last): Line, at: [f64; 3]) -> [f64; 3] {
    if length(sub(last, at)) >= length(sub(first, at)) {
        direction
    } else {
        direction.map(|c| -c)
    }
}

/// A point or direction as the trace writes it.
pub(super) fn triple([x, y, z]: [f64; 3]) -> String {
    format!("{x:.6},{y:.6},{z:.6}")
}

/// What the operator changed in [`controls`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Change {
    Unit,
    Second,
    Decimals,
    SetFrame,
    ModelFrame,
    Projection,
}

/// The readout's rows in the Measure tab. `stated` is whether the file states
/// its unit; `can_set` whether the picks can set a system. The caller makes
/// the system, since it holds the picks.
pub(super) fn controls(
    ui: &mut Ui,
    r: &mut Readout,
    stated: bool,
    can_set: bool,
) -> Option<Change> {
    let mut change = None;
    if stated {
        ui.horizontal_wrapped(|ui| {
            ui.label(t::UNIT).on_hover_text(t::UNIT_TIP);
            if unit_choices(ui, REGION_UNIT_PREFIX, t::FILE_UNIT, &mut r.unit) {
                change = Some(Change::Unit);
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(t::SECOND_UNIT).on_hover_text(t::SECOND_UNIT_TIP);
            if unit_choices(ui, REGION_SECOND_PREFIX, t::NO_SECOND_UNIT, &mut r.second) {
                change = Some(Change::Second);
            }
        });
    }
    ui.horizontal_wrapped(|ui| {
        let places = ui
            .add(egui::DragValue::new(&mut r.decimals).range(0..=MAX_DECIMALS))
            .on_hover_text(t::DECIMALS_TIP);
        crate::diag::ui_rect_visible(REGION_DECIMALS, places.rect, ui.clip_rect());
        ui.label(t::DECIMALS);
        if places.changed() {
            change = Some(Change::Decimals);
        }
    });
    if let Some(c) = frame_row(ui, r, can_set) {
        change = Some(c);
    }
    if let Some(c) = projection_row(ui, r) {
        change = Some(c);
    }
    change
}

/// `first` (`None`) and every engine unit as a row of choices, the region
/// `prefix` and the place; whether a click changed `chosen`.
fn unit_choices(ui: &mut Ui, prefix: &str, first: &str, chosen: &mut Option<LengthUnit>) -> bool {
    let names = std::iter::once((None, first))
        .chain(LengthUnit::ALL.map(|u| (Some(u), t::unit_word(u.symbol()))));
    let mut changed = false;
    for (i, (unit, name)) in names.enumerate() {
        let choice = ui.selectable_label(*chosen == unit, name);
        // ui-text-exempt: trace region name, never displayed
        let region = format!("{prefix}{i}");
        crate::diag::ui_rect_visible(&region, choice.rect, ui.clip_rect());
        if choice.clicked() && *chosen != unit {
            *chosen = unit;
            changed = true;
        }
    }
    changed
}

fn frame_row(ui: &mut Ui, r: &mut Readout, can_set: bool) -> Option<Change> {
    let mut change = None;
    ui.horizontal_wrapped(|ui| {
        ui.label(t::COORDINATES);
        let own = r.frame.is_none();
        ui.label(if own {
            t::MODEL_COORDINATES
        } else {
            t::PICKED_COORDINATES
        });
        let set = ui
            .add_enabled(can_set, egui::Button::new(t::SET_FROM_PICKS))
            .on_hover_text(t::SET_FROM_PICKS_TIP)
            .on_disabled_hover_text(t::SET_FROM_PICKS_OFF);
        crate::diag::ui_rect_visible(REGION_SET_FRAME, set.rect, ui.clip_rect());
        if set.clicked() {
            change = Some(Change::SetFrame);
        }
        if !own {
            let back = ui
                .button(t::MODEL_BUTTON)
                .on_hover_text(t::MODEL_BUTTON_TIP);
            crate::diag::ui_rect_visible(REGION_MODEL_FRAME, back.rect, ui.clip_rect());
            if back.clicked() {
                r.frame = None;
                change = Some(Change::ModelFrame);
            }
        }
    });
    change
}

fn projection_row(ui: &mut Ui, r: &mut Readout) -> Option<Change> {
    let mut change = None;
    ui.horizontal_wrapped(|ui| {
        ui.label(t::PROJECTED);
        for (i, name) in t::PROJECTIONS.into_iter().enumerate() {
            let choice = ui
                .selectable_label(r.projection == i, name)
                .on_hover_text(t::PROJECTION_TIPS[i]);
            // ui-text-exempt: trace region name, never displayed
            let region = format!("{REGION_PROJECTION_PREFIX}{i}");
            crate::diag::ui_rect_visible(&region, choice.rect, ui.clip_rect());
            if choice.clicked() && r.projection != i {
                r.projection = i;
                change = Some(Change::Projection);
            }
        }
    });
    change
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_edges_meeting_at_a_corner_set_a_system_there() {
        let first = ([-1.0, 0.0, 0.0], [4.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        let second = ([0.0, -1.0, 0.0], [0.0, 4.0, 0.0], [0.0, 0.0, 0.0]);
        let f = frame_at([0.0; 3], first, [0.0; 3], second).expect("a system");
        assert_eq!(f.origin(), [0.0; 3]);
        assert_eq!(f.point_to_local([4.0, 4.0, 0.0]), [4.0, 4.0, 0.0]);
        assert!(frame_at([0.0; 3], first, [0.0; 3], first).is_none());
    }

    #[test]
    fn a_unit_converts_only_when_the_file_states_one() {
        let r = Readout {
            unit: Some(LengthUnit::Millimetre),
            ..Readout::default()
        };
        assert_eq!(r.lengths_from(Some(1000.0), "m").length(1.5), "1500.000 mm");
        assert_eq!(r.lengths_from(None, "").length(1.5), "1.500");
        let both = Readout {
            second: Some(LengthUnit::Inch),
            ..r
        };
        assert_eq!(
            both.lengths_from(Some(25.4), "in").length(1.0),
            "25.400 mm (1.000 in)"
        );
        assert_eq!(both.lengths_from(None, "").length(1.0), "1.000");
    }

    #[test]
    fn a_plane_is_square_to_its_systems_axis() {
        let mut r = Readout {
            projection: 3,
            ..Readout::default()
        };
        assert_eq!(r.normal(None), Some([1.0, 0.0, 0.0]));
        r.projection = 1;
        assert_eq!(r.normal(Some([0.0, 0.0, 2.0])), Some([0.0, 0.0, 2.0]));
        r.projection = 0;
        assert_eq!(r.normal(Some([0.0, 0.0, 2.0])), None);
    }
}
