//! # `canvas::markup::band` — the two-point rubber band
//!
//! Rectangle, Ellipse, Arrow and Highlight: **press, drag out a shape,
//! release.** One of the four gesture families [`super`]'s header tabulates;
//! [`super::vertex`] and [`super::ink`] own the other shapes.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/markup/band.md`.

use egui::{CornerRadius, Painter, Pos2, Stroke, StrokeKind};
use pdfcer_core::page_tree::Page;

use super::{Geometry, MarkupKind, Refusal};
use crate::app::actions::Action;
use crate::canvas::gesture::Phase;
use crate::canvas::mapping::PageMapping;
use crate::viewer;

/// A markup drag in flight, in **canvas space**, ready to be drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Preview {
    /// Which shape is being authored.
    pub kind: MarkupKind,
    /// Where the press landed. For [`MarkupKind::Arrow`] this is the **tail**.
    pub from: Pos2,
    /// Where the pointer is now. For [`MarkupKind::Arrow`] this is the **head**.
    pub to: Pos2,
}

/// Convert a **canvas-space** drag into a pair of **PDF user-space** endpoints.
#[must_use]
pub fn endpoints(from: Pos2, to: Pos2, page: &Page) -> Option<((f64, f64), (f64, f64))> {
    let start = viewer::canvas_to_pdf_space(from, page)?;
    let end = viewer::canvas_to_pdf_space(to, page)?;
    Some((
        (f64::from(start.x), f64::from(start.y)),
        (f64::from(end.x), f64::from(end.y)),
    ))
}

/// Apply one frame of a markup drag: return the preview, or commit the markup.
#[allow(
    clippy::too_many_arguments,
    reason = "a gesture entry point's inputs are eight independent facts about one frame — the pen, the armed kind, two pointer positions, the page, its geometry, the phase and the action queue. Grouping any subset into a struct would be grouping by arity rather than by meaning, and the resulting type would have no name that was true." // ui-text-exempt: lint justification, never displayed
)]
pub fn drag(
    pen: super::pen::Pen,
    kind: MarkupKind,
    from: Pos2,
    to: Pos2,
    phase: Phase,
    page_index: usize,
    page: Option<&Page>,
    actions: &mut Vec<Action>,
) -> Option<Preview> {
    if !kind.is_band() {
        return None;
    }
    let Some(page) = page else {
        if phase == Phase::Complete {
            super::decline(kind, page_index, Refusal::NoPage);
        }
        return None;
    };
    let Some((start, end)) = endpoints(from, to, page) else {
        if phase == Phase::Complete {
            super::decline(kind, page_index, Refusal::DegeneratePage);
        }
        return None;
    };

    if phase == Phase::InFlight {
        return Some(Preview { kind, from, to });
    }

    match super::action(kind, page_index, Geometry::Band { start, end }, pen) {
        Ok(raised) => {
            // Traced with its COORDINATES, not a success flag — see
            // `super::trace_commit`. The RAW endpoints, in drag order, so a
            // harness can prove the arrow's head is at the end the operator
            // dragged to, which a normalised rect could not express.
            super::trace_commit(
                kind,
                page_index,
                &format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "x0={:.2} y0={:.2} x1={:.2} y1={:.2}",
                    start.0, start.1, end.0, end.1
                ),
            );
            actions.push(raised);
        }
        Err(reason) => super::decline(kind, page_index, reason),
    }
    None
}

/// The number of segments an ellipse preview is drawn with.
///
/// 48 is enough that the polyline is indistinguishable from a curve at any zoom
/// this canvas reaches, and it is cheap: one band, once per frame of one drag.
const ELLIPSE_SEGMENTS: usize = 48;

/// The arrowhead barb length, in **screen** points, and the angle it opens at.
const HEAD_LEN_PX: f32 = 14.0;
/// Half-angle of the arrowhead, in radians (≈ 24°).
const HEAD_ANGLE: f32 = 0.42;

/// Paint the markup band, given the [`Preview`] [`drag`] returned.
pub fn draw_preview(
    painter: &Painter,
    mapping: &PageMapping,
    preview: Preview,
    pen: super::pen::Pen,
) {
    let Preview { kind, from, to } = preview;
    let (a, b) = (mapping.to_screen(from), mapping.to_screen(to));
    let stroke = Stroke::new(super::pen_px(mapping, pen), super::pen_color(kind, pen));

    match kind {
        MarkupKind::Rectangle => {
            painter.rect_stroke(
                egui::Rect::from_two_pos(a, b),
                CornerRadius::ZERO,
                stroke,
                StrokeKind::Middle,
            );
        }
        MarkupKind::Ellipse => {
            let rect = egui::Rect::from_two_pos(a, b);
            let (cx, cy) = (rect.center().x, rect.center().y);
            let (rx, ry) = (rect.width() / 2.0, rect.height() / 2.0);
            let mut points: Vec<Pos2> = (0..=ELLIPSE_SEGMENTS)
                .map(|i| {
                    #[allow(clippy::cast_precision_loss)]
                    let t = (i as f32) / (ELLIPSE_SEGMENTS as f32) * std::f32::consts::TAU;
                    Pos2::new(cx + rx * t.cos(), cy + ry * t.sin())
                })
                .collect();
            // Close it exactly rather than relying on the last sample landing
            // on the first: a visible seam in a preview reads as a shape that
            // did not close.
            if let (Some(first), Some(last)) = (points.first().copied(), points.last_mut()) {
                *last = first;
            }
            painter.add(egui::Shape::line(points, stroke));
        }
        MarkupKind::Arrow => {
            painter.line_segment([a, b], stroke);
            for barb in arrowhead(a, b) {
                painter.line_segment([b, barb], stroke);
            }
        }
        // A wash, not an outline: a highlight IS a translucent fill, and an
        // outlined empty box would describe a rectangle annotation instead.
        MarkupKind::Highlight => {
            painter.rect_filled(
                egui::Rect::from_two_pos(a, b),
                CornerRadius::ZERO,
                highlight_wash(kind, pen),
            );
        }
        // Not reachable, and spelled rather than wildcarded so a NINTH kind
        // has to be classified here rather than silently drawing nothing. This
        // arm is one of exactly two places in the crate where the compiler
        // stops a newly added `MarkupKind`, which is what the spelling is for.
        //
        // `drag` refuses a non-band kind at its first line, so no `Preview` can
        // carry one; the four that land here draw their own previews, in the
        // module that owns their gesture, because neither a freehand trail nor a
        // vertex run is describable by two points.
        MarkupKind::PolyLine | MarkupKind::Polygon | MarkupKind::Cloud | MarkupKind::Ink => {}
    }
}

/// The two barb endpoints of the preview arrowhead at `head`.
fn arrowhead(tail: Pos2, head: Pos2) -> [Pos2; 2] {
    let dir = head - tail;
    let len = dir.length();
    if !len.is_finite() || len <= f32::EPSILON {
        return [head, head];
    }
    let back = -dir / len;
    let (s, c) = (HEAD_ANGLE.sin(), HEAD_ANGLE.cos());
    let rot = |x: f32, y: f32| Pos2::new(head.x + x * HEAD_LEN_PX, head.y + y * HEAD_LEN_PX);
    [
        rot(back.x * c - back.y * s, back.x * s + back.y * c),
        rot(back.x * c + back.y * s, -back.x * s + back.y * c),
    ]
}

/// The highlight preview's fill: the pen colour at the alpha a highlight reads
/// at over content.
fn highlight_wash(kind: MarkupKind, pen: super::pen::Pen) -> egui::Color32 {
    let c = super::pen_color(kind, pen);
    // DOCUMENT COLOUR: arithmetic on the pen colour above, not a second choice
    // of colour. The alpha is a legibility figure for the *preview* — the
    // committed annotation's translucency is the engine's `/CA`, which pdfcer
    // does not yet write (filed in `open/`), so this states "a highlight" and
    // does not promise a specific opacity.
    egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 90)
}

/// **Paint a text-following highlight's preview: one wash per line.**
pub fn draw_text_marks(
    painter: &egui::Painter,
    map: &crate::canvas::mapping::PageMapping,
    marks: &[egui::Rect],
    pen: super::pen::Pen,
) {
    let wash = highlight_wash(MarkupKind::Highlight, pen);
    for mark in marks {
        painter.rect_filled(map.rect_to_screen(*mark), CornerRadius::ZERO, wash);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::annot_author::MarkupSpec;
    use pdfcer_core::object::{Dict, ObjId};
    use pdfcer_core::page_tree::Rect as PageRect;

    /// A minimal page fixture — the same one `viewer`'s and `moving`'s geometry
    /// tests use, because these functions read exactly what those do:
    /// `crop_box` and `rotate`.
    fn test_page(w: f64, h: f64, rotate: u16) -> Page {
        Page {
            id: ObjId::new(1, 0),
            resources: Dict::new(),
            media_box: PageRect::from_corners(0.0, 0.0, w, h),
            crop_box: PageRect::from_corners(0.0, 0.0, w, h),
            crop_box_resolution: pdfcer_core::page_tree::BoxResolution::Defaulted,
            bleed_box: PageRect::from_corners(0.0, 0.0, w, h),
            bleed_box_resolution: pdfcer_core::page_tree::BoxResolution::Defaulted,
            trim_box: PageRect::from_corners(0.0, 0.0, w, h),
            trim_box_resolution: pdfcer_core::page_tree::BoxResolution::Defaulted,
            art_box: PageRect::from_corners(0.0, 0.0, w, h),
            art_box_resolution: pdfcer_core::page_tree::BoxResolution::Defaulted,
            rotate,
            contents: Vec::new(),
            contents_unresolved: 0,
            resources_defaulted: false,
            contents_flattened: 0,
        }
    }

    // -----------------------------------------------------------------
    // The defect this whole module exists to prevent
    // -----------------------------------------------------------------

    /// **The markup lands where the operator dragged, not at the page
    /// centre.**
    #[test]
    fn the_markup_lands_where_the_drag_was_and_not_at_the_page_centre() {
        let page = test_page(612.0, 792.0, 0);
        // A drag in the lower-left quadrant of the canvas, i.e. the UPPER-left
        // of the page in PDF space.
        let (start, end) =
            endpoints(Pos2::new(72.0, 90.0), Pos2::new(200.0, 150.0), &page).expect("invertible");

        assert!((start.0 - 72.0).abs() < 1e-3, "{start:?}");
        assert!((end.0 - 200.0).abs() < 1e-3, "{end:?}");
        // Canvas Y is down, PDF Y is up: 90 from the top of a 792-high page is
        // 702 from the bottom.
        assert!((start.1 - 702.0).abs() < 1e-3, "{start:?}");
        assert!((end.1 - 642.0).abs() < 1e-3, "{end:?}");

        let Some(MarkupSpec::Square { rect, .. }) =
            super::super::spec_default_pen(MarkupKind::Rectangle, &Geometry::Band { start, end })
        else {
            panic!("Rectangle must author a /Square");
        };
        let (cx, cy) = ((rect.llx + rect.urx) / 2.0, (rect.lly + rect.ury) / 2.0);
        assert!(
            (cx - 306.0).abs() > 100.0 && (cy - 396.0).abs() > 100.0,
            "the shape drifted toward the page centre: centre=({cx}, {cy})"
        );
    }

    /// The same drag, at four magnifications, through the frame's real
    /// mapping — because the pointer only ever reports **screen** positions and
    /// a stray zoom would enter exactly there.
    #[test]
    fn the_same_drag_authors_the_same_page_coordinates_at_every_zoom() {
        use crate::viewer::page_extent_pts;

        let page = test_page(612.0, 792.0, 0);
        let extent = page_extent_pts(&page);
        let (grabbed, dropped) = (Pos2::new(100.0, 120.0), Pos2::new(260.0, 300.0));

        let mut seen: Vec<((f64, f64), (f64, f64))> = Vec::new();
        for &zoom in &[0.25_f32, 1.0, 4.0, 12.0] {
            let image_rect = egui::Rect::from_min_size(
                Pos2::new(37.0, 11.0),
                egui::vec2(extent.0 * zoom, extent.1 * zoom),
            );
            let map = PageMapping::new(image_rect, extent, zoom);
            let from = map.to_page(map.to_screen(grabbed));
            let to = map.to_page(map.to_screen(dropped));
            seen.push(endpoints(from, to, &page).expect("invertible"));
        }
        for got in &seen {
            assert!(
                (got.0.0 - seen[0].0.0).abs() < 1e-2
                    && (got.0.1 - seen[0].0.1).abs() < 1e-2
                    && (got.1.0 - seen[0].1.0).abs() < 1e-2
                    && (got.1.1 - seen[0].1.1).abs() < 1e-2,
                "the page coordinates changed with the zoom: {seen:?}"
            );
        }
        // …and they are the right coordinates, not merely consistent ones.
        assert!((seen[0].0.0 - 100.0).abs() < 1e-2, "{seen:?}");
        assert!((seen[0].0.1 - 672.0).abs() < 1e-2, "{seen:?}");
        assert!((seen[0].1.0 - 260.0).abs() < 1e-2, "{seen:?}");
        assert!((seen[0].1.1 - 492.0).abs() < 1e-2, "{seen:?}");
    }

    /// A rotated page rotates the placement, through the renderer's own
    /// transform rather than a formula written out here.
    #[test]
    fn a_rotated_page_places_the_markup_through_the_page_transform() {
        let upright = test_page(612.0, 792.0, 0);
        let turned = test_page(612.0, 792.0, 90);
        let at = Pos2::new(100.0, 120.0);
        let a = endpoints(at, at + egui::vec2(10.0, 10.0), &upright).expect("invertible");
        let b = endpoints(at, at + egui::vec2(10.0, 10.0), &turned).expect("invertible");
        assert_ne!(
            a, b,
            "a 90° page must not author the same coordinates as an upright one"
        );
    }

    // -----------------------------------------------------------------
    // The gesture
    // -----------------------------------------------------------------

    /// **A click with no drag never reaches this module at all**, and the
    /// degenerate drag it would look like is refused.
    #[test]
    fn a_click_places_nothing_and_the_degenerate_drag_it_resembles_is_refused() {
        use crate::canvas::gesture::{
            DragKind, GestureOutcome, GestureState, PointerFrame, PressMeaning,
        };

        let mut gestures = GestureState::default();
        let out = gestures.update(
            PointerFrame {
                clicked: true,
                pos: Some(Pos2::new(150.0, 150.0)),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Markup(MarkupKind::Rectangle)),
        );
        assert!(
            matches!(out, GestureOutcome::Click { .. }),
            "a click must stay a click: {out:?}"
        );

        let page = test_page(612.0, 792.0, 0);
        let mut actions = Vec::new();
        let at = Pos2::new(150.0, 150.0);
        let preview = drag(
            crate::canvas::markup::pen::Pen::default(),
            MarkupKind::Rectangle,
            at,
            at,
            Phase::Complete,
            0,
            Some(&page),
            &mut actions,
        );
        assert_eq!(preview, None);
        assert!(
            actions.is_empty(),
            "a zero-extent drag must author nothing, not a default-sized box"
        );
    }

    /// An in-flight drag previews and commits nothing; the release commits
    /// exactly one action and previews nothing.
    #[test]
    fn a_markup_draws_in_flight_and_commits_once() {
        let page = test_page(612.0, 792.0, 0);
        let (from, to) = (Pos2::new(50.0, 60.0), Pos2::new(150.0, 200.0));

        let mut actions = Vec::new();
        let preview = drag(
            crate::canvas::markup::pen::Pen::default(),
            MarkupKind::Ellipse,
            from,
            to,
            Phase::InFlight,
            2,
            Some(&page),
            &mut actions,
        );
        assert_eq!(
            preview,
            Some(Preview {
                kind: MarkupKind::Ellipse,
                from,
                to
            })
        );
        assert!(actions.is_empty(), "an in-flight drag must not commit");

        let preview = drag(
            crate::canvas::markup::pen::Pen::default(),
            MarkupKind::Ellipse,
            from,
            to,
            Phase::Complete,
            2,
            Some(&page),
            &mut actions,
        );
        assert_eq!(preview, None, "a released drag draws no band");
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            actions[0],
            Action::CommitMarkup {
                page: 2,
                kind: MarkupKind::Ellipse,
                ..
            }
        ));
    }

    /// **A non-band kind draws no band and authors nothing here.**
    #[test]
    fn a_non_band_kind_is_refused_by_the_band_gesture() {
        let page = test_page(612.0, 792.0, 0);
        for kind in [MarkupKind::Ink, MarkupKind::PolyLine, MarkupKind::Polygon] {
            let mut actions = Vec::new();
            for phase in [Phase::InFlight, Phase::Complete] {
                assert_eq!(
                    drag(
                        crate::canvas::markup::pen::Pen::default(),
                        kind,
                        Pos2::new(10.0, 10.0),
                        Pos2::new(200.0, 160.0),
                        phase,
                        0,
                        Some(&page),
                        &mut actions,
                    ),
                    None,
                    "{kind:?}"
                );
            }
            assert!(actions.is_empty(), "{kind:?} must author nothing here");
        }
    }

    /// With no page under it, a markup drag draws nothing and commits nothing —
    /// a band that promised an annotation the frame cannot author would be the
    /// dishonest preview rule 4 forbids.
    #[test]
    fn a_frame_with_no_page_draws_no_band_and_commits_nothing() {
        let mut actions = Vec::new();
        for phase in [Phase::InFlight, Phase::Complete] {
            assert_eq!(
                drag(
                    crate::canvas::markup::pen::Pen::default(),
                    MarkupKind::Arrow,
                    Pos2::ZERO,
                    Pos2::new(10.0, 10.0),
                    phase,
                    0,
                    None,
                    &mut actions,
                ),
                None
            );
        }
        assert!(actions.is_empty());
    }

    /// **The preview's arrowhead is at the head end**, whichever way the
    /// operator drags — the on-screen half of the raw-endpoint rule.
    #[test]
    fn the_preview_arrowhead_sits_at_the_head_whichever_way_the_drag_went() {
        for (tail, head) in [
            (Pos2::new(10.0, 10.0), Pos2::new(200.0, 120.0)),
            (Pos2::new(200.0, 120.0), Pos2::new(10.0, 10.0)),
            (Pos2::new(200.0, 10.0), Pos2::new(10.0, 120.0)),
        ] {
            for barb in arrowhead(tail, head) {
                assert!(
                    (barb - head).length() <= HEAD_LEN_PX + 1e-3,
                    "a barb landed {} from the head",
                    (barb - head).length()
                );
                assert!(
                    (barb - tail).length() > HEAD_LEN_PX,
                    "a barb landed at the TAIL: the arrow is drawn backwards"
                );
            }
        }
    }

    /// A zero-length band produces no head rather than a NaN one.
    #[test]
    fn a_zero_length_band_has_no_arrowhead() {
        let at = Pos2::new(5.0, 5.0);
        assert_eq!(arrowhead(at, at), [at, at]);
    }

    /// The highlight wash is the pen colour with an alpha, not a second choice
    /// of colour — so restyling the application cannot move it and the wash
    /// cannot disagree with the `/C` that lands in the file.
    #[test]
    fn the_highlight_wash_is_the_pen_colour_with_an_alpha() {
        let pen = super::super::pen_color(
            MarkupKind::Highlight,
            crate::canvas::markup::pen::Pen::default(),
        );
        let wash = highlight_wash(
            MarkupKind::Highlight,
            crate::canvas::markup::pen::Pen::default(),
        );
        assert_eq!(
            wash,
            // NOT A THEME COLOUR: a test fixture rebuilding the value under test
            // from the pen it is asserted to be derived from. Nothing here is
            // drawn.
            egui::Color32::from_rgba_unmultiplied(pen.r(), pen.g(), pen.b(), wash.a()),
            "the wash must be the pen with an alpha, not a second choice of colour"
        );
        assert!(
            wash.a() < 255,
            "a highlight that hides its content is a fill"
        );
        assert!(wash.a() > 0, "…and one nobody can see is not a highlight");
    }
}
