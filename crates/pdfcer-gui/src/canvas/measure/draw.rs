//! **What the measure tools draw**, and the one projection they draw through.
//!
//! The parent decides what a click *means* — which point is picked, whether the
//! gesture is finishable, what action it commits. This module decides what the
//! operator sees before any of that has happened: the snap indicator, the
//! hover highlight, the rubber-band, the committed-pick rings and the fitted
//! circle. R8b calls those the cursor, and they are the one class of mark the
//! canvas is allowed to carry that is not content.
//!
//! ## Why the projection lives here rather than with the picking
//!
//! [`page_to_screen`] is the only place in `canvas/measure/` that crosses from
//! PDF user space to the painter's space, and every mark this module makes goes
//! through it. Picking never needs it: a pick is resolved in page space and
//! compared in page space. So the conversion is a property of drawing, and its
//! own header records the defect that follows from splitting it — a mark
//! authored in the right place and drawn in the wrong one, invisible to every
//! test about *which* point is picked.
//!
//! ## Reach
//!
//! `Preview`, `preview`, `SNAP_MARKER_PT` and `page_to_screen` are spelled
//! `pub(in crate::canvas)` rather than `pub(super)`, and the distinction is
//! load-bearing: `super` here is `crate::canvas::measure`, one level narrower
//! than it was in the parent, and `canvas::painting` — which calls all four —
//! sits outside it. The parent re-exports them so `measure::preview` and
//! `measure::page_to_screen` stay the spelling every caller and every citation
//! uses.

use egui::{Pos2, Ui};
use pdfcer_core::vector::Point;

use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::snap;
use crate::viewer;

use super::state::MeasureState;
use super::{MeasureKind, Resolved, hover, pick, read};

/// **Draw what the next click would commit.**
///
/// Rule 4's pre-commit affordance: *"a snap indicator, a hover highlight, a
/// rubber-band … these are the cursor; they describe what is about to
/// happen."* It is only honest if it is derived from the values the commit will
/// use, which is why the placing preview goes through
/// [`pick::dimension_preview_segments`] — the *same* function a committed
/// dimension is drawn from — rather than drawing a line of its own.
/// # ★ The circular tool's preview is the whole of its feedback
///
/// The other two tools draw something that follows the pointer, so an operator
/// can see the gesture working. The circular tool's pick lands on geometry that
/// is *already drawn* — toggling an arc into the fit changes nothing on screen
/// unless this function says so — and its answer is a circle nobody has drawn
/// at all. Without both halves the operator cannot tell a set of three arcs
/// that fits their hole from one that has accidentally caught the leader line
/// beside it, and the residual is invisible until the dimension is already on
/// the page.
///
/// So two things are drawn, and the second goes through
/// [`pick::dimension_preview_segments`]:
///
/// 1. **A marker on every picked point**, straight out of
///    [`pick::CircularPick::points`]. ★ A rectangle round every picked *object*
///    is the wrong picture: on a real drawing one click outlines a 550 × 500 pt
///    region — see `pick::CircularPick`'s header and `OPERATOR_REQUESTS.md`
///    O105. A marker per point is both the honest
///    picture of what is in the fit and the thing an operator aims at to take a
///    point back out.
/// 2. **The fitted circle**, derived by handing the *same*
///    [`pick::CircularPick::author`] value the commit would use to the *same*
///    segment function a committed dimension is drawn from. That is this
///    module's standing rule and it matters more here than anywhere else: the
///    fit is an inference, and an inference previewed by a second derivation is
///    an inference the operator cannot actually check.
pub(in crate::canvas) struct Preview<'a> {
    /// The open document, for the page transform.
    pub doc: &'a OpenDoc,
    /// The page being drawn on.
    pub page_index: usize,
    /// Which measure tool is armed.
    pub kind: MeasureKind,
    /// The frame's screen ⟷ canvas map. **The projection every mark here goes
    /// through**, and the reason this struct exists — see [`preview`].
    pub map: &'a PageMapping,
    /// Where the pointer would pick, resolved once for the frame.
    pub hover: Option<Resolved>,
}

pub(in crate::canvas) fn preview(ui: &Ui, preview: Preview<'_>) {
    let Preview {
        doc,
        page_index,
        kind,
        map,
        hover,
    } = preview;
    let Some(page) = doc.current_page() else {
        return;
    };
    let ctx = ui.ctx();
    // ★★ The SECOND instance of the same bail, and it is why fixing
    // `resolve_hover` alone changed nothing.
    //
    // `read` returns `None` until the operator has clicked once, because
    // [`load`] builds a default and only the click paths [`store`] it. This
    // function returned early on that, so a freshly armed tool painted no snap
    // marker and no hover highlight — the exact state the operator reported,
    // and the exact state the comment forty lines below promises is handled.
    //
    // A driven run found `resolve_hover` producing `entity=1 snap=1` while
    // nothing was drawn, which is what separated the two: one instrument said
    // the answer existed and another said it was never painted. Neither alone
    // would have located it.
    //
    // The fallback is a value, not a write. Painting must not mutate shared
    // state, and `kind` is already on `Preview` because the caller knew what
    // was armed.
    let st = read(ctx)
        .filter(|s| s.page_index == page_index)
        .unwrap_or_else(|| MeasureState::for_kind(page_index, kind));
    let color = snap::snap_indicator_tint(ctx)
        .unwrap_or_else(|| egui_shell::theme::Theme::canvas_selection_ink(ctx));

    // ★ The picked POINTS, marked, and drawn on EVERY frame the set is
    // non-empty — not only while the pointer is over the canvas.
    //
    // A `hover` of `None` means the pointer has left the widget, which for the
    // other tools means there is nothing to preview against. Here it means the
    // operator has moved to the Tool panel or the ribbon, which is exactly when
    // they most need to still be able to see what is in the set.
    //
    // ★★ The marker's GLYPH is the snap kind's, through the same
    // `snap::snap_marker_shapes` the hover indicator uses — so a point picked
    // on an endpoint is marked the way an endpoint is marked, and the operator
    // reads one vocabulary rather than two. A FREE point gets the endpoint
    // glyph, because that is what it is: a terminus the operator asserted.
    // Nothing distinguishes it ON THE CANVAS, deliberately — rule 4 puts that
    // disclosure off-canvas, in the Tool panel's list, where it can be read
    // rather than decoded.
    let painter = ui.painter();
    let stroke = egui::Stroke::new(1.5, color);
    for point in st.circular.points() {
        // `None` when the page transform refuses the point — the same bail
        // every other projection here takes, and the right one: one marker
        // fewer beats a panic in the frame that is trying to draw.
        let Some(at) = page_to_screen(point.at, page, map) else {
            continue;
        };
        let kind = match point.origin {
            pick::PickOrigin::Snapped(k) => k,
            pick::PickOrigin::Free => pdfcer_core::vector::snap::SnapKind::Endpoint,
        };
        painter.extend(snap::snap_marker_shapes(at, kind, color, SNAP_MARKER_PT));
        // ★★ …and a ring around it, which is the ONE thing that distinguishes
        // a committed pick from the hover marker under the pointer.
        //
        // Without it the two are the same glyph at the same size, and while the
        // pointer is over a picked point the operator cannot tell *"this is in
        // the fit"* from *"this is where a click would land"*. Those are
        // different claims — rule 4 calls the second one the cursor — and a
        // surface that renders both identically is one an operator has to
        // decode rather than read.
        //
        // A ring rather than a second colour: colour is how this canvas says
        // *kind*, and spending it on *state* would mean an endpoint pick and a
        // midpoint pick stopped being distinguishable to make room.
        painter.circle_stroke(at, SNAP_MARKER_PT * PICKED_RING_SCALE, stroke);
    }

    // ★★ The hovered entity, drawn UNDER the snap marker.
    //
    // Order is the whole of it: the highlight is a wide translucent stroke and
    // the marker is a small opaque glyph, so painting the highlight second
    // would put a coloured bar over the very node it is meant to accompany. The
    // node is the precise statement and must stay legible; the line is the
    // context.
    //
    // Drawn before the in-progress check for the same reason the marker is —
    // it does its work while the operator is deciding *where to click first*.
    if let Some(entity) = hover.and_then(|h| h.entity) {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                // ★ The list is named as well as the index. A page has two,
                // and a trace that cannot tell `objects[7]` from `leaves[7]`
                // is a trace that cannot be read back.
                "measure-hover-entity {}={} segment={}",
                match entity.target {
                    pdfcer_core::vector::hit::HitTarget::Object(_) => "object",
                    pdfcer_core::vector::hit::HitTarget::Leaf(_) => "leaf",
                },
                match entity.target {
                    pdfcer_core::vector::hit::HitTarget::Object(i)
                    | pdfcer_core::vector::hit::HitTarget::Leaf(i) => i,
                },
                u8::from(entity.segment.is_some())
            )
        });
        ui.painter().extend(hover::shapes(entity, color, |p| {
            page_to_screen(p, page, map)
        }));
    }

    // ★ The snap indicator is drawn BEFORE the in-progress check, and that is
    // the point of it.
    //
    // It has to appear while the operator is still deciding *where to click
    // first* — that is when it does its work, by saying "this click will land
    // on that endpoint, not where your pointer is". Gating it on a gesture
    // already being in progress would show it only after the first pick, i.e.
    // everywhere except the place it is needed most.
    //
    // This is also what makes the snap honest rather than sneaky
    // (`pdfce_FeatureRequests/README.md` rule 4): the point is moved, and the
    // operator is told, before anything is committed. A snap that silently
    // relocated a click would be an inference applied without disclosure.
    if let Some(c) = hover.and_then(|h| h.candidate)
        && let Some(screen) = page_to_screen(c.point, page, map)
    {
        // ★★ The marker's screen position and the pointer's, on one line.
        //
        // The evidence for an invariant that is **true by the definition of
        // snapping**: a snap marker is never further from the pointer than the
        // snap tolerance. That is what "snap" means — the click is being moved
        // to something *near* where the operator is aiming.
        //
        // It exists because of the defect `resolve_hover`'s own docs record:
        // the preview was resolved against a raw SCREEN position while the
        // click used a converted CANVAS one, so the marker sat away from the
        // pointer by the scroll origin over the zoom. It survived four days and
        // no unit test could have seen it — both functions were individually
        // correct and the caller mixed two spaces that are the same type.
        //
        // `dx`/`dy` rather than a distance: a pure-x or pure-y offset says
        // "one axis of the conversion", and a reader chasing a regression
        // wants to know which.
        crate::diag::trace(|| {
            let p = ctx.pointer_latest_pos();
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "measure-snap-marker kind={:?} marker={:.1},{:.1} pointer={:?} dx={:.1} dy={:.1} tol={:.2}",
                c.kind,
                screen.x,
                screen.y,
                p.map(|q| (q.x, q.y)),
                p.map_or(f32::NAN, |q| screen.x - q.x),
                p.map_or(f32::NAN, |q| screen.y - q.y),
                map.snap_tolerance(),
            )
        });
        // Zoom-invariant: the marker is a screen-space affordance, so its size
        // is in points and does not grow with magnification.
        ui.painter().extend(snap::snap_marker_shapes(
            screen,
            c.kind,
            color,
            SNAP_MARKER_PT,
        ));
    }

    if !st.gesture_in_progress() {
        return;
    }

    let segments: Vec<(Point, Point)> = match kind {
        // ★ The reference line, drawn exactly as the linear tool draws its
        // measuring segment — because it IS one. `ScalePick::line` is a
        // `LinearPick`, so an operator calibrating sees the same constrained
        // A-to-pointer rubber band, the same snap markers and the same
        // H/V/aligned behaviour they already know from placing a dimension.
        //
        // Once both points are picked the segment stops following the pointer
        // and the dialog is up, so the line stays drawn while the operator
        // types what it represents — which is the picture that makes the
        // question answerable: *this* line is how long?
        MeasureKind::Scale => {
            if st.scale.drawn_pdf_length.is_some() {
                // Complete: hold the committed line still while the operator
                // types. `placing_preview` is fed the pointer only so it can
                // answer at all; with both points picked the segment it
                // returns is the picked line and does not follow the pointer.
                hover
                    .and_then(|h| st.scale.line.placing_preview(h.at))
                    .as_ref()
                    .map_or_else(Vec::new, pick::dimension_preview_segments)
            } else {
                let Some(at) = hover.map(|h| h.at) else {
                    return;
                };
                st.scale.line.preview_segment(at).into_iter().collect()
            }
        }
        MeasureKind::Linear => {
            // ★ The placing preview needs the pointer; the measuring one does
            // too. With the pointer off the widget there is nothing honest to
            // draw, so nothing is.
            let Some(at) = hover.map(|h| h.at) else {
                return;
            };
            if let Some(authored) = st.linear.placing_preview(at) {
                // Placing: draw the dimension itself, exactly as it will land.
                pick::dimension_preview_segments(&authored)
            } else {
                // Measuring: the constrained A→pointer segment.
                st.linear.preview_segment(at).into_iter().collect()
            }
        }
        // ★ The fitted circle, from the value the commit would author.
        //
        // `author()` is `None` for a degenerate set — one arc, or three points
        // on a line — and that draws nothing, which is the correct picture: the
        // objects are outlined so the operator can see what is in the set, and
        // no circle appears because there is no circle. Drawing a guess would
        // promise a dimension `circular::commit` is about to refuse.
        MeasureKind::Circular => st
            .circular
            .author()
            .map(|kind| pick::dimension_preview_segments(&kind))
            .unwrap_or_default(),
        // ★ The perimeter, drawn through the SAME segment function a committed
        // one is drawn from - the standing rule in this module, and the whole
        // of what makes a preview a preview rather than an illustration.
        //
        // `preview` appends the pointer as a provisional last vertex, so the
        // rubber band runs from the last committed pick to the cursor and says
        // *"this click would add this segment"*. It is deliberately never drawn
        // closed: the operator has not closed it, and showing the closing
        // segment early would promise a shape one segment longer than the one
        // the next click commits.
        MeasureKind::Perimeter | MeasureKind::PathLength => {
            let Some(at) = hover.map(|h| h.at) else {
                return;
            };
            st.perimeter
                .preview(at)
                .map(|kind| pick::dimension_preview_segments(&kind))
                .unwrap_or_default()
        }
        // A two-line pick has nothing to preview against the pointer: what it
        // has picked is a *line already on the page*, which the page is already
        // drawing. Highlighting the picked line is the affordance it wants, and
        // that needs the hover query this call site does not yet run — see the
        // module header's note on what remains.
        MeasureKind::TwoLine => Vec::new(),
    };
    if segments.is_empty() {
        return;
    }

    for (a, b) in segments {
        let (Some(sa), Some(sb)) = (page_to_screen(a, page, map), page_to_screen(b, page, map))
        else {
            continue;
        };
        painter.line_segment([sa, sb], stroke);
    }
}

/// How large the snap marker is drawn, in **points**.
///
/// Screen-space rather than page-space on purpose: the marker is an
/// affordance, not content, so it must stay the same apparent size whether the
/// operator is zoomed to a whole A1 sheet or to one dimension line. Carried
/// from the old shell's own indicator sizing.
///
/// ★ `pub(in crate::canvas)` because the perimeter's vertex drag snaps too and
/// draws the SAME marker at the SAME size. A second constant would be two
/// sizes for one affordance, free to
/// diverge — and an operator who has learned that a small square means
/// *endpoint* while placing a perimeter must read the identical square while
/// correcting one.
pub(in crate::canvas) const SNAP_MARKER_PT: f32 = 6.0;

/// How much wider than the snap glyph the **committed-pick ring** is drawn.
///
/// Big enough to read as a ring around the glyph rather than as a fatter glyph,
/// small enough that four picks round a small hole do not merge into a blob.
/// See the ring's own comment in [`preview`] for why the distinction exists at
/// all.
const PICKED_RING_SCALE: f32 = 1.7;

/// **PDF user space → screen**, both hops, in one place.
///
/// # ★ This function is the fix for a defect, and the defect had shipped
///
/// It replaced a `page_to_canvas` that stopped after the first hop and handed
/// the result straight to `ui.painter()`. Three frames are in play — screen,
/// canvas and PDF user (`crate::canvas::mapping`'s header carries the table) —
/// and `viewer::pdf_space_to_canvas` lands in the **middle** one: y-down, but
/// with its origin at the page's top-left corner and **no zoom applied**,
/// because `page_device_geometry` is asked for scale `1.0`. The painter speaks
/// screen. So every mark this module drew — the snap indicator and the linear
/// preview alike — was offset by wherever the page happened to sit in the
/// window and drawn at 100 % regardless of the actual magnification.
///
/// It is a whole class — a mark authored in the right place and drawn in the
/// wrong one — and it is invisible to every test in this file, because the
/// tests that exist are about *which point is picked* and the picking is
/// right. Only the drawing is wrong, and only in a window.
///
/// The second hop is [`PageMapping::to_screen`] — the canvas's own outward
/// boundary crossing, the one `canvas::overlay` has always used for selection
/// outlines. There is no third conversion here and no arithmetic; if there
/// were, it would be the second place in `canvas/` that divides by zoom, which
/// `crate::canvas::mapping`'s header forbids.
pub(in crate::canvas) fn page_to_screen(
    p: Point,
    page: &pdfcer_core::page_tree::Page,
    map: &PageMapping,
) -> Option<Pos2> {
    let canvas = viewer::pdf_space_to_canvas(Pos2::new(p.x as f32, p.y as f32), page)?;
    Some(map.to_screen(canvas))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **The preview is drawn in SCREEN space, through the frame's map.**
    ///
    /// The regression test for the defect `page_to_screen`'s own docs describe:
    /// `viewer::pdf_space_to_canvas` lands in **canvas** space — page top-left
    /// origin, no zoom — and the painter speaks screen, so a preview that
    /// stopped after the first hop drew every mark offset by wherever the page
    /// sat in the window and at 100 % whatever the magnification.
    ///
    /// Asserted as a **magnitude**, not a relation: at zoom 2 with the page's
    /// corner at (37, 11), the page-space point that is 50 canvas units in from
    /// the page corner must land 100 screen points in from (37, 11). A test
    /// that merely checked "the two differ" would be satisfied by any wrong
    /// answer in the right direction.
    #[test]
    fn the_preview_projects_page_space_all_the_way_to_the_screen() {
        let origin = egui::Pos2::new(37.0, 11.0);
        let zoom = 2.0_f32;
        let extent = (200.0_f32, 300.0_f32);
        let map = PageMapping::new(
            egui::Rect::from_min_size(origin, egui::vec2(extent.0 * zoom, extent.1 * zoom)),
            extent,
            zoom,
        );
        // Canvas (50, 60) is 50 across and 60 down from the page's top-left.
        let canvas = egui::Pos2::new(50.0, 60.0);
        let screen = map.to_screen(canvas);
        assert!(
            (screen.x - (origin.x + 100.0)).abs() < 1e-3
                && (screen.y - (origin.y + 120.0)).abs() < 1e-3,
            "the second hop must apply the page origin AND the zoom, got {screen:?}"
        );
        // …and the canvas coordinate on its own is neither, which is what made
        // the defect invisible at zoom 1 with the page at the window's origin.
        assert!(
            (canvas.x - screen.x).abs() > 1.0,
            "a canvas coordinate handed straight to the painter is the defect"
        );
    }
}
