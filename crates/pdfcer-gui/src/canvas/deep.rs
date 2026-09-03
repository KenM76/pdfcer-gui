//! # `canvas::deep` — everything the canvas does about the `f64` position tier
//!
//! ## Why this is its own module
//!
//! `OPERATOR_REQUESTS.md` O24 introduced a second mechanism for *"where is the
//! view"*. Below about two million percent the `egui` scroll offset holds it,
//! as it always has; above, an `f32` measured in screen pixels over a content
//! space of `page × zoom` can no longer address every pixel — at a trillion
//! percent it moves in 2,048-pixel jumps — so
//! [`crate::viewer::deep::DeepAnchor`] takes over, holding a page point in
//! `f64` and the screen pixel it sits under.
//!
//! Two mechanisms means **two hand-overs**, and the whole of this module's
//! subject is the seam: seeding the anchor on the way in, restating it on
//! every zoom while it holds, moving it on a pan or a wheel, and converting it
//! back into a scroll offset on the way out.
//!
//! ★★★ It lives in its own file because the seam is where the defects are and
//! it had become impossible to see them together. O24f found three faults in
//! the upward hand-over. O26e then found that the **downward** one did not
//! exist at all — the anchor was discarded and the `f32` machinery resumed
//! from the zero this tier forces, which put the page's own origin under the
//! pointer and lost twelve million pixels of drawing. Both sets of code were
//! inside a 1,700-line `canvas::show`, hundreds of lines apart, and R2's rule
//! exists for exactly that: *"nothing could be reasoned about locally"*.
//!
//! ## The invariant this module owns
//!
//! > **While the tier holds, the scroll offset is meaningless and must not be
//! > read as a position; on the frames either side of it, the two mechanisms
//! > must agree about where the view is to within the `f32` offset's own
//! > resolution.**
//!
//! Everything here is in service of the second clause. The first is why the
//! caller forces the scroll offset to zero while `deep` is true, and why
//! `canvas::show`'s current-page tracker skips its scroll-derived answer at
//! this tier: both would otherwise be reading a value that says nothing.
//!
//! ## What is deliberately NOT here
//!
//! The **placement** — where the strip is drawn from the anchor — and the
//! **visible rect** the raster region is derived from. Those are branches
//! inside `canvas::show`'s scroll-area closure, where the rects they produce
//! are consumed a few lines later, and lifting them out would trade one
//! locality for a worse one. They read the anchor; they do not maintain it.

use egui::{Vec2, vec2};

use crate::app::state::OpenDoc;
use crate::canvas::geometry;
use crate::canvas::input::pan_delta;
use crate::canvas::tool::CanvasTool;
use crate::canvas::zoom;
use crate::viewer;

/// **Maintain the `f64` anchor for this frame, and hand its position back to
/// the scroll offset on the frame that leaves the tier.**
///
/// Called once per frame from `canvas::show`, before the scroll area is built,
/// with `deep` already decided by
/// [`crate::viewer::deep_position_needed`].
///
/// Returns the **page-local** scroll offset that continues where the anchor
/// left off, and `Some` on exactly one frame: the one that has just dropped
/// below the threshold. `None` on every other frame, including every frame
/// inside the tier — there the caller forces the offset to zero instead, for
/// the reason its own comment gives.
///
/// # Arguments
///
/// * `layout` — the strip, for the current page's origin within it. The
///   anchor's page point is in the **current page's** own coordinates, so
///   every conversion in here needs to know where that page sits.
/// * `current_display` — the current page's drawn size; `display_size` — the
///   whole strip's. Both are needed and they are not the same question: the
///   first is what a single-page solve is handed, the second is what the
///   scroll content is made of.
/// * `vp` — the viewport measured **before** the scroll area was built, the
///   same measurement every margin term in [`crate::canvas::geometry`] is
///   derived against.
#[allow(clippy::too_many_arguments, reason = "one frame's geometry, named")] // ui-text-exempt: clippy lint justification, never displayed
pub(super) fn track(
    ui: &egui::Ui,
    doc: &mut OpenDoc,
    layout: &viewer::strip::Strip,
    current: usize,
    current_display: (f32, f32),
    display_size: Vec2,
    vp: Vec2,
    active_tool: CanvasTool,
    deep: bool,
) -> Option<Vec2> {
    // The strip conversion the seed needs: a page-local offset, expressed as
    // the offset the scroll area would be at. Spelled here rather than passed
    // in as a closure so this module can be read on its own.
    let current_origin = layout
        .rect_of(current)
        .map_or((0.0, 0.0), |r| (r.min.x, r.min.y));
    let to_strip = |local: (f32, f32)| {
        let (x, y) = geometry::strip_offset(
            local,
            current_origin,
            (display_size.x, display_size.y),
            current_display,
            (vp.x, vp.y),
        );
        vec2(x, y)
    };

    if deep {
        // ★★★ THE ZOOM ANCHOR IS CONSUMED AT THIS TIER TOO —
        // `OPERATOR_REQUESTS.md` **O24f**.
        //
        // It used to be consumed only on the shallow branch, and both halves
        // of that were wrong at once:
        //
        // 1. **On the way in**, the seed below read `doc.last_scroll_offset` —
        //    the PREVIOUS frame's settled offset, recorded before this frame's
        //    zoom landed. Dividing it by the NEW zoom asks where a point is
        //    using one frame's distance and the next frame's scale, so the
        //    seeded page point was wrong by the whole zoom ratio. Crossing the
        //    threshold threw the view to an unrelated part of the sheet.
        // 2. **Once inside**, the anchor was dropped entirely, so a zoom held
        //    nothing under the cursor: the anchored page point stayed nailed
        //    to the viewport's top-left corner and everything the operator was
        //    looking at expanded off the screen.
        //
        // Both surface as the same sentence — *"I do lose the view at 2000000%
        // magnification"* — and 2,000,000 % is not a number he picked. The
        // threshold is `SUB_PIXEL_CONTENT_EXTENT / page_height` = 16,777,216 /
        // 792 ≈ **2,118,000 %** on a Letter sheet. A defect that begins at the
        // tier boundary is a defect in the tier hand-over.
        //
        // ★ Consumed unconditionally, not only when seeding. An anchor left
        // pending here would fire on whatever frame the operator next dropped
        // below the threshold, moving the view for a zoom that happened
        // minutes earlier.
        let landed = zoom::consume_anchor(ui.ctx(), doc, current_display)
            .map(|local| to_strip((local.x, local.y)));
        if doc.deep_anchor.is_none() {
            // Prefer the offset the zoom just solved for; fall back to the
            // last settled one only when no zoom is landing, which is the case
            // where they agree anyway.
            let from = landed.unwrap_or(doc.last_scroll_offset);
            let seen = geometry::scroll_to_strip(from.x, display_size.x, vp.x);
            let seen_y = geometry::scroll_to_strip(from.y, display_size.y, vp.y);
            let origin = layout
                .rect_of(current)
                .map_or((0.0, 0.0), |r| (r.min.x, r.min.y));
            let zoom = f64::from(doc.view.zoom);
            doc.deep_anchor = Some(viewer::deep::DeepAnchor {
                page: (
                    f64::from(seen - origin.0) / zoom,
                    f64::from(seen_y - origin.1) / zoom,
                ),
                screen: (0.0, 0.0),
            });
        } else if let Some(prev) = doc.deep_zoom
            && (prev - f64::from(doc.view.zoom)).abs() > f64::EPSILON
        {
            // ★★ ZOOM ABOUT THE CURSOR, by re-statement rather than by solving
            // for an offset. `DeepAnchor::zoomed_about` reads which page point
            // is under a window point at the OLD zoom and declares that point
            // anchored there — no large intermediate is formed, so nothing is
            // lost however deep the zoom goes. That is the operation the whole
            // `viewer::deep` module exists for, and until O24f nothing called
            // it.
            //
            // ★ The viewport centre when the pointer is elsewhere, matching
            // what `+`, `−` and Ctrl+0 anchor on at every other zoom. A
            // keyboard zoom must not lurch toward wherever the mouse happens
            // to be resting.
            // ★ `ui.max_rect()` is the canvas's own region — inside the ruler
            // gutters, and the same rect `input::pan_delta` tests against, so
            // a pointer over a ruler is treated as "not over the page" by both
            // and cannot anchor a zoom to a place the operator was not
            // pointing at. At this tier the scroll area's content IS the
            // viewport, so its origin and this rect's origin are the same
            // point — which is what the anchor's `screen` is measured from.
            let region = ui.max_rect();
            let at = ui
                .input(|i| i.pointer.latest_pos())
                .filter(|p| region.contains(*p))
                .map_or((vp.x / 2.0, vp.y / 2.0), |p| {
                    (p.x - region.min.x, p.y - region.min.y)
                });
            doc.deep_anchor = doc.deep_anchor.map(|a| a.zoomed_about(at, prev));
        }
        doc.deep_zoom = Some(f64::from(doc.view.zoom));
        // ★ Pan and wheel move the ANCHOR now. The scroll area has nothing to
        // scroll, so routing them to it would be a gesture that silently does
        // nothing — which is how a deep zoom would come to feel frozen.
        if let Some(anchor) = doc.deep_anchor {
            let zoom = f64::from(doc.view.zoom);
            let mut moved = anchor;
            if let Some(pan) = pan_delta(ui, active_tool) {
                moved = moved.panned((pan.x, pan.y), zoom);
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            }
            let wheel = ui.input(|i| i.smooth_scroll_delta);
            if wheel != egui::Vec2::ZERO {
                moved = moved.panned((wheel.x, wheel.y), zoom);
            }
            doc.deep_anchor = Some(moved);
        }
    } else if let Some(anchor) = doc.deep_anchor {
        // ★★★ LEAVING THE DEEP TIER — the hand-over BACK, which for two days
        // did not exist. `OPERATOR_REQUESTS.md` O26e.
        //
        // O24f built the hand-over **in**: crossing the threshold upward
        // seeds the `f64` anchor from the offset the zoom just solved for.
        // Nothing did the inverse. Coming back down, the anchor was simply
        // discarded and the `f32` machinery took over from whatever the
        // scroll area happened to be carrying — which, at this tier, is the
        // zero the branch above forces every frame.
        //
        // ★★ A hand-over is two functions and only one of them was written.
        // The suite could not see it because `zoom_keeps_place` **climbs**:
        // it climbs to the ceiling, one notch at a time, with a tolerance
        // fine enough to catch a hundredth of a point, and then the run ends
        // without ever rolling the wheel the other way. A check that travels
        // in one direction tests one direction.
        //
        // ★ Solved here, in `f64`, rather than left to the `f32` anchor
        // machinery. `offset_from_drawn` already stops the answer being
        // nonsense — it made this same descent land 0.005 pt out instead of
        // 1,152 pt out — but 0.005 pt at a million percent is fifty screen
        // pixels, because every term it subtracts has a magnitude near 10⁷
        // and an `f32`'s step there is a whole pixel. The anchor holds the
        // same quantity in `f64` and can simply be *asked*.
        let handed = handover_offset(ui, doc, anchor, current_display, (vp.x, vp.y));
        // Forget the anchor so re-entering seeds afresh from wherever the
        // scroll area has since settled, and forget the zoom it was valid for
        // so the first frame back inside seeds rather than re-anchors.
        doc.deep_anchor = None;
        doc.deep_zoom = None;
        if handed.is_some() {
            // ★ Spend the pending zoom anchor rather than clearing the field,
            // so the `waited` bookkeeping in `zoom::consume_anchor` stays
            // consistent. Its answer is discarded: it is the `f32` solve this
            // branch exists to replace, and leaving the anchor armed would
            // fire it on some later frame — a view that moves for a zoom that
            // happened seconds ago.
            let _ = zoom::consume_anchor(ui.ctx(), doc, current_display);
        }
        return handed;
    }
    None
}

/// **The page-local scroll offset that continues where the `f64` anchor left
/// off**, for the one frame that drops out of the deep-position tier.
///
/// `OPERATOR_REQUESTS.md` O26e. See the branch in [`show_in`] that calls it
/// for why the hand-over back had to exist, and
/// [`viewer::deep::DeepAnchor::page_local_offset`] for why the arithmetic is
/// there and not here.
///
/// # What this function contributes beyond that arithmetic
///
/// **The last zoom step.** `DeepAnchor` describes the position at the zoom it
/// was last updated for — `doc.deep_zoom` — and this frame is running at a
/// *new*, lower zoom whose step nothing has applied to the anchor: the deep
/// branch's `zoomed_about` call is inside the `if deep` arm, and this frame is
/// not in it. Handing the stale anchor straight over would keep the position
/// but discard the final notch of zoom-about-the-cursor, so the last step out
/// of deep zoom would be the one step that did not hold the pointer.
///
/// ★ The anchor point is the pointer when it is over the canvas and the
/// viewport's centre when it is not — the rule stated once in
/// [`zoom::anchor_point`] and applied here in the same words the deep branch
/// applies it in, against the same `ui.max_rect()` so that a pointer resting
/// on a ruler gutter counts as "not over the page" for both.
///
/// `None` when there is no zoom to describe a placement at, which the caller
/// treats as *"fall through to the ordinary anchor"* — the behaviour before
/// this function existed, and safe because it is only reachable for a zoom
/// that is not a positive finite number.
fn handover_offset(
    ui: &egui::Ui,
    doc: &OpenDoc,
    anchor: viewer::deep::DeepAnchor,
    display: (f32, f32),
    viewport: (f32, f32),
) -> Option<Vec2> {
    let zoom = f64::from(doc.view.zoom);
    let region = ui.max_rect();
    let at = ui
        .input(|i| i.pointer.latest_pos())
        .filter(|p| region.contains(*p))
        .map_or((viewport.0 / 2.0, viewport.1 / 2.0), |p| {
            (p.x - region.min.x, p.y - region.min.y)
        });
    let stated = match doc.deep_zoom {
        Some(prev) if prev.is_finite() && prev > 0.0 && (prev - zoom).abs() > f64::EPSILON => {
            anchor.zoomed_about(at, prev)
        }
        _ => anchor,
    };
    stated
        .page_local_offset(display, viewport, zoom)
        .map(|(x, y)| vec2(x, y))
}

/// **Where the strip is drawn, when the anchor owns the position.**
///
/// The anchor says *this page point sits under that screen pixel*, so the
/// current page's top-left lands at `anchor.screen − anchor.page × zoom` from
/// the scroll content's origin, and the strip's origin is that less where the
/// page sits inside the strip.
///
/// ★★★ Every large magnitude is subtracted **inside `f64`** before anything
/// narrows. At a trillion percent `anchor.page × zoom` is around 10¹², where
/// an `f32` cannot represent the difference of two neighbouring screen pixels
/// at all — this is the same technique the engine's own deep-zoom work
/// describes as *"one subtraction moved into `f64`"*, and it is the reason the
/// tier exists.
///
/// ★ Falls back to [`viewer::deep::DeepAnchor::origin`] rather than declining,
/// because a frame in this tier must draw something and the origin is the one
/// placement that needs no history. The caller seeds a real anchor on the same
/// frame it first becomes `deep`, so the fallback is reachable only on a frame
/// where the seed itself failed.
pub(super) fn strip_placement(
    doc: &OpenDoc,
    layout: &viewer::strip::Strip,
    current: usize,
    content_min: egui::Pos2,
    display_size: Vec2,
) -> egui::Rect {
    let anchor = doc
        .deep_anchor
        .unwrap_or_else(viewer::deep::DeepAnchor::origin);
    let zoom = f64::from(doc.view.zoom);
    let page_origin = layout
        .rect_of(current)
        .map_or((0.0, 0.0), |r| (r.min.x, r.min.y));
    let x = f64::from(content_min.x) + f64::from(anchor.screen.0) - anchor.page.0 * zoom;
    let y = f64::from(content_min.y) + f64::from(anchor.screen.1) - anchor.page.1 * zoom;
    egui::Rect::from_min_size(
        egui::Pos2::new(x as f32 - page_origin.0, y as f32 - page_origin.1),
        display_size,
    )
}

/// **What of the strip is on screen, when the anchor owns the position** — the
/// rect that decides which pages are drawn at all.
///
/// ★ The scroll offset cannot answer this at this tier, because it has been
/// forced to zero and describes a place nobody is looking at. The strip's own
/// placement on screen is the truth instead: whatever of it overlaps the
/// viewport is what can be seen, so the viewport's origin expressed in strip
/// space is simply `content_min − strip_min`.
///
/// A two-line function with a paragraph of reasoning, deliberately. Below the
/// threshold the same quantity comes from
/// [`crate::canvas::geometry::scroll_to_strip`] and the two look
/// interchangeable; they are not, and one line of the wrong one is the whole
/// of `canvas-unavailable reason=nothing-visible`.
pub(super) fn visible_in_strip(
    content_min: egui::Pos2,
    strip_min: egui::Pos2,
    viewport: Vec2,
) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::Pos2::new(content_min.x - strip_min.x, content_min.y - strip_min.y),
        viewport,
    )
}
