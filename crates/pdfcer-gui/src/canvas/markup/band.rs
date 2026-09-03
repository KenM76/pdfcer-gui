//! # `canvas::markup::band` — the two-point rubber band
//!
//! Rectangle, Ellipse, Arrow and Highlight: **press, drag out a shape,
//! release.** One of the four gesture families [`super`]'s header tabulates,
//! and the one that shipped first — this file is that original gesture, moved
//! out unchanged when [`super::vertex`] and [`super::ink`] arrived and it
//! stopped being the only one.
//!
//! ## ★ What moved, what did not, and why the seam is here
//!
//! Everything in this file was in `canvas/markup.rs` until 2026-08-14. What
//! stayed behind is *what a markup is* — the kinds, the geometry, `spec`,
//! `action`, the pen — and what came here is *how this family is gestured*: the
//! canvas→page conversion for two points, the one function that touches the
//! frame, and the band that is drawn while the button is down.
//!
//! **The seam is a subject and not a line count**, which matters because a line
//! count would have suggested a different cut. The test for it is the one
//! `tools/gates/check-file-size.sh` states in its own header: the two sides
//! change for different reasons. A new markup *kind* changes [`super`] — a
//! variant, an `rgb` arm, a `spec` arm — and does not touch this file unless it
//! is band-shaped. A change to *how a band is drawn* — a snap, a modifier that
//! constrains the aspect ratio, a different preview — changes this file and
//! nothing in [`super`].
//!
//! ## The band draws the shape it is about to author, not a box round it
//!
//! Rule 4's pre-commit affordance, applied literally: see [`draw_preview`],
//! which is where the argument lives, and which the old shell got wrong in a way
//! worth knowing about (it previewed an ellipse as the inscribed *circle*).
//!
//! ## A click with no drag places nothing
//!
//! [`super`]'s header carries that decision in full, including the two reasons
//! the old shell's 120 × 60 default box and its 4-point page-space threshold are
//! both deliberately absent. The mechanical half of it lives here: [`drag`] is
//! reached only from `GestureOutcome::Markup`, which only a real drag produces,
//! and a zero-extent drag is refused by [`super::action`].

use egui::{CornerRadius, Painter, Pos2, Stroke, StrokeKind};
use pdfcer_core::page_tree::Page;

use super::{Geometry, MarkupKind, Refusal};
use crate::app::actions::Action;
use crate::canvas::gesture::Phase;
use crate::canvas::mapping::PageMapping;
use crate::viewer;

/// A markup drag in flight, in **canvas space**, ready to be drawn.
///
/// Returned by [`drag`] only while the pointer is down, and only when the
/// release would commit — the same "the preview describes something that will
/// actually happen" contract [`crate::canvas::moving::drag`] honours with its
/// ghost.
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
///
/// # Why two point conversions and no arithmetic of our own
///
/// [`viewer::canvas_to_pdf_space`] applies the renderer's own page transform —
/// the crop-box origin, the `/Rotate`, and the Y flip. Writing any part of that
/// out here would be a second derivation of the page transform, which is the
/// precise failure `viewer`'s header warns about: *"PDF user space is y-UP;
/// canvas and screen are y-DOWN. The failure is silent — the page looks perfect
/// until someone selects a line and gets a different one."* For a markup the
/// symptom is worse than a mis-selection, because it is written to the file: a
/// rectangle dragged over the title block lands mirrored about the page's
/// horizontal centre line, and the operator finds out after saving.
///
/// Unlike [`crate::canvas::moving::page_delta`] this maps **positions**, not a
/// displacement, so the transform's translation is *not* cancelled — which is
/// the whole point. A markup has an absolute place on the page.
///
/// Returns `None` for a page whose device transform cannot be inverted, which
/// is the same condition under which both halves of the `viewer` bridge
/// decline.
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
///
/// The **only** function here that touches the frame. It does one of two
/// things:
///
/// * [`Phase::InFlight`] — returns the band for [`draw_preview`] and changes
///   nothing. Nothing is decomposed and nothing is re-rasterized: a markup drag
///   hit-tests nothing at all, which is why `canvas::interact` deliberately
///   leaves it out of the set of outcomes that need an object model. A preview
///   over a 129,758-object drawing costs one stroke.
/// * [`Phase::Complete`] — converts both endpoints to page space and pushes
///   exactly one [`Action::CommitMarkup`].
///
/// Returns `Some` only when a band should be drawn, and — as with the move
/// ghost — only when the release would actually commit. A drag with no page
/// under it draws nothing rather than a band that promises an annotation the
/// frame cannot author.
///
/// ★ **The first line is a guard on the family**, and it is not defensive
/// clutter: `canvas::interact` routes a freehand drag to [`super::ink`] before
/// reaching here, so a non-band kind arriving means the routing changed and this
/// function's two-point assumption stopped holding. Drawing nothing is the only
/// honest answer available — a band drawn between an ink stroke's first and
/// latest point is a rectangle the operator never asked for and the release
/// would author it.
///
/// # Why the refusal is traced only on release
///
/// An in-flight drag is re-evaluated 60 times a second, and the `canvas-pointer`
/// lesson — fifty identical lines in nine seconds from a stationary pointer —
/// is what a per-frame refusal trace would reproduce. The release is one event,
/// and it is the one a harness reading the trace is asking about.
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
            // ★ Traced with its COORDINATES, not a success flag — see
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
///
/// Screen-space, deliberately: the head is part of the *cursor*, and a head that
/// shrank to nothing at 25 % would stop saying which end of the band is the
/// head — which is the one thing this preview exists to say. The committed
/// annotation's own `/LE` head is drawn by the appearance stream at whatever
/// size the engine chooses; this is not a promise about that size, it is a
/// statement about direction.
const HEAD_LEN_PX: f32 = 14.0;
/// Half-angle of the arrowhead, in radians (≈ 24°).
const HEAD_ANGLE: f32 = 0.42;

/// Paint the markup band, given the [`Preview`] [`drag`] returned.
///
/// # ★ Why this is not `draw_marquee` with a different colour
///
/// Because a marquee and a markup band answer different questions. A marquee
/// asks *"what does this rectangle enclose?"* and is therefore always a
/// rectangle whatever it is about to select. A markup band asks *"is this the
/// shape you meant?"*, and the only way it can answer is by **being that
/// shape**: an ellipse drawn as its bounding box misstates the geometry by the
/// difference between a box and the ellipse inside it — 21 % of the area — and
/// an arrow drawn as a plain segment says nothing about which end the head is
/// on, which is the single most reversible property of the thing being
/// committed.
///
/// The old shell previewed a Circle as `circle_stroke` with the *smaller* of
/// the two half-extents, i.e. as the inscribed **circle** rather than the
/// ellipse it was about to author. That is drawn correctly here instead: on a
/// wide drag the two differ by the whole aspect ratio, and the operator would
/// have released expecting the circle they were shown.
///
/// # The colours are document colours, and that is why they are literals
///
/// Everything painted here is the pen — the colour and width that are about to
/// be written into the file. Reading it from [`egui::Visuals`] would be wrong in
/// the way `check-theme-colors.sh`'s own header describes: restyling the
/// application would change the colour of markup about to be committed, and the
/// change would only become visible after saving.
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
        // ★ Not reachable, and spelled rather than wildcarded so a NINTH kind
        // has to be classified here rather than silently drawing nothing. That
        // is not a hypothetical any more: `MarkupKind::Cloud` landed on
        // 2026-08-19 and this arm is one of exactly two places in the crate the
        // compiler stopped it, which is what the spelling was for.
        //
        // `drag` refuses a non-band kind at its first line, so no `Preview` can
        // carry one; the four that land here draw their own previews, in the
        // module that owns their gesture, because neither a freehand trail nor a
        // vertex run is describable by two points.
        MarkupKind::PolyLine | MarkupKind::Polygon | MarkupKind::Cloud | MarkupKind::Ink => {}
    }
}

/// The two barb endpoints of the preview arrowhead at `head`.
///
/// Returns an empty array's worth of coincident points for a zero-length band,
/// which draws nothing — a head with no direction to point in must not be
/// invented from a normalised zero vector (which would be NaN).
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
///
/// `OPERATOR_REQUESTS.md` **O54**. The sibling of [`draw_preview`] for the
/// gesture that found text under it.
///
/// ★★ It uses `highlight_wash` — the identical colour the area band draws —
/// because they are one feature reached by one tool. A preview that changed
/// colour depending on whether the pointer had found text would tell the
/// operator they had switched tools when they had not.
///
/// ★ Rectangles rather than quads: the quads a text sweep produces are already
/// axis-aligned per line in canvas space, which is what `TextSelection::
/// highlights` hands back. A rotated run is drawn by the committed appearance
/// stream, not by this — the same division `draw_preview` makes.
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
            rotate,
            contents: Vec::new(),
            contents_unresolved: 0,
            contents_flattened: 0,
        }
    }

    // -----------------------------------------------------------------
    // ★ The defect this whole module exists to prevent
    // -----------------------------------------------------------------

    /// ★ **The markup lands where the operator dragged, not at the page
    /// centre.**
    ///
    /// The regression test for *"they just drop things into the center of the
    /// pdf window."* It is written as a **magnitude** assertion against the
    /// dragged corners and, separately, as a statement that the result is
    /// nowhere near the media-box centre — because `HANDOFF.md` §2's lesson is
    /// that a test asserting a relation rather than a magnitude is satisfied by
    /// any absurdity in the right direction. "The shape is on the page" would
    /// have passed on the defective build; "the shape's corners ARE the corners
    /// dragged" cannot.
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
    ///
    /// This is the markup gesture's form of
    /// `moving::a_drag_between_two_page_points_moves_the_same_distance_at_every_zoom`,
    /// and it is the stronger of the two statements: a move only has to be the
    /// same *displacement* at every zoom, while a markup has to land on the same
    /// *absolute* page coordinates.
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

    /// ★ **A click with no drag never reaches this module at all**, and the
    /// degenerate drag it would look like is refused.
    ///
    /// The module docs' decision, pinned from both ends: the gesture machine
    /// raises `Click` (not a `DragKind`) for a press-and-release under egui's
    /// threshold, and if a zero-extent drag does arrive it commits nothing.
    /// Without the second half, "a click places nothing" would rest on egui's
    /// behaviour alone.
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

    /// ★ **A non-band kind draws no band and authors nothing here.**
    ///
    /// The guard on the first line of [`drag`], asserted because its absence is
    /// silent: an ink drag routed here by mistake would draw a rectangle between
    /// the stroke's first and latest points and would **author** one on release
    /// — a perfectly valid `/Square` that the operator did not draw, over the
    /// region their freehand stroke happened to span.
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

    /// ★ **The preview's arrowhead is at the head end**, whichever way the
    /// operator drags — the on-screen half of the raw-endpoint rule.
    ///
    /// Asserted as a distance, not as a side: both barbs must be within a barb's
    /// length of the head and nowhere near the tail. A "the head is drawn"
    /// assertion would pass on an implementation that drew it at the wrong end.
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
    ///
    /// Compared as the *whole* `Color32` against a value rebuilt from the pen,
    /// rather than channel by channel, because `egui::Color32` stores
    /// **premultiplied** components: `r()` on a translucent colour returns the
    /// multiplied byte, so a per-channel comparison against the opaque pen fails
    /// for a wash that is completely correct. Worth the note — the first version
    /// of this test made exactly that mistake and reported a working wash as
    /// broken.
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
