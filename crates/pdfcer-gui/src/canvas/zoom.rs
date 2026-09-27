//! # `canvas::zoom` — the anchor rule, decided once, and the five paths that route through it
//!
//! ## The rule
//!
//! > **A zoom holds one page point still, and that point is where the operator
//! > is looking: the pointer when it is over the canvas, the centre of the
//! > viewport when it is not. A zoom that *frames* something — a selection, a
//! > marquee'd region — holds that thing's centre still, at the centre of the
//! > viewport.**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/zoom.md`.

use egui::{Context, Pos2, Rect, Vec2};

use crate::app::actions::Action;
use crate::app::state::{OpenDoc, ZoomAnchor};
use crate::canvas::geometry;
use crate::canvas::mapping::PageMapping;
use crate::viewer::{self, FitMode};

/// `egui::Memory` key for the last drawn frame's canvas geometry.
const FRAME_MEMORY_KEY: &str = "pdfcer-canvas-frame"; // ui-text-exempt: internal memory id, never displayed

/// `egui::Memory` key for the one-shot marquee-zoom arming flag.
const REGION_MEMORY_KEY: &str = "pdfcer-canvas-zoom-region"; // ui-text-exempt: internal memory id, never displayed

/// `egui::Memory` key for "a pending anchor has already waited one frame".
const WAITED_MEMORY_KEY: &str = "pdfcer-canvas-anchor-waited"; // ui-text-exempt: internal memory id, never displayed

/// The smallest region, in canvas units, that a zoom-to-rect will fit.
pub const MIN_REGION_EXTENT: f32 = 8.0;

// ---------------------------------------------------------------------------
// The frame record
// ---------------------------------------------------------------------------

/// The geometry the last drawn canvas frame settled on — everything an entry
/// point outside the canvas needs in order to describe a zoom.
#[derive(Debug, Clone, Copy)]
pub struct CanvasFrame {
    /// The frame's screen ⟷ canvas map — used here only to convert a pointer
    /// or a viewport centre into canvas space, which is the one conversion
    /// this module performs and the reason it does not hold a zoom of its own.
    pub map: PageMapping,
    /// The page's extent in canvas units, `/Rotate` applied.
    pub extent: (f32, f32),
    /// The page's drawn size in logical points — `extent × zoom`.
    pub display: (f32, f32),
    /// The scroll viewport's **size**, exactly as the wheel path measures it
    /// (`ui.available_size()` inside the scroll area). This is the number the
    /// centring-margin term in
    /// [`crate::canvas::geometry::zoom_anchor_offset`] is derived against, so
    /// it must be the same measurement or the margin will disagree with the
    /// one the frame actually drew.
    pub viewport: (f32, f32),
    /// The scroll viewport's **rect in screen coordinates** — a different
    /// question from `viewport`, and both are needed: this one answers *"is
    /// the pointer over the canvas, and where is the middle of what is on
    /// screen?"*, which is a position, while `viewport` answers *"how much
    /// room is there?"*, which is a size.
    pub viewport_rect: Rect,
    /// The scroll offset the frame settled on.
    pub offset: (f32, f32),
    /// **The page every other field here describes** — `canvas::show`'s
    /// acting page for the frame this record was written on.
    ///
    /// Under a continuous mode `map`, `extent`, `display` and `offset` are
    /// all about one page of a strip, and an anchor built from them is only
    /// meaningful against that same page. Carrying the index is what lets the
    /// canvas convert the solve's answer back through the **right** page's
    /// origin a frame later, when the current page may have moved on. See
    /// [`crate::viewer::ZoomAnchor::page`].
    pub page: usize,
    /// **The OUTER viewport** — `ui.available_size()` measured *before* the
    /// scroll area — as against [`Self::viewport`], which is the size inside
    /// it. `OPERATOR_REQUESTS.md` O78.
    ///
    /// # Why both, and why the difference is not a rounding error
    ///
    /// A scroll bar takes real width. `canvas::fit` measures where the centred
    /// page point was against the frame this record describes, and places it
    /// against the frame being laid out — and the frame being laid out only
    /// knows the **outer** size at that moment, because its scroll area has not
    /// been built yet. Measuring "before" against the inner size and placing
    /// "after" against the outer would land the centre about half a scroll
    /// bar's width off, systematically, and only when a bar is showing — which
    /// is exactly the shape of defect nobody reports and everybody notices.
    ///
    /// ⇒ Two fields, one question each: `viewport` is *how much room the
    /// content had*, this is *how much room the canvas had*. The centre rule
    /// uses this one at both ends.
    ///
    ///
    /// The canvas's scroll bars became solid (`canvas::present::scroll_style`),
    /// and a solid bar takes real width — the situation the paragraph above
    /// was written for and, with egui's floating default allocating **zero**,
    /// had never actually met. On the first build where it did, the frame was
    /// recorded against the inner size, the centre was measured against this
    /// outer one, and the page **crept 7 px per frame** — half the 14 pt bar
    /// allocation — until it left the screen. Measured on a driven run.
    ///
    /// The repair was not to choose between the two but to make the question
    /// go away: `canvas::present` now measures ONE viewport, `inner_avail =
    /// available − allocated_width()`, before the scroll area is built, and
    /// derives the fit viewport, `vp`, and therefore this field from it — so
    /// `outer == viewport` on every frame, and every margin term in
    /// `geometry` is derived against the room the content actually got.
    /// The field is kept, with its history, so the next reader who finds two
    /// names for one number knows it was once two numbers and why it is not.
    pub outer: (f32, f32),
}

/// Record this frame's canvas geometry. Called once, at the end of
/// [`crate::canvas::show`].
pub fn remember_frame(ctx: &Context, frame: CanvasFrame) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(FRAME_MEMORY_KEY), frame));
}

/// The last drawn frame's canvas geometry, or `None` before the canvas has
/// ever drawn a page.
#[must_use]
pub fn last_frame(ctx: &Context) -> Option<CanvasFrame> {
    ctx.data(|d| d.get_temp::<CanvasFrame>(egui::Id::new(FRAME_MEMORY_KEY)))
}

// ---------------------------------------------------------------------------
// The rule
// ---------------------------------------------------------------------------

/// **The anchor rule.** The canvas-space point a zoom step must hold still.
#[must_use]
pub fn anchor_point(pointer: Option<Pos2>, frame: &CanvasFrame) -> Pos2 {
    let screen = pointer
        .filter(|p| frame.viewport_rect.contains(*p))
        .unwrap_or_else(|| frame.viewport_rect.center());
    frame.map.to_page(screen)
}

pub use pdfcer_gui_base::viewgeometry::frac_of;

/// An anchor that **holds** the point at `frac` exactly where it is now — the
/// wheel's and the discrete commands' shape.
#[must_use]
pub fn hold(frac: (f32, f32), frame: &CanvasFrame) -> ZoomAnchor {
    ZoomAnchor {
        frac,
        offset_before: frame.offset,
        display_before: frame.display,
        viewport: frame.viewport,
        page: frame.page,
    }
}

/// An anchor that **places** the point at `frac` at the centre of the viewport
/// — the framing shape, used by zoom-to-selection and zoom-to-region.
#[must_use]
pub fn place_centred(frac: (f32, f32), frame: &CanvasFrame) -> ZoomAnchor {
    let centre = (frame.viewport.0 / 2.0, frame.viewport.1 / 2.0);
    ZoomAnchor {
        frac,
        offset_before: geometry::offset_holding_anchor_at(
            frac,
            centre,
            frame.display,
            frame.viewport,
        ),
        display_before: frame.display,
        viewport: frame.viewport,
        page: frame.page,
    }
}

// ---------------------------------------------------------------------------
// The two-frame handshake
// ---------------------------------------------------------------------------

/// What [`crate::canvas::show`] should do with a pending anchor this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorStep {
    /// The zoom has not landed yet. Leave the anchor alone; it is for a later
    /// frame.
    Hold,
    /// The page's drawn size has changed, so the zoom has landed: solve the
    /// offset and consume the anchor.
    Solve,
    /// The zoom never came — a command that was already at the ceiling or the
    /// floor. Consume the anchor without moving the view.
    Drop,
}

/// The consume gate. See the module docs for the whole argument.
#[must_use]
pub fn anchor_step(anchor: &ZoomAnchor, display_now: (f32, f32), waited: bool) -> AnchorStep {
    // Exact inequality rather than a tolerance: `display_before` was written
    // from the same expression (`extent × zoom`) that produces `display_now`,
    // so an unchanged zoom on an unchanged page reproduces the bits exactly.
    // A tolerance here would swallow the smallest real ladder step on a small
    // page and turn it into a `Drop`.
    #[allow(
        clippy::float_cmp,
        reason = "both sides are `extent × zoom` from the same f32 inputs; equality means the zoom did not move" // ui-text-exempt: clippy lint justification, never displayed
    )]
    let moved =
        display_now.0 != anchor.display_before.0 || display_now.1 != anchor.display_before.1;
    if moved {
        AnchorStep::Solve
    } else if waited {
        AnchorStep::Drop
    } else {
        AnchorStep::Hold
    }
}

/// Resolve this frame's pending anchor, returning the scroll offset the canvas
/// must force — or `None` to leave the scroll area alone.
pub fn consume_anchor(ctx: &Context, doc: &mut OpenDoc, display_now: (f32, f32)) -> Option<Vec2> {
    let anchor = doc.frame.zoom_anchor?;
    let waited_id = egui::Id::new(WAITED_MEMORY_KEY);
    let waited = ctx.data(|d| d.get_temp::<bool>(waited_id).unwrap_or(false));
    match anchor_step(&anchor, display_now, waited) {
        AnchorStep::Hold => {
            ctx.data_mut(|d| d.insert_temp(waited_id, true));
            None
        }
        AnchorStep::Drop => {
            doc.frame.zoom_anchor = None;
            ctx.data_mut(|d| d.insert_temp(waited_id, false));
            None
        }
        AnchorStep::Solve => {
            doc.frame.zoom_anchor = None;
            ctx.data_mut(|d| d.insert_temp(waited_id, false));
            let (x, y) = geometry::zoom_anchor_offset(
                anchor.offset_before,
                anchor.display_before,
                display_now,
                anchor.viewport,
                anchor.frac,
            );
            Some(Vec2::new(x, y))
        }
    }
}

// ---------------------------------------------------------------------------
// Entry points — the discrete commands
// ---------------------------------------------------------------------------

/// Which discrete zoom a command asked for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ZoomStep {
    /// The next ladder rung up — the status bar's `+`, and `Ctrl` `+`.
    ///
    In,
    /// The next ladder rung down — the status bar's `−`, and `Ctrl` `-`. Its
    /// twin above carries the note about the name.
    Out,
    /// One PDF point per screen point — `view.zoom_actual`, Ctrl+0.
    ActualSize,
    /// An exact factor, for a zoom picker.
    To(f32),
}

impl ZoomStep {
    /// The action this step raises.
    #[must_use]
    pub fn action(self) -> Action {
        match self {
            Self::In => Action::ZoomIn,
            Self::Out => Action::ZoomOut,
            Self::ActualSize => Action::ZoomTo(1.0),
            Self::To(zoom) => Action::ZoomTo(zoom),
        }
    }
}

/// Arm the anchor for a discrete zoom that is about to be raised.
pub fn arm_anchor(ctx: &Context, doc: &mut OpenDoc) {
    let Some(frame) = last_frame(ctx) else {
        return;
    };
    let point = anchor_point(ctx.pointer_latest_pos(), &frame);
    doc.frame.zoom_anchor = Some(hold(frac_of(point, frame.extent), &frame));
}

/// Whether an action is a **discrete** zoom — one that arrives in one piece
/// and therefore wants an anchor.
#[must_use]
pub fn is_discrete_zoom(action: &Action) -> bool {
    matches!(action, Action::ZoomIn | Action::ZoomOut | Action::ZoomTo(_))
}

/// **Arm the anchor for any discrete zoom in `actions`.** The one-line
/// integration point, and the one to prefer.
pub fn arm_for_actions(ctx: &Context, doc: &mut OpenDoc, actions: &[Action]) {
    if doc.frame.zoom_anchor.is_none() && actions.iter().any(is_discrete_zoom) {
        arm_anchor(ctx, doc);
    }
}

/// **Zoom in / out / actual size, anchored.** The explicit alternative to
/// [`arm_for_actions`], for a caller that would rather say so at the call
/// site than rely on a funnel.
pub fn zoom_step(ctx: &Context, doc: &mut OpenDoc, step: ZoomStep, actions: &mut Vec<Action>) {
    arm_anchor(ctx, doc);
    actions.push(step.action());
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("canvas-zoom step={step:?} from={:.4}", doc.view.zoom)
    });
}

/// **A modified wheel notch, turned into an anchored zoom** — the body of the
/// Ctrl+wheel gesture, with the hover gate left to the caller.
pub fn wheel_step(ctx: &Context, doc: &mut OpenDoc, actions: &mut Vec<Action>) {
    let factor = ctx.input(|i| i.zoom_delta());
    if (factor - 1.0).abs() <= f32::EPSILON {
        return;
    }
    //
    //
    arm_anchor(ctx, doc);
    actions.push(Action::ZoomBy(factor));
}

// ---------------------------------------------------------------------------
// Entry points — the framing commands
// ---------------------------------------------------------------------------

/// What a framing zoom did, or why it did not.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ZoomOutcome {
    /// The view was moved.
    Zoomed {
        /// The scale that would have fitted the region exactly.
        requested: f32,
        /// The scale actually pinned, after [`viewer::clamp_zoom`] applied the
        /// per-page raster ceiling. Equal to `requested` unless the ceiling or
        /// the floor bit.
        applied: f32,
    },
    /// **The selection has no resolvable bounds**, so there is nothing to
    /// frame. Raised when nothing is selected, and equally when something *is*
    /// selected but lies on another page or no longer resolves against the
    /// current decomposition — from the operator's side those are one
    /// situation: *"there is nothing on screen for this command to act on."*
    NoBounds,
    /// The canvas has not drawn yet, so there is no viewport to frame
    /// anything into. Distinct from [`Self::NoBounds`] because the operator's
    /// remedy is different and because a harness reading the trace is entitled
    /// to know which happened.
    NoCanvas,
}

impl ZoomOutcome {
    /// Whether the per-page raster ceiling (or the floor) changed the answer.
    pub fn ceiling_changed_the_answer(self) -> bool {
        match self {
            Self::Zoomed { requested, applied } => (requested - applied).abs() > 1e-4,
            Self::NoBounds | Self::NoCanvas => false,
        }
    }
}

/// Grow a region to [`MIN_REGION_EXTENT`] on each axis, about its own centre.
#[must_use]
pub fn framed_region(rect: Rect) -> Rect {
    let rect = Rect::from_two_pos(rect.min, rect.max);
    if !rect.min.x.is_finite()
        || !rect.min.y.is_finite()
        || !rect.max.x.is_finite()
        || !rect.max.y.is_finite()
    {
        return rect;
    }
    let pad = |extent: f32| ((MIN_REGION_EXTENT - extent) / 2.0).max(0.0);
    Rect::from_min_max(
        Pos2::new(
            rect.min.x - pad(rect.width()),
            rect.min.y - pad(rect.height()),
        ),
        Pos2::new(
            rect.max.x + pad(rect.width()),
            rect.max.y + pad(rect.height()),
        ),
    )
}

/// A framing zoom, decided in full — **without a document, so it is testable
/// without one.**
#[derive(Debug, Clone, Copy)]
pub struct FramingPlan {
    /// What the operator asked for and what the page's raster ceiling allowed.
    pub outcome: ZoomOutcome,
    /// The anchor that will centre the region once the zoom lands.
    pub anchor: ZoomAnchor,
}

/// Decide a framing zoom: the scale that fits `region`, the scale the page's
/// raster ceiling actually permits, and the anchor that centres it.
#[must_use]
pub fn plan_framing(
    frame: &CanvasFrame,
    region: Rect,
    margin: f32,
    pixels_per_point: f32,
    // O218: the operator's View ▸ Render ▸ Quality. It belongs beside
    // `pixels_per_point` and not somewhere else, because the two are the two
    // halves of `viewer::raster_density` — a ceiling derived from one without
    // the other is wrong by the multiplier, which on Sharper is 1.5× and hands
    // the engine a pixmap it refuses.
    quality: crate::app::prefs::RenderQuality,
    // O24: the operator's configured maximum, as a percentage. Threaded
    // rather than read from a global for the same reason `pixels_per_point`
    // is — this function stays pure with respect to egui and to app state,
    // which is what keeps it reviewable and unit-testable.
    max_zoom_percent: f32,
    // O186: the raster ceiling this page has already been measured to have,
    // as a raster SCALE, or `None` if it has never refused a render — which is
    // the answer for every page of every document until one does.
    //
    // Threaded for the same reason `max_zoom_percent` is, and the purity it
    // preserves is load-bearing here rather than stylistic: the caller
    // ([`frame_rect`]) holds the `OpenDoc` and can ask
    // `RasterCeiling::for_page` the page-and-epoch question, and this function
    // could not — it is given a `CanvasFrame`, which knows a page's extent and
    // not its index.
    learned_raster_scale: Option<f32>,
) -> FramingPlan {
    let region = framed_region(region);
    let viewport = (
        (frame.viewport.0 - margin).max(1.0),
        (frame.viewport.1 - margin).max(1.0),
    );
    let requested = viewer::fit_scale((region.width(), region.height()), viewport, FitMode::Page);
    // The ceiling is recomputed here rather than read from anywhere: it is a
    // function of this page and this display density, both of which are known,
    // and caching it would be a second copy of a number `app::actions` also
    // derives per action.
    let applied = viewer::clamp_zoom(
        requested,
        viewer::zoom_ceiling(
            frame.extent,
            pixels_per_point,
            quality,
            max_zoom_percent,
            learned_raster_scale,
        ),
    );
    FramingPlan {
        outcome: ZoomOutcome::Zoomed { requested, applied },
        anchor: place_centred(frac_of(region.center(), frame.extent), frame),
    }
}

/// **Zoom so a canvas-space region fills the viewport, centred.** The shared
/// verb behind both marquee-zoom and zoom-to-selection.
pub fn zoom_to_rect(
    ctx: &Context,
    doc: &mut OpenDoc,
    region: Rect,
    margin: f32,
    // O24: the operator's configured maximum, threaded to `plan_framing`.
    max_zoom_percent: f32,
    actions: &mut Vec<Action>,
) -> ZoomOutcome {
    // ui-text-exempt: trace field value, never displayed
    trace_outcome(
        "rect",
        frame_rect(ctx, doc, region, margin, max_zoom_percent, actions),
    )
}

/// Apply a [`FramingPlan`] to the document — the body both framing verbs
/// share, minus the trace, so that each of them reports itself under its own
/// name and a harness can tell which command ran.
fn frame_rect(
    ctx: &Context,
    doc: &mut OpenDoc,
    region: Rect,
    margin: f32,
    // O24: the operator's configured maximum, threaded to `plan_framing`.
    max_zoom_percent: f32,
    actions: &mut Vec<Action>,
) -> ZoomOutcome {
    let Some(frame) = last_frame(ctx) else {
        return ZoomOutcome::NoCanvas;
    };
    // O186. Resolved here because this is the first frame in the call chain
    // that holds the document: `plan_framing` below is deliberately pure and
    // `zoom_ceiling` below that is deliberately ignorant of page identity, so
    // the page-and-epoch question can only be asked at this level.
    //
    // It matters that a FRAMING zoom is capped too, and not only the ladder.
    // "Zoom to selection" on a small object is the single easiest way to ask
    // for an enormous magnification in one gesture — no wheel notches, no
    // typing a percentage — which makes it the most likely route to the wall,
    // and it was the route a driven check found first.
    let learned = doc.raster_ceiling.for_page(
        doc.view.page_index,
        doc.page_epochs.get(doc.view.page_index),
    );
    let plan = plan_framing(
        &frame,
        region,
        margin,
        ctx.pixels_per_point(),
        doc.prefs.render_quality,
        max_zoom_percent,
        learned,
    );
    doc.frame.zoom_anchor = Some(plan.anchor);
    if let ZoomOutcome::Zoomed { applied, .. } = plan.outcome {
        actions.push(Action::ZoomTo(applied));
    }
    plan.outcome
}

/// **Zoom to the selection.** The entry point `view.zoom_selection` calls.
pub fn zoom_to_selection(
    ctx: &Context,
    doc: &mut OpenDoc,
    margin: f32,
    // O24: the operator's configured maximum, threaded to `plan_framing`.
    max_zoom_percent: f32,
    actions: &mut Vec<Action>,
) -> ZoomOutcome {
    let Some(bounds) = doc.selection.outline_union() else {
        // ui-text-exempt: trace field value, never displayed
        return trace_outcome("selection", ZoomOutcome::NoBounds);
    };
    let outcome = frame_rect(ctx, doc, bounds, margin, max_zoom_percent, actions);
    // ui-text-exempt: trace field value, never displayed
    trace_outcome("selection", outcome)
}

/// Whether [`zoom_to_selection`] would have anything to do — **the condition a
/// `view.zoom_selection` command must be bound to**, so the control greys out
/// instead of no-opping.
#[must_use]
pub fn can_zoom_to_selection(doc: &OpenDoc) -> bool {
    doc.selection.outline_union().is_some()
}

// ---------------------------------------------------------------------------
// Marquee zoom — the one-shot arming
// ---------------------------------------------------------------------------

/// Arm the next primary drag on the canvas to **zoom to the region it
/// encloses** instead of selecting what it encloses. The entry point
/// `view.zoom_region` calls.
pub fn arm_region_zoom(ctx: &Context) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(REGION_MEMORY_KEY), true));
}

/// Whether the next marquee will zoom. Read by the canvas at **press** time
/// and by a ribbon toggle that wants to render itself as armed.
#[must_use]
pub fn region_zoom_armed(ctx: &Context) -> bool {
    ctx.data(|d| {
        d.get_temp::<bool>(egui::Id::new(REGION_MEMORY_KEY))
            .unwrap_or(false)
    })
}

/// Disarm the marquee zoom, returning whether it had been armed.
pub fn disarm_region_zoom(ctx: &Context) -> bool {
    let was = region_zoom_armed(ctx);
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(REGION_MEMORY_KEY), false));
    was
}

/// Report a framing zoom on the `PDFCER_DIAG` channel and hand the outcome
/// back.
fn trace_outcome(to: &str, outcome: ZoomOutcome) -> ZoomOutcome {
    crate::diag::trace(|| match outcome {
        ZoomOutcome::Zoomed { requested, applied } => format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-zoom to={to} requested={requested:.4} applied={applied:.4} clamped={}",
            outcome.ceiling_changed_the_answer()
        ),
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        ZoomOutcome::NoBounds => format!("canvas-zoom to={to} declined=no-bounds"),
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        ZoomOutcome::NoCanvas => format!("canvas-zoom to={to} declined=no-canvas"),
    });
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::vec2;

    /// A 200×300 page drawn at `zoom`, with its top-left at a non-zero screen
    /// position inside a 400×400 viewport — an origin a bug could show up in.
    fn frame(zoom: f32) -> CanvasFrame {
        let extent = (200.0_f32, 300.0_f32);
        let display = (extent.0 * zoom, extent.1 * zoom);
        let image_rect = Rect::from_min_size(Pos2::new(37.0, 11.0), vec2(display.0, display.1));
        CanvasFrame {
            map: PageMapping::new(image_rect, extent, zoom),
            extent,
            display,
            viewport: (400.0, 400.0),
            // No scroll bar in a test world, so the outer and inner sizes
            // agree. See `CanvasFrame::outer` for when they do not.
            outer: (400.0, 400.0),
            viewport_rect: Rect::from_min_size(Pos2::new(20.0, 5.0), vec2(400.0, 400.0)),
            offset: (0.0, 0.0),
            // The single-page world every test in here builds: one page,
            // at the strip origin. See `ZoomAnchor::page`.
            page: 0,
        }
    }

    // ---- the rule -----------------------------------------------------

    /// **The pointer wins when it is over the canvas.**
    #[test]
    fn a_pointer_over_the_canvas_is_the_anchor() {
        let f = frame(2.0);
        // 137,111 on screen is (137-37)/2, (111-11)/2 = 50,50 in canvas space.
        let p = anchor_point(Some(Pos2::new(137.0, 111.0)), &f);
        assert!(
            (p.x - 50.0).abs() < 1e-3 && (p.y - 50.0).abs() < 1e-3,
            "{p:?}"
        );
    }

    /// **A pointer that is not over the canvas falls back to the viewport
    /// centre — never to the page's top-left**, which is the defect this rule
    /// replaces.
    #[test]
    fn a_pointer_elsewhere_falls_back_to_the_viewport_centre_not_the_origin() {
        let f = frame(2.0);
        let centre = f.map.to_page(f.viewport_rect.center());
        for pointer in [
            None,
            // Over the ribbon, above the canvas.
            Some(Pos2::new(200.0, -400.0)),
            // Over a dock, to the right of it.
            Some(Pos2::new(5_000.0, 100.0)),
        ] {
            let p = anchor_point(pointer, &f);
            assert!(
                (p.x - centre.x).abs() < 1e-3 && (p.y - centre.y).abs() < 1e-3,
                "{pointer:?} anchored at {p:?}, expected the viewport centre {centre:?}"
            );
            assert!(
                p != Pos2::ZERO,
                "the page's top-left is the behaviour being replaced, not a fallback"
            );
        }
    }

    /// `frac` is the same number at every zoom — the property that lets an
    /// anchor be described before the new zoom is known.
    #[test]
    fn the_anchor_fraction_does_not_depend_on_the_zoom() {
        let point = Pos2::new(50.0, 150.0);
        for zoom in [0.1_f32, 1.0, 8.0] {
            let f = frame(zoom);
            let frac = frac_of(point, f.extent);
            assert!(
                (frac.0 - 0.25).abs() < 1e-6 && (frac.1 - 0.5).abs() < 1e-6,
                "{frac:?}"
            );
        }
    }

    /// A degenerate page yields the middle of the page rather than a NaN that
    /// would reach a scroll offset.
    #[test]
    fn a_degenerate_extent_anchors_at_the_middle_rather_than_at_nan() {
        assert_eq!(frac_of(Pos2::new(10.0, 10.0), (0.0, 100.0)), (0.5, 0.1));
        assert_eq!(
            frac_of(Pos2::new(f32::NAN, 10.0), (100.0, 100.0)),
            (0.5, 0.1)
        );
    }

    // ---- the handshake -------------------------------------------------

    /// **An anchor armed before the zoom action is applied survives the
    /// frame it was armed on** — the gate without which every discrete zoom
    /// command silently does nothing.
    #[test]
    fn an_anchor_waits_for_the_zoom_to_land_and_is_then_solved() {
        let f = frame(1.0);
        let anchor = hold((0.5, 0.5), &f);
        // Frame it was armed on: the action has not been applied, so the page
        // is still the same size.
        assert_eq!(anchor_step(&anchor, f.display, false), AnchorStep::Hold);
        // The next frame: the zoom landed.
        assert_eq!(
            anchor_step(&anchor, (f.display.0 * 2.0, f.display.1 * 2.0), true),
            AnchorStep::Solve
        );
    }

    /// **A zoom that never landed drops its anchor** rather than leaving it
    /// pending to be spent on an unrelated layout change frames later.
    #[test]
    fn an_anchor_whose_zoom_never_landed_is_dropped_after_one_frame() {
        let f = frame(1.0);
        let anchor = hold((0.5, 0.5), &f);
        assert_eq!(anchor_step(&anchor, f.display, false), AnchorStep::Hold);
        assert_eq!(anchor_step(&anchor, f.display, true), AnchorStep::Drop);
    }

    /// The wheel's anchor is solved on the very next frame with no grace
    /// needed, because it is armed at the *end* of a frame whose zoom action
    /// is applied immediately after.
    #[test]
    fn the_wheel_path_still_solves_on_the_next_frame() {
        let f = frame(1.0);
        let anchor = hold((0.75, 0.75), &f);
        assert_eq!(
            anchor_step(&anchor, (f.display.0 * 1.5, f.display.1 * 1.5), false),
            AnchorStep::Solve
        );
    }

    // ---- framing --------------------------------------------------------

    /// **A framing anchor really does land its point at the centre of the
    /// viewport**, at the scale that was granted.
    #[test]
    fn framing_puts_the_regions_centre_in_the_middle_of_the_viewport() {
        let f = frame(1.0);
        // Deliberately near the middle of the page: a region close to an edge
        // cannot be centred without scrolling blank space into view, and the
        // scrollable-range clamp correctly refuses to — see
        // `a_region_near_the_page_edge_saturates_rather_than_centring`.
        let region = Rect::from_min_max(Pos2::new(80.0, 120.0), Pos2::new(120.0, 180.0));
        let frac = frac_of(region.center(), f.extent);
        let anchor = place_centred(frac, &f);

        // The zoom lands: the page is now drawn four times bigger.
        let display_after = (f.display.0 * 4.0, f.display.1 * 4.0);
        let (ox, oy) = geometry::zoom_anchor_offset(
            anchor.offset_before,
            anchor.display_before,
            display_after,
            anchor.viewport,
            anchor.frac,
        );
        let landed = geometry::anchor_screen_pos(frac, (ox, oy), display_after, f.viewport);
        assert!(
            (landed.0 - f.viewport.0 / 2.0).abs() < 0.01
                && (landed.1 - f.viewport.1 / 2.0).abs() < 0.01,
            "the region's centre landed at {landed:?}, not the viewport centre"
        );
    }

    /// **A region at the page's very corner CAN now be centred** — and
    /// this test is the record of that changing.
    #[test]
    fn a_region_at_the_page_corner_can_be_centred_because_the_pasteboard_is_there() {
        let f = frame(1.0);
        // Hard against the page's top-left corner.
        let region = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(40.0, 60.0));
        let frac = frac_of(region.center(), f.extent);
        let anchor = place_centred(frac, &f);
        let display_after = (f.display.0 * 4.0, f.display.1 * 4.0);
        let (ox, oy) = geometry::zoom_anchor_offset(
            anchor.offset_before,
            anchor.display_before,
            display_after,
            anchor.viewport,
            anchor.frac,
        );
        assert!(
            ox < 0.0 && oy < 0.0,
            "framing the top-left corner must ask to scroll ABOVE and LEFT of the page; got \
             ({ox}, {oy})"
        );

        // …and the ask is honoured: the region lands in the middle of the
        // viewport, which is what centring means and what the operator asked
        // for. This is the assertion the old test could not make.
        let landed = geometry::anchor_screen_pos(frac, (ox, oy), display_after, f.viewport);
        assert!(
            (landed.0 - f.viewport.0 / 2.0).abs() < 0.01
                && (landed.1 - f.viewport.1 / 2.0).abs() < 0.01,
            "the corner region landed at {landed:?}, not the viewport centre"
        );

        // And the offset that actually reaches the scroll area is inside the
        // content, because `strip_offset` clamps against `content_extent` —
        // the pasteboard included. The saturation the old test protected is
        // still there; it is just further out.
        let reached = geometry::strip_offset(
            (ox, oy),
            (0.0, 0.0),
            display_after,
            display_after,
            f.viewport,
            (0.0, 0.0),
        );
        let range = (
            geometry::content_extent(display_after.0, f.viewport.0, 0.0) - f.viewport.0,
            geometry::content_extent(display_after.1, f.viewport.1, 0.0) - f.viewport.1,
        );
        assert!(
            reached.0 >= 0.0 && reached.0 <= range.0,
            "x offset {} outside [0, {}]",
            reached.0,
            range.0
        );
        assert!(
            reached.1 >= 0.0 && reached.1 <= range.1,
            "y offset {} outside [0, {}]",
            reached.1,
            range.1
        );
    }

    /// A one-pixel marquee and a zero-height selection are both grown to
    /// something fittable, about their own centre.
    #[test]
    fn a_degenerate_region_is_grown_about_its_own_centre() {
        let hairline = Rect::from_min_max(Pos2::new(100.0, 200.0), Pos2::new(300.0, 200.0));
        let grown = framed_region(hairline);
        assert!((grown.height() - MIN_REGION_EXTENT).abs() < 1e-4);
        assert!(
            (grown.width() - 200.0).abs() < 1e-4,
            "the wide axis is untouched"
        );
        assert!(
            (grown.center().y - 200.0).abs() < 1e-4,
            "and it stays centred"
        );

        let speck = Rect::from_min_max(Pos2::new(10.0, 10.0), Pos2::new(10.0, 10.0));
        let grown = framed_region(speck);
        assert!((grown.width() - MIN_REGION_EXTENT).abs() < 1e-4);
        assert!((grown.height() - MIN_REGION_EXTENT).abs() < 1e-4);
    }

    /// A marquee dragged up-and-left frames the same region as one dragged
    /// down-and-right. Without the normalisation the negative extents make
    /// `fit_scale` fall back to actual size and the command looks broken in
    /// exactly two of its four directions.
    #[test]
    fn a_backwards_marquee_frames_the_same_region() {
        let forwards = Rect::from_min_max(Pos2::new(10.0, 20.0), Pos2::new(110.0, 220.0));
        let backwards = Rect::from_two_pos(Pos2::new(110.0, 220.0), Pos2::new(10.0, 20.0));
        assert_eq!(framed_region(forwards), framed_region(backwards));
    }

    /// **The ceiling is reported, not hidden.** A region small enough to ask
    /// for more magnification than the page's raster allows still zooms — to
    /// the ceiling — and says that the answer was changed.
    #[test]
    fn a_region_past_the_raster_ceiling_reports_that_it_was_clamped() {
        let clamped = ZoomOutcome::Zoomed {
            requested: 40.0,
            applied: viewer::MAX_ZOOM,
        };
        assert!(clamped.ceiling_changed_the_answer());
        let granted = ZoomOutcome::Zoomed {
            requested: 2.0,
            applied: 2.0,
        };
        assert!(!granted.ceiling_changed_the_answer());
        assert!(!ZoomOutcome::NoBounds.ceiling_changed_the_answer());
    }

    // ---- arming ---------------------------------------------------------

    /// The marquee zoom arms, reports itself armed, and disarms once —
    /// reporting on the way out whether it had been armed, which is what lets
    /// Escape spend itself on exactly one thing.
    #[test]
    fn the_region_zoom_arms_once_and_disarms_once() {
        let ctx = Context::default();
        assert!(!region_zoom_armed(&ctx));
        assert!(
            !disarm_region_zoom(&ctx),
            "disarming an idle canvas retires nothing"
        );
        arm_region_zoom(&ctx);
        assert!(region_zoom_armed(&ctx));
        assert!(disarm_region_zoom(&ctx));
        assert!(!region_zoom_armed(&ctx));
    }

    /// The frame record round-trips, and is absent before the canvas has ever
    /// drawn — the state every entry point declines on rather than guessing.
    #[test]
    fn the_frame_record_is_absent_until_the_canvas_has_drawn() {
        let ctx = Context::default();
        assert!(last_frame(&ctx).is_none());
        remember_frame(&ctx, frame(1.5));
        let back = last_frame(&ctx).expect("the frame was just recorded");
        assert_eq!(back.extent, (200.0, 300.0));
        assert!((back.display.0 - 300.0).abs() < 1e-4);
    }

    /// **Every discrete zoom is recognised, and the wheel is not.**
    #[test]
    fn the_discrete_zooms_are_recognised_and_the_wheel_is_not() {
        assert!(is_discrete_zoom(&Action::ZoomIn));
        assert!(is_discrete_zoom(&Action::ZoomOut));
        assert!(is_discrete_zoom(&Action::ZoomTo(1.0)));
        assert!(
            !is_discrete_zoom(&Action::ZoomBy(1.1)),
            "the wheel arms its own anchor from the pointer it can see"
        );
        assert!(!is_discrete_zoom(&Action::NextPage));
        assert!(
            !is_discrete_zoom(&Action::Fit(FitMode::Page)),
            "a fit mode re-derives from the viewport and centres by construction"
        );
    }

    /// **The bounds a zoom-to-selection needs come from the selection layer,
    /// and an empty selection has none** — the input side of the decline,
    /// asserted where it can be asserted without a document.
    #[test]
    fn an_empty_selection_offers_no_bounds_to_frame() {
        use crate::canvas::selection::SelectionState;
        assert!(SelectionState::default().outline_union().is_none());
    }

    /// A region far smaller than the page asks for more magnification than the
    /// raster ceiling allows, and the plan carries **both** numbers: the
    /// requested scale and the one that will be pinned.
    #[test]
    fn a_plan_past_the_ceiling_carries_the_clamped_scale_as_well_as_the_asked_one() {
        let f = frame(1.0);
        let plan = plan_framing(
            &f,
            Rect::from_min_max(Pos2::new(50.0, 50.0), Pos2::new(51.0, 51.0)),
            16.0,
            1.0,
            crate::app::prefs::RenderQuality::Normal,
            viewer::MAX_ZOOM * 100.0,
            None,
        );
        match plan.outcome {
            ZoomOutcome::Zoomed { requested, applied } => {
                assert!(requested > viewer::MAX_ZOOM, "the ask must be over-range");
                assert!(applied <= viewer::MAX_ZOOM, "the answer must not be");
            }
            other => panic!("a real region must produce a zoom, got {other:?}"),
        }
        assert!(plan.outcome.ceiling_changed_the_answer());
    }

    /// A region the page can actually fit is granted exactly, and the ceiling
    /// reports nothing — the other direction, without which the test above
    /// would pass on a build that clamped everything.
    #[test]
    fn a_plan_within_the_ceiling_is_granted_exactly() {
        let f = frame(1.0);
        // Half the page wide: (400-16)/100 = 3.84, well inside MAX_ZOOM.
        let plan = plan_framing(
            &f,
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 150.0)),
            16.0,
            1.0,
            crate::app::prefs::RenderQuality::Normal,
            crate::app::prefs::DEFAULT_MAX_ZOOM_PERCENT,
            None,
        );
        match plan.outcome {
            ZoomOutcome::Zoomed { requested, applied } => {
                assert!((requested - applied).abs() < 1e-6)
            }
            other => panic!("expected a zoom, got {other:?}"),
        }
        assert!(!plan.outcome.ceiling_changed_the_answer());
    }
}
