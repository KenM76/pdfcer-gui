//! # `canvas::zoom` — the anchor rule, decided once, and the five paths that route through it
//!
//! ## ★ The rule
//!
//! > **A zoom holds one page point still, and that point is where the operator
//! > is looking: the pointer when it is over the canvas, the centre of the
//! > viewport when it is not. A zoom that *frames* something — a selection, a
//! > marquee'd region — holds that thing's centre still, at the centre of the
//! > viewport.**
//!
//! It lives in [`anchor_point`] and in nothing else, and every zoom in the
//! product goes through it:
//!
//! | path | anchor | how it gets here |
//! |---|---|---|
//! | Ctrl+wheel | the pointer | [`arm_anchor`], from `canvas::show` |
//! | Ctrl+Plus / Ctrl+Minus | pointer, else viewport centre | [`arm_for_actions`], or [`zoom_step`] |
//! | Ctrl+0 (actual size) | pointer, else viewport centre | [`arm_for_actions`], or [`zoom_step`] |
//! | Zoom to selection | the selection's centre → viewport centre | [`zoom_to_selection`] |
//! | Marquee zoom to region | the region's centre → viewport centre | [`zoom_to_rect`] |
//!
//! `FEATURES.md` records the discrete commands as deferred *"so the rule is
//! decided once for all four"*. This module is that decision, and it is
//! deliberately a *rule about the operator's attention* rather than a rule
//! about arithmetic:
//!
//! * **The pointer wins when it is over the canvas.** It is the only evidence
//!   the application has about what the operator is looking at, it is what the
//!   wheel path already honours (measured under 0.01 px of drift), and a
//!   keyboard zoom that ignored a pointer resting on the detail being
//!   inspected would behave differently from the wheel for no reason the
//!   operator could see.
//! * **The viewport centre when it is not.** A Ctrl+Plus pressed with the
//!   pointer parked over the Objects panel, off-window, or in the ribbon must
//!   not zoom about a point outside the page. The centre of what is on screen
//!   is the only other honest candidate; the page's **top-left** — today's
//!   behaviour, and the defect — is not a candidate at all, because it is
//!   where nobody is looking.
//! * **Framing is the same solve with a different target.** "Zoom to this
//!   rect" is not a different feature from "zoom about this point"; it is
//!   *put this point at the centre* instead of *put this point back where it
//!   was*. [`crate::canvas::geometry::offset_holding_anchor_at`] is the half
//!   they share.
//!
//! ## Why the anchor is a CANVAS-space point and not a screen position
//!
//! Because a screen position stops naming the thing it named the instant the
//! zoom lands — which is the same reason [`crate::canvas::selection`] holds
//! identity rather than coordinates. `frac`, the fraction of the page's drawn
//! size that [`crate::app::state::ZoomAnchor`] carries, is
//! `canvas_point / extent` and therefore **independent of the zoom**: it can
//! be computed before the new zoom is known, which is exactly the situation
//! every one of these commands is in.
//!
//! ## ★ The two-frame handshake, and the gate that makes it work for commands
//!
//! `ZoomAnchor`'s own docs explain why it spans two frames: *"the new zoom is
//! not known when the wheel is seen … recording the inputs and solving later
//! avoids predicting a clamp we do not control."* The wheel is seen **during**
//! `canvas::show`, so the anchor it records is always consumed on the frame
//! after, by which time the action has been applied and the page's real drawn
//! size is known.
//!
//! A **command** is not seen during `show`. Keyboard chords are dispatched at
//! step 1a of the frame, the ribbon at 1b, the status bar at 1b² — all *before*
//! the canvas draws — while the zoom action they raise is applied at step 3,
//! *after*. An anchor armed at step 1a and consumed unconditionally at the top
//! of `show` would therefore be spent on a frame that still shows the old zoom:
//! `display_after == display_before`, the solve is a no-op, and the anchor is
//! gone before the zoom it was for ever lands. That is precisely how a
//! plausible implementation of this feature ships doing nothing.
//!
//! So consumption is gated by [`anchor_step`]: **an anchor is solved only on a
//! frame whose drawn page size differs from the size recorded when it was
//! armed** — i.e. only once the zoom has actually landed. One frame of grace is
//! allowed for the action to be applied, and an anchor still unspent after that
//! is *dropped* rather than held, because a zoom that never changed the page
//! size (Ctrl+Plus already at the raster ceiling) has nothing to re-anchor, and
//! an anchor left pending would be spent much later on an unrelated layout
//! change — a page step, a window resize under a fit mode — as a visible jump.
//!
//! ## Where the per-frame geometry comes from
//!
//! An entry point called from the command dispatcher has no `Ui`, no page
//! rect, no viewport and no scroll offset. [`CanvasFrame`] is the canvas's own
//! record of those, written at the end of every `show` and read here. It is
//! *last drawn frame's* geometry, which is the correct "before" state in both
//! call orders — before `show`, the layout has not changed since; after `show`
//! but before the actions are applied, it **is** this frame's geometry.

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
///
/// A marquee can be dragged three pixels, and a selected horizontal rule has a
/// real bounding box that is *exactly zero* high (see
/// [`crate::canvas::overlay::visible_outline_rect`], which grows outlines for
/// the same reason). Fitting either literally asks for an unbounded scale,
/// which the ceiling then clamps to something that shows the operator a
/// featureless field of ink. Growing the region to a minimum first makes the
/// answer *"as close as this page can go, framed on what you pointed at"*
/// rather than *"as close as this page can go, framed on nothing"*.
///
/// Applied symmetrically about the region's own centre, so the thing the
/// operator aimed at stays in the middle of what they get.
pub const MIN_REGION_EXTENT: f32 = 8.0;

// ---------------------------------------------------------------------------
// The frame record
// ---------------------------------------------------------------------------

/// The geometry the last drawn canvas frame settled on — everything an entry
/// point outside the canvas needs in order to describe a zoom.
///
/// `Copy` and small, for the same reason [`PageMapping`] is: it is a fact
/// about one frame, and anything that outlived a frame would be a mapping for
/// a page rect that has since moved.
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
    /// ★ Under a continuous mode `map`, `extent`, `display` and `offset` are
    /// all about one page of a strip, and an anchor built from them is only
    /// meaningful against that same page. Carrying the index is what lets the
    /// canvas convert the solve's answer back through the **right** page's
    /// origin a frame later, when the current page may have moved on. See
    /// [`crate::viewer::ZoomAnchor::page`].
    pub page: usize,
    /// ★★★ **The OUTER viewport** — `ui.available_size()` measured *before* the
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
    pub outer: (f32, f32),
}

/// Record this frame's canvas geometry. Called once, at the end of
/// [`crate::canvas::show`].
pub fn remember_frame(ctx: &Context, frame: CanvasFrame) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(FRAME_MEMORY_KEY), frame));
}

/// The last drawn frame's canvas geometry, or `None` before the canvas has
/// ever drawn a page.
///
/// `None` is a real state and every entry point declines on it rather than
/// guessing: before the first frame there is no viewport, no page rect and no
/// offset, and a zoom described against invented geometry would move the view
/// to somewhere the operator did not ask for.
#[must_use]
pub fn last_frame(ctx: &Context) -> Option<CanvasFrame> {
    ctx.data(|d| d.get_temp::<CanvasFrame>(egui::Id::new(FRAME_MEMORY_KEY)))
}

// ---------------------------------------------------------------------------
// The rule
// ---------------------------------------------------------------------------

/// ★ **The anchor rule.** The canvas-space point a zoom step must hold still.
///
/// `pointer` is the pointer's latest screen position, if it has one. It is
/// honoured when it lies inside the canvas viewport; otherwise the viewport's
/// own centre is used. See the module docs for why those two, and why the
/// page's top-left is not a third option.
///
/// Note what is *not* consulted: the zoom. The result is a canvas coordinate,
/// which is the same number before and after the step — that is what allows an
/// anchor to be described before the new zoom is known.
#[must_use]
pub fn anchor_point(pointer: Option<Pos2>, frame: &CanvasFrame) -> Pos2 {
    let screen = pointer
        .filter(|p| frame.viewport_rect.contains(*p))
        .unwrap_or_else(|| frame.viewport_rect.center());
    frame.map.to_page(screen)
}

/// A canvas-space point as a fraction of the page's drawn size — the form
/// [`ZoomAnchor`] carries.
///
/// `canvas_point / extent`, and therefore **zoom-independent**: `display` is
/// `extent × zoom` and a canvas point projects to `canvas_point × zoom` inside
/// it, so the ratio cancels. Dividing by the *drawn size* instead would give
/// the same number by a longer route and would need a zoom to do it, which is
/// the argument for computing it here.
///
/// A degenerate extent yields `0.5` on that axis — the middle of the page —
/// rather than a NaN that would reach a scroll offset. `viewer::clamp_zoom`'s
/// discipline: fail to a finite, harmless value.
#[must_use]
pub fn frac_of(point: Pos2, extent: (f32, f32)) -> (f32, f32) {
    fn axis(v: f32, extent: f32) -> f32 {
        if extent.is_finite() && extent > 0.0 && v.is_finite() {
            v / extent
        } else {
            0.5
        }
    }
    (axis(point.x, extent.0), axis(point.y, extent.1))
}

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
///
/// # How one struct expresses both
///
/// [`crate::canvas::geometry::zoom_anchor_offset`] reads its "before" fields
/// only through [`crate::canvas::geometry::anchor_screen_pos`], i.e. only as
/// the single quantity *"where was the anchor on screen"*. So placing the
/// anchor somewhere else is a matter of stating that quantity directly, and
/// `offset_before` is solved backwards from it with
/// [`crate::canvas::geometry::offset_holding_anchor_at`] — the exact inverse,
/// pinned by `placing_an_anchor_and_measuring_it_are_exact_inverses`.
///
/// The field is therefore truthful in the only sense the solver uses it: it is
/// *the offset at which the anchor would have been sitting in the middle of the
/// view*. It is not this frame's scroll offset, and it is not claimed to be.
/// The alternative — a second `Option` field on `OpenDoc` and a second consume
/// path in `show` — would be two mechanisms for one two-frame handshake, and
/// the second one would be the one that gets the clamp gate wrong.
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
///
/// `waited` is whether this same anchor already saw one frame in which nothing
/// had changed. One frame of grace and no more: an action raised at step 1a is
/// applied at step 3 of the same frame, so a zoom that is going to land has
/// landed by the *next* frame's `show`, and anything still pending after that
/// is a zoom that did not happen.
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
///
/// The whole of `canvas::show`'s zoom-anchor wiring, so that the gate, the
/// solve and the one-frame grace counter cannot be re-derived differently at
/// the call site.
pub fn consume_anchor(ctx: &Context, doc: &mut OpenDoc, display_now: (f32, f32)) -> Option<Vec2> {
    let anchor = doc.zoom_anchor?;
    let waited_id = egui::Id::new(WAITED_MEMORY_KEY);
    let waited = ctx.data(|d| d.get_temp::<bool>(waited_id).unwrap_or(false));
    match anchor_step(&anchor, display_now, waited) {
        AnchorStep::Hold => {
            ctx.data_mut(|d| d.insert_temp(waited_id, true));
            None
        }
        AnchorStep::Drop => {
            doc.zoom_anchor = None;
            ctx.data_mut(|d| d.insert_temp(waited_id, false));
            None
        }
        AnchorStep::Solve => {
            doc.zoom_anchor = None;
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
    /// ★ Named after `view.zoom_in` until 2026-08-15, when that dispatch arm
    /// was deleted: no such command is registered, so the id named nothing an
    /// operator could press. The two routes above are the real ones —
    /// `RIBBON_IA.md` §6 puts `zoom −/%/+` on the status bar deliberately.
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
    ///
    /// `ActualSize` is [`Action::ZoomTo`] and **not** `Fit(FitMode::None)`:
    /// that distinction was a live defect (see `Action::ZoomTo`'s docs — a
    /// control whose label promised 100 % and whose behaviour pinned 73 %),
    /// and it is restated here rather than re-derived because this is now a
    /// second place that has to know it.
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
///
/// The primitive. Call it immediately before pushing a zoom action from
/// anywhere that is not the canvas — a keyboard chord, a ribbon button, the
/// status bar's ± — and the next frame's `show` will keep the anchored point
/// still. Calling it and then raising *no* zoom costs one frame of grace and
/// a `Drop`; it cannot move the view on its own.
///
/// Does nothing before the canvas has drawn (no geometry to describe an anchor
/// against), which is also the state in which no zoom command can be reached.
pub fn arm_anchor(ctx: &Context, doc: &mut OpenDoc) {
    let Some(frame) = last_frame(ctx) else {
        return;
    };
    let point = anchor_point(ctx.pointer_latest_pos(), &frame);
    doc.zoom_anchor = Some(hold(frac_of(point, frame.extent), &frame));
}

/// Whether an action is a **discrete** zoom — one that arrives in one piece
/// and therefore wants an anchor.
///
/// [`Action::ZoomBy`] is deliberately absent: it is the *wheel*, which arms
/// its own anchor inside `canvas::show` from the pointer it can see, and
/// which arrives as a stream of steps rather than as a command.
#[must_use]
pub fn is_discrete_zoom(action: &Action) -> bool {
    matches!(action, Action::ZoomIn | Action::ZoomOut | Action::ZoomTo(_))
}

/// ★ **Arm the anchor for any discrete zoom in `actions`.** The one-line
/// integration point, and the one to prefer.
///
/// Called once per frame at the action funnel — immediately before the
/// actions are applied — it covers **every** surface that can raise a zoom in
/// one call: the keyboard chords collected at step 1a, the manifest chords,
/// the ribbon at 1b, the status bar's ± at 1b², and a canvas context menu.
/// The alternative is arming at each of those five sites, which is the shape
/// the defect already took: `Action::ZoomIn` is raised from three places today
/// and not one of them anchors.
///
/// Two guards, both of which matter:
///
/// * **an anchor already pending is left alone.** The framing verbs
///   ([`zoom_to_rect`], [`zoom_to_selection`]) raise `Action::ZoomTo` *and*
///   arm a centring anchor of their own; overwriting it here with a
///   hold-the-pointer anchor would turn every marquee zoom back into a zoom
///   about the cursor, which is the one thing a marquee zoom must not be.
///   The same guard keeps a wheel anchor armed at the end of the previous
///   frame intact;
/// * **nothing is armed when no zoom is present**, so this is free on the
///   overwhelming majority of frames — one `matches!` per action, over a list
///   that is almost always empty.
pub fn arm_for_actions(ctx: &Context, doc: &mut OpenDoc, actions: &[Action]) {
    if doc.zoom_anchor.is_none() && actions.iter().any(is_discrete_zoom) {
        arm_anchor(ctx, doc);
    }
}

/// ★ **Zoom in / out / actual size, anchored.** The explicit alternative to
/// [`arm_for_actions`], for a caller that would rather say so at the call
/// site than rely on a funnel.
///
/// Arms the anchor and raises the action, in that order, so a caller cannot do
/// one and forget the other — which is the shape the defect took: the actions
/// were raised from three surfaces and none of them anchored.
pub fn zoom_step(ctx: &Context, doc: &mut OpenDoc, step: ZoomStep, actions: &mut Vec<Action>) {
    arm_anchor(ctx, doc);
    actions.push(step.action());
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("canvas-zoom step={step:?} from={:.4}", doc.view.zoom)
    });
}

// ---------------------------------------------------------------------------
// Entry points — the framing commands
// ---------------------------------------------------------------------------

/// What a framing zoom did, or why it did not.
///
/// `#[must_use]` because the *declining* variants are the whole point: a
/// caller that drops this on the floor has silently turned "there is nothing
/// to zoom to" into "the command did nothing", which is the difference between
/// a control that declines and a control that looks broken.
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
    ///
    /// # ★ How the ceiling reports itself, and why this follows rather than
    /// invents
    ///
    /// The ceiling already has a self-report and it is deliberately quiet:
    /// `viewer`'s header states that *"rather than let the operator zoom into
    /// an error message, `max_zoom_for_page` lowers the ceiling per page and
    /// `ViewState` clamps against it — the zoom buttons simply stop"*, and the
    /// status bar's readout then shows the scale that was actually pinned.
    /// The report is **the number on the status bar being the truth**.
    ///
    /// A framing zoom must not break that contract by claiming a fit it did
    /// not get, so it does two things and neither is a new mechanism:
    ///
    /// 1. it raises [`Action::ZoomTo`] carrying the **clamped** scale, so the
    ///    status readout states the real answer on the same frame;
    /// 2. it still frames the region *centred*, at whatever scale it got —
    ///    because the offset is solved on the following frame from the page's
    ///    real drawn size, the framing is correct even when the scale was not
    ///    granted. The operator gets "as close as this page can go, centred on
    ///    what you asked for", which is the honest partial answer.
    ///
    /// ## ★ The surface now exists, and this is deliberately NOT wired to it
    ///
    /// This sentence used to read *"this predicate is what a caller with a
    /// notice surface would key on to say so in words. There is no such
    /// surface in this shell yet."* Both halves are now out of date: the
    /// status bar words declines (`crate::app::status::decline`, 2026-08-14),
    /// and the operator's ruling was that **the clamped region zoom must not
    /// be worded through it.**
    ///
    /// The reason is the two numbered points above, taken seriously. A clamped
    /// framing zoom is **a partial grant, not a decline**:
    ///
    /// * the region really is framed, centred, at the closest scale this page
    ///   can go to — the operator got the honest partial answer;
    /// * the scale that was pinned is already stated, in words, in the one
    ///   place an operator looks for a scale: the status bar's zoom readout,
    ///   on the same frame, because point 1 raises `Action::ZoomTo` carrying
    ///   the clamped number. **The report is the number on the status bar
    ///   being the truth**, and that contract is kept.
    ///
    /// Adding a sentence beside it would word a non-event, and would train the
    /// operator to read a decline line that fires when nothing was declined —
    /// which is how a surface stops being read at all. Only
    /// [`ZoomOutcome::NoBounds`] and [`ZoomOutcome::NoCanvas`] are worded; see
    /// `crate::app::status::decline`'s header, whose `Declined` type cannot
    /// even represent a grant.
    ///
    /// So this predicate keeps exactly the job it has: it feeds the
    /// `PDFCER_DIAG` line, and it is returned to the dispatcher, which reads it
    /// and correctly says nothing.
    pub fn ceiling_changed_the_answer(self) -> bool {
        match self {
            Self::Zoomed { requested, applied } => (requested - applied).abs() > 1e-4,
            Self::NoBounds | Self::NoCanvas => false,
        }
    }
}

/// Grow a region to [`MIN_REGION_EXTENT`] on each axis, about its own centre.
///
/// Also normalises: a marquee is dragged in any of four directions, and a
/// rect whose `min` is not the smaller corner has a negative width that would
/// make [`viewer::fit_scale`] return the degenerate fallback and the zoom a
/// no-op.
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
///
/// The split is this project's standing one (`PROJECT_PLAN.md`: the
/// unit-testable arithmetic on one side, the wiring on the other). Everything
/// that can be wrong about a framing zoom — the scale, the clamp, the anchor,
/// a degenerate region — is decided here and asserted headlessly; the two
/// public verbs below add only "read the selection" and "raise the action".
#[derive(Debug, Clone, Copy)]
pub struct FramingPlan {
    /// What the operator asked for and what the page's raster ceiling allowed.
    pub outcome: ZoomOutcome,
    /// The anchor that will centre the region once the zoom lands.
    pub anchor: ZoomAnchor,
}

/// Decide a framing zoom: the scale that fits `region`, the scale the page's
/// raster ceiling actually permits, and the anchor that centres it.
///
/// The scale is [`viewer::fit_scale`] under [`FitMode::Page`], which is the
/// *same* derivation "Fit page" uses, against the region instead of the page.
/// One derivation, so a region zoom and a page fit cannot disagree about what
/// "fits" means. `margin` is subtracted from the viewport first for the same
/// reason `canvas::show` subtracts it before fitting a page: fitting exactly
/// and then being clipped by the gap is not fitting.
#[must_use]
pub fn plan_framing(
    frame: &CanvasFrame,
    region: Rect,
    margin: f32,
    pixels_per_point: f32,
    // ★ O24: the operator's configured maximum, as a percentage. Threaded
    // rather than read from a global for the same reason `pixels_per_point`
    // is — this function stays pure with respect to egui and to app state,
    // which is what keeps it reviewable and unit-testable.
    max_zoom_percent: f32,
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
        viewer::zoom_ceiling(frame.extent, pixels_per_point, max_zoom_percent),
    );
    FramingPlan {
        outcome: ZoomOutcome::Zoomed { requested, applied },
        anchor: place_centred(frac_of(region.center(), frame.extent), frame),
    }
}

/// ★ **Zoom so a canvas-space region fills the viewport, centred.** The shared
/// verb behind both marquee-zoom and zoom-to-selection.
///
/// `region` is in canvas space — the space the marquee already reports and the
/// space the selection's outlines are cached in — so neither caller performs a
/// coordinate conversion of its own.
pub fn zoom_to_rect(
    ctx: &Context,
    doc: &mut OpenDoc,
    region: Rect,
    margin: f32,
    // ★ O24: the operator's configured maximum, threaded to `plan_framing`.
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
    // ★ O24: the operator's configured maximum, threaded to `plan_framing`.
    max_zoom_percent: f32,
    actions: &mut Vec<Action>,
) -> ZoomOutcome {
    let Some(frame) = last_frame(ctx) else {
        return ZoomOutcome::NoCanvas;
    };
    let plan = plan_framing(
        &frame,
        region,
        margin,
        ctx.pixels_per_point(),
        max_zoom_percent,
    );
    doc.zoom_anchor = Some(plan.anchor);
    if let ZoomOutcome::Zoomed { applied, .. } = plan.outcome {
        actions.push(Action::ZoomTo(applied));
    }
    plan.outcome
}

/// ★ **Zoom to the selection.** The entry point `view.zoom_selection` calls.
///
/// # Where the bounds come from, and what happens when there are none
///
/// The selection is *identity* — page, object, subpath, node — and carries no
/// rectangle. Its bounds are therefore resolved the way every other consumer
/// resolves them: through
/// [`crate::canvas::selection::SelectionState::outline_union`], the union of
/// the outlines the selection layer has already resolved against the current
/// decomposition, in canvas space. That is the same value the eight resize
/// grips are laid out on ([`crate::canvas::overlay::grip_box`]), so **what
/// this command frames is exactly the box the operator can see**, which is the
/// only definition that cannot surprise them.
///
/// `outline_union` returns `None` in three situations that are one situation
/// from the operator's side — nothing is selected, the selection is on another
/// page, or it no longer resolves after an edit — and in all three this
/// declines with [`ZoomOutcome::NoBounds`] and **raises no action at all**.
/// It does not fall back to fit-page, and it does not zoom to the page's
/// origin: a command that quietly did something else when it could not do the
/// thing asked is worse than one that does nothing.
///
/// **The visible half of the decline belongs to the caller**, and
/// [`can_zoom_to_selection`] is what it binds: with no resolvable bounds the
/// command must render *unavailable*, which is this shell's established way of
/// declining visibly (`FEATURES.md`: *"a menu with nothing to offer never
/// opens"*). The outcome returned here is the second line of defence, for the
/// keyboard chord that reaches the verb without passing the condition.
pub fn zoom_to_selection(
    ctx: &Context,
    doc: &mut OpenDoc,
    margin: f32,
    // ★ O24: the operator's configured maximum, threaded to `plan_framing`.
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
///
/// One-shot: [`crate::canvas::show`] disarms it when the drag completes, so
/// the canvas returns to selecting without the operator having to leave a
/// mode. That matches every other marquee-zoom in the product class and it is
/// what keeps this from becoming a fourth thing the primary button might mean
/// with nothing on screen to say which.
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
///
/// The return value is what lets Escape spend itself on exactly one thing: the
/// canvas ascends the selection ladder only when this reports there was
/// nothing armed to retire first.
pub fn disarm_region_zoom(ctx: &Context) -> bool {
    let was = region_zoom_armed(ctx);
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(REGION_MEMORY_KEY), false));
    was
}

/// Report a framing zoom on the `PDFCER_DIAG` channel and hand the outcome
/// back.
///
/// Not de-duplicated: two identical zoom commands are two events, and a gate
/// that silenced the second would make a harness unable to tell a command that
/// ran twice from one that ran once.
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

    /// ★ **The pointer wins when it is over the canvas.**
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

    /// ★ **A pointer that is not over the canvas falls back to the viewport
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

    /// ★ **An anchor armed before the zoom action is applied survives the
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

    /// ★ **A zoom that never landed drops its anchor** rather than leaving it
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

    /// ★ **A framing anchor really does land its point at the centre of the
    /// viewport**, at the scale that was granted.
    ///
    /// Asserted as the outcome — where the anchored point ends up on screen —
    /// rather than as an offset, so it checks the framing rather than the code
    /// agreeing with itself.
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

    /// ★★★ **A region at the page's very corner CAN now be centred** — and
    /// this test is the record of that changing.
    ///
    /// It used to assert the opposite: that framing a region hard against the
    /// page's top-left saturated at offset zero, *"there is no page to the
    /// left of or above the origin to scroll to"*, and the operator simply saw
    /// it off-centre. That was true when the scroll content was the page and
    /// nothing else.
    ///
    /// **O23 made it false on purpose.** The operator asked for exactly this:
    ///
    /// > *"I should also be able to move the view of the corner of the page to
    /// > the center of the screen, or even all the way vertically to the
    /// > opposite corner if I want to."*
    ///
    /// The pasteboard is a viewport of slack on every side, so there IS
    /// somewhere above and to the left to scroll to, and the solve's negative
    /// page-local offset is a legitimate position rather than an over-range
    /// one to be truncated.
    ///
    /// ★ It stayed asserting saturation for a while after the pasteboard
    /// landed, because `geometry::zoom_anchor_offset` was still clamping to
    /// the page's own range — which is `OPERATOR_REQUESTS.md` O24e, the zoom
    /// that threw away whatever the operator had panned to. A test that pins
    /// last year's constraint is how a stale clamp survives a feature
    /// designed to remove it.
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

        // ★ And the offset that actually reaches the scroll area is inside the
        // content, because `strip_offset` clamps against `content_extent` —
        // the pasteboard included. The saturation the old test protected is
        // still there; it is just further out.
        let reached = geometry::strip_offset(
            (ox, oy),
            (0.0, 0.0),
            display_after,
            display_after,
            f.viewport,
        );
        let range = (
            geometry::content_extent(display_after.0, f.viewport.0) - f.viewport.0,
            geometry::content_extent(display_after.1, f.viewport.1) - f.viewport.1,
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

    /// ★ **The ceiling is reported, not hidden.** A region small enough to ask
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

    /// ★ **Every discrete zoom is recognised, and the wheel is not.**
    ///
    /// The predicate [`arm_for_actions`] funnels on. A zoom action missing
    /// from it is a command that silently keeps the old top-left anchoring —
    /// which is the defect Phase 3.1 exists to close, reappearing one variant
    /// at a time.
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

    /// ★ **The bounds a zoom-to-selection needs come from the selection layer,
    /// and an empty selection has none** — the input side of the decline,
    /// asserted where it can be asserted without a document.
    ///
    /// The wiring above turns this `None` into [`ZoomOutcome::NoBounds`] and
    /// raises no action; what is pinned here is that the `None` is real, i.e.
    /// that the decline is reachable rather than a branch nothing can enter.
    #[test]
    fn an_empty_selection_offers_no_bounds_to_frame() {
        use crate::canvas::selection::SelectionState;
        assert!(SelectionState::default().outline_union().is_none());
    }

    /// A region far smaller than the page asks for more magnification than the
    /// raster ceiling allows, and the plan carries **both** numbers: the
    /// requested scale and the one that will be pinned.
    ///
    /// The second is what reaches `Action::ZoomTo`, so the status bar's
    /// readout states the truth on the same frame — see
    /// [`ZoomOutcome::ceiling_changed_the_answer`] on why that *is* the
    /// ceiling's report rather than a substitute for one.
    #[test]
    fn a_plan_past_the_ceiling_carries_the_clamped_scale_as_well_as_the_asked_one() {
        let f = frame(1.0);
        let plan = plan_framing(
            &f,
            Rect::from_min_max(Pos2::new(50.0, 50.0), Pos2::new(51.0, 51.0)),
            16.0,
            1.0,
            // ★ A LOW maximum, deliberately. This test is about the plan
            // reporting a clamp, so it needs a ceiling that actually clamps —
            // and since 2026-08-22 the shipped default is the highest
            // available, which clamps almost nothing. Passing the default here
            // made the test assert that an unclamped answer was clamped.
            viewer::MAX_ZOOM * 100.0,
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
            crate::app::prefs::DEFAULT_MAX_ZOOM_PERCENT,
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
