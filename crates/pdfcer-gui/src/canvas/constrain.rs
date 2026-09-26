//! # `canvas::constrain` — what Shift does to a drag, written down once
//!
//! ## Why this module exists at all
//!
//! `ui-conventions/drag-moves.md` D5: modifiers constrain, and the constraint
//! is announced. Shift-preserves-aspect is not an advanced feature, it is *the*
//! resize convention — PowerPoint, Illustrator, Inkscape, Figma, Visio and
//! AutoCAD all answer Shift the same way. An operator who holds Shift and gets
//! a free-form resize does not conclude that pdfcer chose differently. They
//! conclude it is broken, and they are close enough to right.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/constrain.md`.
//!
//! ## conventions: drag-moves
//!
//! This module *is* an answer to D5 rather than a surface of its own, so the
//! rows below are answered as they apply to the constraint itself.
//!
//! - D1 live-preview: the constraint is a pure filter on the live delta, so the
//!   ghost the caller already draws is the constrained one from the first
//!   frame Shift is down.
//! - D2 derived-from-commit: there is one filtered value per frame and both the
//!   preview and the commit read it — the callers apply [`axis`]/[`aspect`]
//!   *before* the split, never twice.
//! - D3 escape-cancels: nothing here holds state, so there is nothing to
//!   abandon; the gesture machine's cancel is unaffected.
//! - D4 one-undo-entry: WAIVED — this changes the value a drag commits, never
//!   how many commits it makes.
//! - D5 modifiers-constrain: this module.
//! - D6 snapping: WAIVED — Shift and snapping are independent; the caller
//!   applies snapping to the constrained delta, which is the order every
//!   program uses (constrain the intent, then land it on a target).
//! - D7 no-op-is-not-an-edit: WAIVED — a constraint can only reduce travel, and
//!   the zero-travel guard belongs to each drag's own commit path.
//! - D8 grab-point: [`toward`] exists precisely so an absolute-position drag
//!   keeps its grab point — it filters the *displacement from the press*, never
//!   the pointer's raw position.
//! - D9 disclosure: [`caption`] on the status row, off-canvas, while the
//!   constraint is live.

use egui::{Pos2, Vec2};

/// Which of the two page axes a constrained translation is locked to.
///
/// Named for what the operator sees on screen, not for a vector component:
/// `Horizontal` means the object slides left and right. The canvas is Y-down
/// and PDF user space is Y-up, and a name like `X` would invite a reader to
/// guess which of those this is about. It is about neither — a lock is a lock
/// in every space, which is why this type can be computed in canvas space and
/// remain true after `canvas::mapping` has done its one conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// Left and right; vertical travel is discarded.
    Horizontal,
    /// Up and down; horizontal travel is discarded.
    Vertical,
}

/// What a live constraint is doing, for the announcement.
///
/// Two variants rather than a `bool`, because the sentence differs and because
/// a future third constraint (centre-scaling on Alt, a 45° lock) is a variant
/// here and a compile error in [`crate::text::constrain`] — which is the same
/// exhaustive-match discipline `canvas::resizing::Refusal` uses to guarantee
/// every refusal has words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lock {
    /// A translation pinned to one axis.
    Axis(Axis),
    /// A resize keeping the object's proportions.
    Aspect,
    /// A rotation snapping to fixed steps.
    ///
    /// The third flavour of the same convention, and the one that is neither an
    /// axis nor a ratio: `drag-moves` D5 says *"Shift constrains to an axis or
    /// preserves aspect"*, and for a rotation every program in the class means
    /// **steps of 15°**. `canvas::rotating::STEP_DEGREES` owns the number and
    /// the argument for it.
    Angle,
}

/// The axis a delta has travelled furthest along.
///
/// Ties — a delta at exactly 45° — resolve to [`Axis::Horizontal`]. Arbitrary,
/// deterministic, and documented so it is a decision: the alternative is a lock
/// that flickers between axes while the pointer sits on the diagonal, which is
/// visibly worse than picking one.
#[must_use]
pub fn dominant(delta: Vec2) -> Axis {
    if delta.x.abs() >= delta.y.abs() {
        Axis::Horizontal
    } else {
        Axis::Vertical
    }
}

/// **Lock a displacement to one axis.**
///
/// The other component is set to exactly zero — not damped, not scaled. A
/// constrained drag that still creeps by a pixel on the locked axis is a
/// constraint the operator cannot trust, and trust is the entire point of
/// holding the key.
#[must_use]
pub fn axis(delta: Vec2) -> Vec2 {
    match dominant(delta) {
        Axis::Horizontal => Vec2::new(delta.x, 0.0),
        Axis::Vertical => Vec2::new(0.0, delta.y),
    }
}

/// **The absolute form of [`axis`]: where the pointer would be if it had stayed
/// on one axis from the press.**
///
/// For the drags whose outcome carries a *position* rather than a displacement
/// — a Bézier handle, a perimeter vertex — because those verbs take the point
/// the thing is going to, not how far it moved.
///
/// It filters `at - from`, never `at`. That is [`drag-moves` D8] restated:
/// the grab point survives because the constraint acts on the displacement, so
/// a handle grabbed three pixels off its centre stays three pixels off its
/// centre for the whole locked drag instead of snapping onto the axis line.
///
/// `from` is whatever the caller decides the drag is measured from, and the two
/// callers decide differently on purpose: a perimeter vertex measures from the
/// **press**, because it is a translation and the grab must be preserved; a
/// Bézier handle measures from its **anchor**, because a control point's
/// meaning is its direction and distance from the on-curve point it belongs to,
/// and locking it to the anchor's horizontal is what makes a smooth join.
///
/// [`drag-moves` D8]: https://example.invalid/ "ui-conventions/drag-moves.md D8"
#[must_use]
pub fn toward(from: Pos2, at: Pos2) -> Pos2 {
    from + axis(at - from)
}

/// **Lock a pair of scale factors to a common ratio.**
///
/// Keeps the factor further from unity and applies it to both axes. See the
/// module header for why that metric *is* relative pointer travel, and for why
/// the mid-edge grips need no special case.
///
/// Non-finite input is returned untouched: judging which of two `NaN`s is
/// "further from unity" is meaningless, and [`crate::canvas::resizing::is_usable`]
/// is the one place that decides a degenerate resize is refused. Two places
/// deciding that would be two chances to disagree.
#[must_use]
pub fn aspect(sx: f32, sy: f32) -> (f32, f32) {
    if !sx.is_finite() || !sy.is_finite() {
        return (sx, sy);
    }
    let s = if (sx - 1.0).abs() >= (sy - 1.0).abs() {
        sx
    } else {
        sy
    };
    (s, s)
}

// ===========================================================================
// The four call sites, as three functions
// ===========================================================================
//
// APPLY AND ANNOUNCE ARE ONE CALL, and that is the point of this section.
//
// A caller cannot reach the arithmetic without the announcement, or the
// announcement without the arithmetic, because a drag that constrains silently
// is not a smaller version of this feature — it is `drag-moves` D5's stated
// failure mode: the operator holds Shift, gets a result they did not expect,
// and cannot tell whether the modifier did anything.
//
// Letting each drag pair the two for itself would also spell "is Shift down" at
// every call site, which is the predicate-with-two-claimants failure the module
// header forbids.
//
// Each function takes `active` rather than reading the modifier itself, because
// *which* modifier constrains is the canvas's decision and this module's job is
// what it does.

/// **A constrained translation**: the delta to move by, announced.
///
/// For the drags whose outcome is a displacement — the object move and the
/// ce-dimension label placement, which share one gesture.
#[must_use]
pub fn translate(ctx: &egui::Context, active: bool, delta: Vec2) -> Vec2 {
    if !active {
        return delta;
    }
    announce(ctx, Lock::Axis(dominant(delta)));
    axis(delta)
}

/// **A constrained absolute move**: where the thing goes, announced.
///
/// For the drags whose outcome is a position — a perimeter vertex, measured
/// from the press, and a Bézier handle, measured from its anchor. See
/// [`toward`] for why the two reference points differ and why filtering the
/// displacement rather than the position is what preserves the grab.
#[must_use]
pub fn reposition(ctx: &egui::Context, active: bool, from: Pos2, at: Pos2) -> Pos2 {
    if !active {
        return at;
    }
    announce(ctx, Lock::Axis(dominant(at - from)));
    toward(from, at)
}

/// **A constrained resize**: announced here, applied inside the drag.
///
/// The odd one out, and deliberately so. A resize's factors are derived inside
/// [`crate::canvas::resizing::drag`] from a grip and a box, and the ghost it
/// returns must be the same pair it commits — so the lock has to be applied
/// *there*, between the derivation and the branch. What the caller can do is
/// announce, and hand the flag on.
///
/// Returns `active` unchanged so the call reads as one expression at the call
/// site, which is what stops a caller announcing one thing and passing another.
#[must_use]
pub fn resize(ctx: &egui::Context, active: bool) -> bool {
    if active {
        announce(ctx, Lock::Aspect);
    }
    active
}

// ===========================================================================
// The announcement
// ===========================================================================

/// The `egui::Memory` slot the announcement travels in.
fn slot() -> egui::Id {
    egui::Id::new("pdfcer.canvas.constraint") // ui-text-exempt: memory key, never displayed.
}

/// **Record that a constraint is active this frame**, for the status row to
/// read on the next one.
///
/// Called by the drag that applied the constraint, not by the modifier check —
/// so a Shift held over an *unconstrainable* drag (a marquee, where Shift means
/// "extend the selection") announces nothing, which is correct: nothing was
/// constrained.
pub fn announce(ctx: &egui::Context, lock: Lock) {
    let frame = ctx.cumulative_pass_nr();
    // Traced ONCE per lock change, not once per frame.
    //
    // A drag runs at 60 Hz and the announcement is re-made every frame of it.
    // Tracing unconditionally would put sixty identical lines a second on the
    // channel and bury every other event — the same reason `canvas::moving::drag`
    // traces its refusal only on release.
    //
    // The previous value is already in memory, so "has it changed" costs the
    // read this function was going to do anyway. What survives on the channel
    // is one line per *transition*: the frame Shift went down, and the frame
    // the locked axis flipped — which are exactly the two events a driven check
    // asks about.
    let previous: Option<(u64, Lock)> = ctx.data(|d| d.get_temp(slot()));
    let changed = match previous {
        // A stamp older than the previous frame means the last constraint has
        // already retired, so this is a new one even if it locks the same way.
        Some((then, was)) => was != lock || frame.saturating_sub(then) > 1,
        None => true,
    };
    if changed {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("constrain lock={lock:?}")
        });
    }
    ctx.data_mut(|d| d.insert_temp(slot(), (frame, lock)));
}

/// **The sentence for a constraint that is live right now**, or `None`.
///
/// Self-retiring: the stamp must be this frame's or the one before it. One
/// frame of slack because the status bar is composed before the canvas that
/// writes the slot, so on any given frame the freshest value available *is* the
/// previous frame's. Two frames would leave the sentence on screen after the
/// key came up; zero would show it never.
#[must_use]
pub fn caption(ctx: &egui::Context) -> Option<&'static str> {
    let (frame, lock): (u64, Lock) = ctx.data(|d| d.get_temp(slot()))?;
    let now = ctx.cumulative_pass_nr();
    if now.saturating_sub(frame) > 1 {
        return None;
    }
    Some(crate::text::constrain::caption(lock))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A mostly-horizontal drag keeps its x and loses its y entirely.**
    ///
    /// The base case, asserted as *exactly* zero rather than "small", because a
    /// residual is the difference between a constraint and a suggestion.
    #[test]
    fn a_mostly_horizontal_drag_locks_to_the_horizontal() {
        let locked = axis(Vec2::new(80.0, 12.0));
        assert!((locked.x - 80.0).abs() < f32::EPSILON);
        assert_eq!(locked.y, 0.0, "the off-axis component must be exactly zero");
    }

    /// The same in the other axis, and with negative travel — dragging up and
    /// slightly right is a vertical drag.
    #[test]
    fn a_mostly_vertical_drag_locks_to_the_vertical() {
        let locked = axis(Vec2::new(5.0, -90.0));
        assert_eq!(locked.x, 0.0);
        assert!((locked.y + 90.0).abs() < f32::EPSILON);
    }

    /// **The lock follows the pointer; it is not sampled at the press.**
    #[test]
    fn the_locked_axis_is_re_decided_from_the_live_delta() {
        assert_eq!(dominant(Vec2::new(30.0, 4.0)), Axis::Horizontal);
        assert_eq!(dominant(Vec2::new(30.0, 400.0)), Axis::Vertical);
    }

    /// A delta at exactly 45° resolves one way and stays there.
    #[test]
    fn the_diagonal_does_not_flicker() {
        assert_eq!(dominant(Vec2::new(10.0, 10.0)), Axis::Horizontal);
        assert_eq!(dominant(Vec2::new(-10.0, 10.0)), Axis::Horizontal);
    }

    /// **The grab point survives an absolute-position lock.**
    #[test]
    fn an_absolute_lock_keeps_the_offset_it_started_with() {
        let from = Pos2::new(100.0, 100.0);
        let at = Pos2::new(180.0, 107.0);
        let locked = toward(from, at);
        assert!((locked.x - 180.0).abs() < f32::EPSILON);
        assert!(
            (locked.y - 100.0).abs() < f32::EPSILON,
            "the locked axis returns to the PRESS row, not to the pointer's"
        );
    }

    /// **Aspect keeps the factor the pointer worked hardest for.**
    ///
    /// Growing 1.5× on x while barely moving y must give 1.5× on both — not the
    /// average, not the smaller, and not x-because-x-is-first.
    #[test]
    fn aspect_keeps_the_dominant_factor() {
        assert_eq!(aspect(1.5, 1.02), (1.5, 1.5));
        assert_eq!(aspect(1.02, 0.4), (0.4, 0.4));
    }

    /// **A mid-edge grip becomes a proportional resize, with no special
    /// case.**
    #[test]
    fn a_mid_edge_grip_scales_both_axes_under_shift() {
        assert_eq!(aspect(1.5, 1.0), (1.5, 1.5));
        assert_eq!(aspect(1.0, 0.75), (0.75, 0.75));
    }

    /// Shrinking is symmetric with growing: 0.5 is as far from unity as 1.5.
    #[test]
    fn shrinking_and_growing_are_measured_the_same_way() {
        assert_eq!(aspect(0.5, 0.9), (0.5, 0.5));
    }

    /// A degenerate pair passes straight through, so exactly one place decides
    /// a resize is unusable.
    #[test]
    fn a_degenerate_pair_is_left_for_the_one_place_that_refuses_it() {
        let (sx, sy) = aspect(f32::NAN, 2.0);
        assert!(sx.is_nan());
        assert!((sy - 2.0).abs() < f32::EPSILON);
        assert!(!crate::canvas::resizing::is_usable(sx, sy));
    }
}
