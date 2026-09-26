//! Tests for `crate::canvas::constrain`, kept in the gui because they reach gui modules.

use crate::canvas::constrain::*;
use egui::{Pos2, Vec2};

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
