use super::*;

const EPS: f32 = 1e-3;

fn near(a: Rect, b: Rect) -> bool {
    (a.min - b.min).length() < EPS && (a.max - b.max).length() < EPS
}

#[test]
fn a_wide_mark_fills_the_width_and_is_centred() {
    let target = Rect::from_min_size(pos2(0.0, 0.0), vec2(200.0, 40.0));
    let r = fit_rect(vec2(10.0, 1.0), target).expect("fits");
    assert!((r.width() - 190.0).abs() < EPS);
    assert!((r.min.x - 6.0).abs() < EPS);
    assert!((r.center().y - 20.0).abs() < EPS);
}

#[test]
fn a_tall_mark_stands_on_the_lower_edge_and_rises() {
    let target = Rect::from_min_size(pos2(0.0, 100.0), vec2(200.0, 20.0));
    let r = fit_rect(vec2(1.0, 1.0), target).expect("fits");
    assert!((r.height() - 38.0).abs() < EPS);
    assert!((r.max.y - 119.0).abs() < EPS);
    assert!(r.min.y < target.min.y);
}

#[test]
fn nothing_fits_without_extent_or_a_box() {
    let target = Rect::from_min_size(pos2(0.0, 0.0), vec2(200.0, 40.0));
    assert!(fit_rect(Vec2::ZERO, target).is_none());
    assert!(fit_rect(vec2(1.0, 1.0), Rect::NOTHING).is_none());
}

#[test]
fn the_allowed_region_is_the_box_and_one_height_above() {
    let target = Rect::from_min_size(pos2(10.0, 100.0), vec2(200.0, 30.0));
    assert!(near(
        allowed(target),
        Rect::from_min_max(pos2(10.0, 70.0), pos2(210.0, 130.0))
    ));
}

#[test]
fn clamping_moves_a_stray_rect_back_inside() {
    let target = Rect::from_min_size(pos2(0.0, 100.0), vec2(200.0, 30.0));
    let stray = Rect::from_min_size(pos2(190.0, 125.0), vec2(40.0, 20.0));
    assert!(near(
        clamp(stray, target),
        Rect::from_min_size(pos2(160.0, 110.0), vec2(40.0, 20.0))
    ));
}

#[test]
fn clamping_shrinks_an_oversize_rect_keeping_its_shape() {
    let target = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 10.0));
    let big = Rect::from_min_size(pos2(0.0, -10.0), vec2(400.0, 20.0));
    let c = clamp(big, target);
    assert!((c.width() / c.height() - 20.0).abs() < 1e-2);
    assert!(allowed(target).expand(EPS).contains_rect(c));
}

#[test]
fn a_corner_keeps_proportions_and_the_opposite_corner() {
    let r = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 20.0));
    let out = reshape(Grip::SouthEast, r, vec2(100.0, 0.0), false, true);
    assert!(near(
        out,
        Rect::from_min_size(pos2(0.0, 0.0), vec2(200.0, 40.0))
    ));
}

#[test]
fn a_free_corner_stretches() {
    let r = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 20.0));
    let out = reshape(Grip::NorthWest, r, vec2(-10.0, 0.0), true, true);
    assert!(near(
        out,
        Rect::from_min_max(pos2(-10.0, 0.0), pos2(100.0, 20.0))
    ));
}

#[test]
fn a_free_corner_on_a_shape_that_cannot_stretch_keeps_proportions() {
    let r = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 20.0));
    let out = reshape(Grip::SouthEast, r, vec2(100.0, 0.0), true, false);
    assert!((out.width() / out.height() - 5.0).abs() < EPS);
}

#[test]
fn an_edge_stretches_one_side_or_scales_both() {
    let r = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 20.0));
    let s = reshape(Grip::East, r, vec2(50.0, 7.0), false, true);
    assert!(near(
        s,
        Rect::from_min_size(pos2(0.0, 0.0), vec2(150.0, 20.0))
    ));
    let k = reshape(Grip::East, r, vec2(100.0, 0.0), false, false);
    assert!(near(
        k,
        Rect::from_min_max(pos2(0.0, -10.0), pos2(200.0, 30.0))
    ));
}

#[test]
fn the_body_moves_and_no_side_collapses() {
    let r = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 20.0));
    assert!(near(
        reshape(Grip::Move, r, vec2(5.0, -3.0), false, true),
        r.translate(vec2(5.0, -3.0))
    ));
    let tiny = reshape(Grip::SouthEast, r, vec2(-200.0, -200.0), true, true);
    assert!(tiny.width() >= MIN_SIDE && tiny.height() >= MIN_SIDE);
}

#[test]
fn a_placement_round_trips_through_any_scale() {
    let small = Rect::from_min_size(pos2(10.0, 10.0), vec2(100.0, 20.0));
    let large = Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 80.0));
    let chosen = Rect::from_min_size(pos2(20.0, 0.0), vec2(50.0, 25.0));
    let p = Placement::of(chosen, small);
    assert!(near(p.in_box(small), chosen));
    assert!(near(
        p.in_box(large),
        Rect::from_min_size(pos2(40.0, -40.0), vec2(200.0, 100.0))
    ));
}

#[test]
fn a_chosen_placement_wins_and_is_kept_inside_the_region() {
    let target = Rect::from_min_size(pos2(0.0, 100.0), vec2(200.0, 20.0));
    let fitted = fit_rect(vec2(4.0, 1.0), target).expect("fits");
    assert_eq!(ink_rect(vec2(4.0, 1.0), target, None), Some(fitted));
    let inside = Rect::from_min_size(pos2(50.0, 95.0), vec2(40.0, 10.0));
    let chosen = Placement::of(inside, target);
    assert!(near(
        ink_rect(vec2(4.0, 1.0), target, Some(chosen)).expect("placed"),
        inside
    ));
    let outside = Placement::of(inside.translate(vec2(500.0, 0.0)), target);
    let kept = ink_rect(vec2(4.0, 1.0), target, Some(outside)).expect("placed");
    assert!((kept.max.x - 200.0).abs() < EPS, "{kept:?}");
}
