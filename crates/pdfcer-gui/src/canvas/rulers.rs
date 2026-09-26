//! # `canvas::rulers` — the ruler gutters, the tick ladder, and the drawing grid
//!
//! `RIBBON_IA.md` §5.2's View ▸ Display row *"Rulers · Grid · Guides"*, and
//! `FEATURES.md`'s last unbuilt Phase 3 line. This module owns two of the
//! three; [`super::guides`] owns the third and is built on the arithmetic
//! here.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/rulers.md`.

use egui::{Align, Layout, Pos2, Rect, Stroke, Ui, UiBuilder, pos2, vec2};
use pdfcer_core::dimension::{
    DEFAULT_GROUP_ID, NumberFormat, ScaleState, Unit, format_measurement,
};

use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::strip::PageView;

/// The outer thickness of each ruler gutter, in egui logical points.
pub(super) const THICKNESS_PTS: f32 = 22.0;

/// The shortest on-screen distance, in logical points, between two
/// **labelled** ticks.
pub(super) const MIN_MAJOR_PITCH_PTS: f32 = 76.0;

/// A hard ceiling on the ticks or grid lines drawn along one axis.
pub(super) const MAX_LINES: usize = 4_000;

/// How far a major tick runs in from the gutter's inner edge, in points.
const MAJOR_TICK_PTS: f32 = 6.0;

/// How far a minor tick runs in from the gutter's inner edge, in points.
const MINOR_TICK_PTS: f32 = 2.5;

/// The alpha, out of 255, of the tint marking the page's own span on a ruler.
const PAGE_SPAN_ALPHA: u8 = 40;

/// Named region: the horizontal ruler gutter, in window logical points.
const REGION_RULER_TOP: &str = "ruler-top"; // ui-text-exempt: trace region name, never displayed

/// Named region: the vertical ruler gutter.
const REGION_RULER_LEFT: &str = "ruler-left"; // ui-text-exempt: trace region name, never displayed

// ---------------------------------------------------------------------------
// The reservation
// ---------------------------------------------------------------------------

/// The rectangles a ruler-bearing canvas is divided into.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Gutters {
    /// The whole region the canvas was given, gutters included.
    pub(super) outer: Rect,
    /// The region the strip is laid out into.
    pub(super) content: Rect,
    /// The horizontal ruler, along the top of the content.
    pub(super) top: Option<Rect>,
    /// The vertical ruler, down the left of the content.
    pub(super) left: Option<Rect>,
    /// The square where the two meet, above and left of the content.
    ///
    /// Held as a **rect** rather than left implicit because the two rulers
    /// must not overlap in it: a tick drawn into the corner by both would be
    /// drawn twice, at two different alphas, and would read as a defect at
    /// exactly the place the eye starts.
    pub(super) corner: Option<Rect>,
}

impl Gutters {
    /// A child [`Ui`] covering [`Self::content`], which the whole of the
    /// canvas is then drawn into.
    pub(super) fn content_ui(self, ui: &mut Ui) -> Ui {
        let mut child = ui.new_child(
            UiBuilder::new()
                // ui-text-exempt: internal widget id, never displayed
                .id_salt("canvas-content")
                .max_rect(self.content)
                .layout(Layout::top_down(Align::Min)),
        );
        child.set_clip_rect(self.content.intersect(ui.clip_rect()));
        child
    }
}

/// Take the ruler gutters out of `ui`'s available space, and claim the whole
/// region in the parent's layout.
pub(super) fn reserve(ui: &mut Ui, show: bool) -> Gutters {
    let outer = ui.available_rect_before_wrap();
    ui.advance_cursor_after_rect(outer);

    let t = THICKNESS_PTS;
    // Three gutters' worth on each axis: two for the rulers and at least one
    // more for the canvas between them. Below that the canvas is not a canvas.
    let room = outer.width() > t * 3.0 && outer.height() > t * 3.0;
    if !show || !room {
        return Gutters {
            outer,
            content: outer,
            top: None,
            left: None,
            corner: None,
        };
    }

    let content = Rect::from_min_max(outer.min + vec2(t, t), outer.max);
    Gutters {
        outer,
        content,
        top: Some(Rect::from_min_max(
            pos2(content.min.x, outer.min.y),
            pos2(content.max.x, content.min.y),
        )),
        left: Some(Rect::from_min_max(
            pos2(outer.min.x, content.min.y),
            pos2(content.min.x, content.max.y),
        )),
        corner: Some(Rect::from_min_max(outer.min, content.min)),
    }
}

// ---------------------------------------------------------------------------
// What the frame learned
// ---------------------------------------------------------------------------

/// The geometry the rulers and the guides are drawn against, produced by the
/// canvas once its scroll area has settled.
pub(super) struct CanvasGeometry {
    /// Every page the frame drew, with its screen ⟷ canvas map.
    pub(super) pages: Vec<PageView>,
    /// The page the frame's input was about — the one the ruler's zero is
    /// pinned to.
    pub(super) current: usize,
    /// The scroll viewport, in screen coordinates. Both the region a pointer
    /// must be inside to count as "over the canvas" and the extent a guide
    /// preview is drawn across.
    pub(super) viewport: Rect,
}

impl CanvasGeometry {
    /// The map for `page`, if the frame drew it.
    pub(super) fn map_of(&self, page: usize) -> Option<PageMapping> {
        self.pages.iter().find(|p| p.page == page).map(|p| p.map)
    }

    /// The map for the page the ruler's zero is pinned to, falling back to
    /// whatever was drawn first.
    pub(super) fn anchor(&self) -> Option<PageMapping> {
        self.map_of(self.current)
            .or_else(|| self.pages.first().map(|p| p.map))
    }

    /// The page under a screen point, if the pointer is over one.
    pub(super) fn page_at(&self, screen: Pos2) -> Option<(usize, PageMapping)> {
        self.pages
            .iter()
            .find(|p| p.map.image_rect().contains(screen))
            .map(|p| (p.page, p.map))
    }
}

// ---------------------------------------------------------------------------
// The unit
// ---------------------------------------------------------------------------

/// What the ruler reads in: the document's own measurement scale and number
/// format.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Scale {
    state: ScaleState,
    format: NumberFormat,
}

impl Default for Scale {
    /// Raw PDF points — what a document that has never been calibrated reads
    /// in, and what every failure below falls back to.
    fn default() -> Self {
        Self {
            state: ScaleState::NeverSet,
            format: NumberFormat::decimal(Unit::Millimeter, 2),
        }
    }
}

impl Scale {
    /// Read the document's scale from its dimensioning sidecar.
    pub(super) fn of(doc: &OpenDoc) -> Self {
        doc.session
            .dimension_model()
            .group(DEFAULT_GROUP_ID)
            .map_or_else(Self::default, |g| Self {
                state: g.scale,
                format: g.format,
            })
    }

    /// Display units per PDF point.
    pub(super) fn units_per_point(self) -> f64 {
        match self.state.effective_scale(self.format.unit) {
            Some(s) if s.is_finite() && s > 0.0 => s,
            _ => 1.0,
        }
    }

    /// Render a canvas-space distance the way a dimension of that length would
    /// be rendered.
    ///
    /// Straight through [`pdfcer_core::dimension::format_measurement`] — see
    /// the header, §1, on why this module formats nothing itself.
    pub(super) fn label(self, points: f64) -> String {
        format_measurement(points, self.state, self.format).text
    }
}

// ---------------------------------------------------------------------------
// The ladder
// ---------------------------------------------------------------------------

/// The tick spacing for one view: how far apart the labelled ticks are, and
/// how far apart the unlabelled ones are.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Ladder {
    /// Distance between labelled ticks, in PDF points.
    pub(super) major: f64,
    /// Distance between unlabelled ticks, in PDF points.
    pub(super) minor: f64,
}

/// The smallest number of the form `1×10ⁿ`, `2×10ⁿ` or `5×10ⁿ` that is at
/// least `minimum`.
#[must_use]
pub(super) fn nice_step(minimum: f64) -> f64 {
    if !minimum.is_finite() || minimum <= 0.0 {
        return 1.0;
    }
    let decade = 10f64.powf(minimum.log10().floor());
    // Guard the decade as well: `log10` of a subnormal underflows, and the
    // division below would then be an infinity no later comparison catches.
    if !decade.is_finite() || decade <= 0.0 {
        return 1.0;
    }
    let mantissa = minimum / decade;
    let chosen = if mantissa <= 1.0 {
        1.0
    } else if mantissa <= 2.0 {
        2.0
    } else if mantissa <= 5.0 {
        5.0
    } else {
        10.0
    };
    chosen * decade
}

/// `Some(v)` when `v` is finite and positive, `None` otherwise.
fn positive(v: f64) -> Option<f64> {
    (v.is_finite() && v > 0.0).then_some(v)
}

/// How many minor ticks one major step is divided into, given that step's
/// 1-2-5 mantissa.
fn minor_divisions(major: f64) -> f64 {
    let decade = 10f64.powf(major.log10().floor());
    if !decade.is_finite() || decade <= 0.0 {
        return 5.0;
    }
    match (major / decade).round() as i64 {
        2 => 4.0,
        5 => 5.0,
        // 1, and the 10 `nice_step` can return from a rounding edge.
        _ => 10.0,
    }
}

impl Ladder {
    /// **The ruler's ladder**: labelled ticks at least `min_pitch_pts` apart on
    /// screen.
    #[must_use]
    pub(super) fn for_labels(scale: Scale, zoom: f32, min_pitch_pts: f32) -> Self {
        let upp = scale.units_per_point();
        let Some(zoom) = positive(f64::from(zoom)) else {
            return Self {
                major: 1.0,
                minor: 1.0,
            };
        };
        let major_units = nice_step(f64::from(min_pitch_pts) * upp / zoom);
        Self::from_major(major_units, upp)
    }

    /// **The grid's ladder**: every *drawn line* at least `min_pitch_pts`
    /// apart on screen — the **minor** step, not the major.
    #[must_use]
    pub(super) fn for_lines(scale: Scale, zoom: f32, min_pitch_pts: f32) -> Self {
        let upp = scale.units_per_point();
        let Some(zoom) = positive(f64::from(zoom)) else {
            return Self {
                major: 1.0,
                minor: 1.0,
            };
        };
        let min_units = f64::from(min_pitch_pts) * upp / zoom;
        let mut major_units = nice_step(min_units);
        // Bounded rather than `loop`: the arithmetic terminates in at most
        // three steps, and a bound means a NaN that slipped past `positive`
        // costs a wrong grid rather than a hung frame.
        for _ in 0..8 {
            if major_units / minor_divisions(major_units) >= min_units {
                break;
            }
            major_units = nice_step(major_units * 1.5);
        }
        Self::from_major(major_units, upp)
    }

    /// A ladder from its major step in **display units**, converted to points.
    fn from_major(major_units: f64, upp: f64) -> Self {
        let minor_units = major_units / minor_divisions(major_units);
        Self {
            major: major_units / upp,
            minor: minor_units / upp,
        }
    }

    /// The **index** of the first multiple of `step` at or after `from`.
    fn first_index(step: f64, from: f64) -> f64 {
        (from / step).ceil()
    }

    /// **Every minor tick between `from` and `to`, as `index × minor`.**
    pub(super) fn steps(self, from: f64, to: f64) -> impl Iterator<Item = f64> {
        let minor = self.minor;
        let first = Self::first_index(minor, from);
        let count = if minor > 0.0 && (to - from).is_finite() {
            (((to - from) / minor).floor() as i64 + 2).clamp(0, MAX_LINES as i64) as usize
        } else {
            0
        };
        (0..count)
            .map(move |i| (first + i as f64) * minor + 0.0)
            .take_while(move |v| *v <= to)
    }

    /// Whether `value` is a whole number of major steps from zero.
    pub(super) fn is_major(self, value: f64) -> bool {
        if self.major <= 0.0 || self.minor <= 0.0 {
            return false;
        }
        let steps = value / self.major;
        (steps - steps.round()).abs() * self.major < self.minor * 0.1
    }
}

// ---------------------------------------------------------------------------
// Painting
// ---------------------------------------------------------------------------

/// Which quantity a ruler measures, or which way a grid line runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Axis {
    /// Canvas **x** — the top ruler, and the vertical grid lines.
    X,
    /// Canvas **y** — the left ruler, and the horizontal grid lines.
    Y,
}

impl Axis {
    /// A point whose component on this axis is `value` and whose other
    /// component is zero.
    pub(super) fn point(self, value: f32) -> Pos2 {
        match self {
            Axis::X => pos2(value, 0.0),
            Axis::Y => pos2(0.0, value),
        }
    }

    /// The component of a screen point that this axis controls.
    pub(super) fn of(self, p: Pos2) -> f32 {
        match self {
            Axis::X => p.x,
            Axis::Y => p.y,
        }
    }
}

/// Draw both ruler gutters.
pub(super) fn draw(ui: &Ui, doc: &OpenDoc, gutters: Gutters, geometry: Option<&CanvasGeometry>) {
    let (Some(top), Some(left), Some(corner)) = (gutters.top, gutters.left, gutters.corner) else {
        return;
    };
    let visuals = ui.visuals();
    let painter = ui.painter().with_clip_rect(gutters.outer);

    // The chrome itself, painted before anything is measured — so a frame that
    // drew no page shows two rulers rather than two holes.
    for rect in [top, left, corner] {
        painter.rect_filled(rect, 0.0, visuals.panel_fill);
    }
    let edge = visuals.widgets.noninteractive.bg_stroke;
    painter.hline(top.x_range(), top.max.y, edge);
    painter.vline(left.max.x, left.y_range(), edge);

    crate::diag::ui_rect(REGION_RULER_TOP, top);
    crate::diag::ui_rect(REGION_RULER_LEFT, left);

    let Some(map) = geometry.and_then(CanvasGeometry::anchor) else {
        return;
    };
    let scale = Scale::of(doc);
    let ladder = Ladder::for_labels(scale, doc.view.zoom, MIN_MAJOR_PITCH_PTS);
    // The content-area selection ink by its role name; `overlay::ink`
    // carries the argument for why `visuals.selection` is not this canvas's
    // channel to read. Same colour, named address.
    let accent = egui_shell::theme::Theme::canvas_selection_ink(ui.ctx());

    // **The page's own span, as a TINT across the gutter** — not a line
    // along the gutter's inner edge.
    //
    // The single most useful thing a ruler can say about a drawing sheet is
    // where its borders are: at a fit zoom the paper's edge against the grey
    // surround is a one-pixel difference in fill, legible on a white sheet and
    // very nearly invisible on a dark theme. ⚠ A heavy line along the inner
    // edge says that and *sits exactly on top of the ticks*, which run 2.5
    // points in from the same edge — drowning every minor tick over the page,
    // which is every tick that matters, under the thing marking the page.
    //
    // A tint over the whole gutter says the same thing in a place nothing else
    // occupies, and it is the convention InDesign and Illustrator use for the
    // same purpose. Painted **before** the ticks so the ticks and labels sit on
    // top of it rather than under it.
    let page = map.image_rect();
    let tint = super::overlay::at_alpha(accent, PAGE_SPAN_ALPHA);
    painter.rect_filled(
        Rect::from_min_max(pos2(page.min.x, top.min.y), pos2(page.max.x, top.max.y)).intersect(top),
        0.0,
        tint,
    );
    painter.rect_filled(
        Rect::from_min_max(pos2(left.min.x, page.min.y), pos2(left.max.x, page.max.y))
            .intersect(left),
        0.0,
        tint,
    );

    ticks(ui, &painter, Axis::X, top, map, scale, ladder);
    ticks(ui, &painter, Axis::Y, left, map, scale, ladder);

    // Where the pointer is, on both rulers at once. A crosshair in the gutter
    // is how a ruler answers "how far across is this" without the operator
    // having to place a dimension to find out.
    if let Some(p) = ui.ctx().pointer_latest_pos()
        && geometry.is_some_and(|g| g.viewport.contains(p))
    {
        let stroke = Stroke::new(1.0, accent);
        painter.vline(p.x, top.y_range(), stroke);
        painter.hline(left.x_range(), p.y, stroke);
    }
}

/// Paint one ruler's ticks and labels.
fn ticks(
    ui: &Ui,
    painter: &egui::Painter,
    axis: Axis,
    gutter: Rect,
    map: PageMapping,
    scale: Scale,
    ladder: Ladder,
) {
    let span = match axis {
        Axis::X => gutter.x_range(),
        Axis::Y => gutter.y_range(),
    };
    // The gutter's two ends, in canvas units. Through the mapping, so the only
    // arithmetic here is on values the one screen ⟷ canvas conversion
    // produced.
    let from = f64::from(axis.of(map.to_page(axis.point(span.min))));
    let to = f64::from(axis.of(map.to_page(axis.point(span.max))));
    if !from.is_finite() || !to.is_finite() || to <= from || ladder.minor <= 0.0 {
        return;
    }

    let visuals = ui.visuals();
    let minor_stroke = Stroke::new(1.0, visuals.weak_text_color());
    let major_stroke = Stroke::new(1.0, visuals.text_color());
    let font = egui::TextStyle::Small.resolve(ui.style());

    for step in ladder.steps(from, to) {
        let value = step;
        let at = axis.of(map.to_screen(axis.point(value as f32)));
        let major = ladder.is_major(value);
        let (length, stroke) = if major {
            (MAJOR_TICK_PTS, major_stroke)
        } else {
            (MINOR_TICK_PTS, minor_stroke)
        };
        match axis {
            Axis::X => painter.vline(at, gutter.max.y - length..=gutter.max.y, stroke),
            Axis::Y => painter.hline(gutter.max.x - length..=gutter.max.x, at, stroke),
        };
        if major {
            let galley =
                painter.layout_no_wrap(scale.label(value), font.clone(), major_stroke.color);
            match axis {
                // Two points clear of the tick, and the label starts AT the
                // tick rather than being centred on it. Centring reads better
                // in isolation and is wrong here: at the gutter's left end
                // half of a centred label is clipped away, and half a number
                // is worse than a number sitting slightly to the right of the
                // line it names.
                Axis::X => painter.galley(
                    pos2(at + 2.0, gutter.min.y + 1.0),
                    galley,
                    major_stroke.color,
                ),
                // A quarter turn anticlockwise, which is what every vertical
                // ruler does and what lets a 22-point gutter hold a
                // ten-character label at all. `with_angle` rotates about the
                // shape's own position, so the anchor is offset by the
                // galley's length to make the text run *up* from the tick.
                Axis::Y => {
                    painter.add(
                        egui::epaint::TextShape::new(
                            pos2(gutter.min.x + 1.0, at + galley.size().x + 2.0),
                            galley,
                            major_stroke.color,
                        )
                        .with_angle(-std::f32::consts::FRAC_PI_2),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The gutters take a CONSTANT bite out of the viewport** — R128.
    #[test]
    fn the_gutters_are_a_constant_bite_out_of_the_viewport() {
        for (w, h) in [(400.0_f32, 300.0_f32), (1936.0, 1100.0), (91.0, 91.0)] {
            let outer = Rect::from_min_size(pos2(17.0, 29.0), egui::vec2(w, h));
            let g = gutters_for(outer, true);
            if g.top.is_none() {
                // Too small to hold rulers at all — the documented degenerate
                // answer, and it must leave the canvas whole.
                assert_eq!(g.content, outer);
                continue;
            }
            assert!(
                (g.content.width() - (w - THICKNESS_PTS)).abs() < f32::EPSILON,
                "the horizontal bite moved at {w}×{h}"
            );
            assert!(
                (g.content.height() - (h - THICKNESS_PTS)).abs() < f32::EPSILON,
                "the vertical bite moved at {w}×{h}"
            );
        }
    }

    /// With rulers off the canvas is byte-for-byte the rect it was handed.
    ///
    /// The mechanical form of "this feature does not change the default path".
    #[test]
    fn rulers_off_leaves_the_canvas_exactly_as_it_was() {
        let outer = Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(1000.0, 800.0));
        let g = gutters_for(outer, false);
        assert_eq!(g.content, outer);
        assert_eq!(g.top, None);
        assert_eq!(g.left, None);
        assert_eq!(g.corner, None);
    }

    /// The three gutters tile the region they take, without overlapping.
    #[test]
    fn the_two_rulers_do_not_overlap_each_other_or_the_canvas() {
        let outer = Rect::from_min_size(pos2(5.0, 7.0), egui::vec2(900.0, 700.0));
        let g = gutters_for(outer, true);
        let (top, left, corner) = (
            g.top.expect("a top ruler"),
            g.left.expect("a left ruler"),
            g.corner.expect("a corner"),
        );
        for (a, b) in [(top, left), (top, corner), (left, corner)] {
            let hit = a.intersect(b);
            assert!(
                hit.width() <= 0.0 || hit.height() <= 0.0,
                "gutters overlap: {a:?} vs {b:?}"
            );
        }
        for r in [top, left, corner] {
            let hit = r.intersect(g.content);
            assert!(
                hit.width() <= 0.0 || hit.height() <= 0.0,
                "a gutter overlaps the canvas: {r:?}"
            );
        }
    }

    /// A degenerate canvas turns the rulers off rather than clamping them into
    /// a picture with no room left for a page.
    #[test]
    fn a_canvas_too_small_for_rulers_draws_none() {
        let tiny = Rect::from_min_size(Pos2::ZERO, egui::vec2(THICKNESS_PTS * 2.0, 400.0));
        assert_eq!(gutters_for(tiny, true).content, tiny);
        let flat = Rect::from_min_size(Pos2::ZERO, egui::vec2(400.0, THICKNESS_PTS));
        assert_eq!(gutters_for(flat, true).content, flat);
    }

    /// [`reserve`]'s geometry, without a `Ui`.
    fn gutters_for(outer: Rect, show: bool) -> Gutters {
        let t = THICKNESS_PTS;
        let room = outer.width() > t * 3.0 && outer.height() > t * 3.0;
        if !show || !room {
            return Gutters {
                outer,
                content: outer,
                top: None,
                left: None,
                corner: None,
            };
        }
        let content = Rect::from_min_max(outer.min + vec2(t, t), outer.max);
        Gutters {
            outer,
            content,
            top: Some(Rect::from_min_max(
                pos2(content.min.x, outer.min.y),
                pos2(content.max.x, content.min.y),
            )),
            left: Some(Rect::from_min_max(
                pos2(outer.min.x, content.min.y),
                pos2(content.min.x, content.max.y),
            )),
            corner: Some(Rect::from_min_max(outer.min, content.min)),
        }
    }

    /// **The 1-2-5 ladder, exhaustively over one decade and across five.**
    #[test]
    fn the_tick_ladder_is_one_two_or_five_times_a_power_of_ten() {
        for exp in -3..=3 {
            let decade = 10f64.powi(exp);
            for k in 1..=99 {
                let want_at_least = decade * f64::from(k) / 10.0;
                let step = nice_step(want_at_least);
                assert!(
                    step >= want_at_least - 1e-9,
                    "nice_step({want_at_least}) = {step} is finer than asked"
                );
                let m = step / 10f64.powf(step.log10().floor());
                assert!(
                    (m - 1.0).abs() < 1e-9 || (m - 2.0).abs() < 1e-9 || (m - 5.0).abs() < 1e-9,
                    "nice_step({want_at_least}) = {step} has mantissa {m}"
                );
            }
        }
    }

    /// Degenerate inputs produce a usable ladder rather than a NaN one.
    #[test]
    fn a_degenerate_ladder_still_produces_finite_ticks() {
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!((nice_step(bad) - 1.0).abs() < f64::EPSILON);
        }
        let scale = Scale::default();
        for zoom in [0.0_f32, -1.0, f32::NAN, f32::INFINITY] {
            let l = Ladder::for_labels(scale, zoom, MIN_MAJOR_PITCH_PTS);
            assert!(l.major.is_finite() && l.major > 0.0, "zoom {zoom}");
            assert!(l.minor.is_finite() && l.minor > 0.0, "zoom {zoom}");
        }
    }

    /// **The on-screen pitch of the labelled ticks stays inside its band at
    /// every zoom on the ladder.**
    #[test]
    fn labelled_ticks_keep_a_readable_pitch_at_every_zoom() {
        let scale = Scale::default();
        for &zoom in crate::viewer::ZOOM_LADDER {
            let l = Ladder::for_labels(scale, zoom, MIN_MAJOR_PITCH_PTS);
            let pitch = l.major * f64::from(zoom);
            assert!(
                pitch >= f64::from(MIN_MAJOR_PITCH_PTS) - 1e-6,
                "at {zoom}× the labels are {pitch} pt apart, closer than the minimum"
            );
            assert!(
                pitch <= f64::from(MIN_MAJOR_PITCH_PTS) * 2.5 + 1e-6,
                "at {zoom}× the labels are {pitch} pt apart, further than one ladder step"
            );
        }
    }

    /// The minor ticks divide the major step into a whole number of parts, so
    /// every major tick is also a minor one and none is drawn twice.
    #[test]
    fn every_major_tick_lands_on_a_minor_tick() {
        let scale = Scale::default();
        for &zoom in crate::viewer::ZOOM_LADDER {
            let l = Ladder::for_labels(scale, zoom, MIN_MAJOR_PITCH_PTS);
            let parts = l.major / l.minor;
            assert!(
                (parts - parts.round()).abs() < 1e-6,
                "at {zoom}× a major step is {parts} minor steps"
            );
            assert!(l.is_major(0.0), "zero is always a labelled tick");
            assert!(l.is_major(l.major * 7.0), "the seventh major step");
            assert!(!l.is_major(l.minor), "one minor step is not a label");
        }
    }

    /// **A ruler with no scale set reads in points, and a calibrated one
    /// reads in its group's unit** — the header's §1 table, as a test.
    #[test]
    fn the_ruler_reads_points_until_the_document_says_otherwise() {
        let raw = Scale::default();
        assert!((raw.units_per_point() - 1.0).abs() < f64::EPSILON);
        assert_eq!(raw.label(144.0), "144.00 pt");
        assert_eq!(raw.label(-72.0), "-72.00 pt");

        // A sheet calibrated so one point is a quarter of a foot — the
        // worked example in `format_measurement`'s own documentation.
        let feet = Scale {
            state: ScaleState::Calibrated { scale: 0.25 },
            format: NumberFormat::decimal(Unit::DecimalFeet, 2),
        };
        assert_eq!(feet.label(144.0), "36.00 ft");
        assert!((feet.units_per_point() - 0.25).abs() < f64::EPSILON);

        // …and an explicit 1:1 metric group reads true size, which is a
        // different state from "never set" even though both are unscaled.
        let mm = Scale {
            state: ScaleState::OneToOne,
            format: NumberFormat::decimal(Unit::Millimeter, 2),
        };
        // ORACLE, NOT A CONVERSION: a whole inch of paper is 25.40 mm, stated
        // as the literal a reader can check against a ruler. The `25.4` is in
        // an expected string, not in arithmetic.
        assert_eq!(mm.label(72.0), "25.40 mm");
    }

    /// **A set scale moves the ladder into the operator's units**, so the
    /// numbers on the ruler are round in *their* system rather than in points.
    #[test]
    fn a_calibrated_sheet_gets_round_numbers_in_its_own_unit() {
        // One point = 0.0176 m, i.e. a sheet at roughly 1:50 in metres.
        let scale = Scale {
            state: ScaleState::Calibrated { scale: 0.0176 },
            format: NumberFormat::decimal(Unit::Meter, 3),
        };
        let l = Ladder::for_labels(scale, 1.0, MIN_MAJOR_PITCH_PTS);
        // The major step, converted back into the display unit, is round.
        let in_metres = l.major * scale.units_per_point();
        let m = in_metres / 10f64.powf(in_metres.log10().floor());
        assert!(
            (m - 1.0).abs() < 1e-6 || (m - 2.0).abs() < 1e-6 || (m - 5.0).abs() < 1e-6,
            "the ladder labelled {in_metres} m, which is not a round number"
        );
        // …and the same step in points is not round, which is exactly why the
        // ladder is chosen in display units rather than in points.
        assert!(
            (l.major - l.major.round()).abs() > 1e-9,
            "the point-space step happened to be round, so this asserts nothing"
        );
    }

    /// The tick walk starts inside the view rather than one step outside it.
    ///
    /// The index, not the value — see [`Ladder::steps`] on why the walk
    /// multiplies an integer.
    #[test]
    fn the_first_tick_index_lands_inside_the_view() {
        assert!((Ladder::first_index(50.0, -120.0) - -2.0).abs() < f64::EPSILON);
        assert!((Ladder::first_index(50.0, 100.0) - 2.0).abs() < f64::EPSILON);
        assert!((Ladder::first_index(50.0, 101.0) - 3.0).abs() < f64::EPSILON);
    }

    /// **The origin is labelled `0.00 pt`, never `-0.00 pt`.**
    #[test]
    fn the_origin_is_never_labelled_minus_zero() {
        let scale = Scale::default();
        let ladder = Ladder::for_labels(scale, 1.36, MIN_MAJOR_PITCH_PTS);
        // A gutter running from a little before the page's corner to well past
        // it — the geometry a view scrolled off the paper's edge produces.
        for from in [-1.5_f64, -18.4, -0.001, -999.0] {
            let zeroes: Vec<f64> = ladder.steps(from, 900.0).filter(|v| *v == 0.0).collect();
            assert_eq!(zeroes.len(), 1, "from {from}: the origin must be a tick");
            assert!(
                !zeroes[0].is_sign_negative(),
                "from {from}: the origin came out as negative zero"
            );
            assert_eq!(scale.label(zeroes[0]), "0.00 pt");
        }
    }

    /// **A long walk does not drift**, which is what makes the last tick on a
    /// wide sheet land where the first one promised.
    #[test]
    fn a_long_tick_walk_stays_exact() {
        let ladder = Ladder {
            major: 100.0,
            minor: 10.0,
        };
        let ticks: Vec<f64> = ladder.steps(0.0, 10_000.0).collect();
        assert_eq!(ticks.len(), 1001);
        for (i, v) in ticks.iter().enumerate() {
            assert!(
                (v - (i as f64) * 10.0).abs() < 1e-9,
                "tick {i} drifted to {v}"
            );
        }
        // …and every hundredth is still recognised as a label, which is the
        // property the drift would have broken first.
        assert_eq!(ticks.iter().filter(|v| ladder.is_major(**v)).count(), 101);
    }

    /// The walk is bounded even when the ladder is degenerate, so a bad zoom is
    /// a frame that draws slightly wrong rather than one that never finishes.
    #[test]
    fn the_tick_walk_is_bounded() {
        let ladder = Ladder {
            major: 1.0,
            minor: 0.000_001,
        };
        assert!(ladder.steps(0.0, 1e9).count() <= MAX_LINES);
        let broken = Ladder {
            major: 0.0,
            minor: 0.0,
        };
        assert_eq!(broken.steps(0.0, 100.0).count(), 0);
        assert_eq!(ladder.steps(f64::NAN, 10.0).count(), 0);
    }
}
