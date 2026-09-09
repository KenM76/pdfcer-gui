//! Tests for [`super`]'s **body strip** — the region of a selection that means
//! *move* rather than *resize*.
//!
//! Split out on 2026-09-08 for R2 (no source file over 1,500 lines) when
//! `Grip::name` needed room. **Nothing else moved**: the same
//! `mod body_strip_tests` block, de-indented, with its parent's private items
//! still reachable because a child module can see them.
//!
//! ⚠ `handles.rs` carries **two** test modules and only this one moved. A
//! first attempt cut at the first `#[cfg(test)]` and swept up both, leaving
//! their braces unbalanced — a seam found by counting braces is not a seam.
//! The other module stays where it is; if the file needs room again, move it
//! the same way rather than widening this one.

#![cfg(test)]

use super::*;

/// ★★★ **The centre of a short, wide selection is the BODY, not a grip.**
///
/// The measured case: a 160 × 20 pt form field at the operator's fitted
/// 29.55 % zoom is 47.3 × 5.9 px. Before this rule, dead centre answered
/// `Grip::North` — so dragging the field to move it committed a degenerate
/// resize the engine then refused, and the operator's field did not move
/// and did not say why.
///
/// ★ The numbers are the real ones from `widget-move.trace.txt` rather than
/// round ones, because the defect is a threshold and a rounded fixture can
/// sit on the comfortable side of it without anybody noticing.
#[test]
fn the_centre_of_a_short_field_is_the_body() {
    let field = Rect::from_min_size(Pos2::new(849.0, 957.3), Vec2::new(47.3, 5.9));
    assert_eq!(
        grip_at(field, field.center(), GripSet::all()),
        Some(Grip::Move),
        "the centre of a 5.9 px-tall box was inside its own North grip"
    );
}

/// ★★ …and on a short box **no grip's grab region reaches into the body at
/// all**, which is the promise that replaced "the mid-edge pair is withheld".
///
/// # Why this assertion changed on 2026-09-05, stated rather than quietly edited
///
/// The original wording asserted that North and South were *dropped* from
/// the offered list on a 5.9 px-tall field. That was the right assertion for
/// the mechanism that existed at the time — a filter — and it is the wrong
/// one for the mechanism that exists now. [`grip_bounds`] pushes the grips
/// outward instead of dropping them, so on this field North and South are
/// offered **and drawn 7.05 px clear of the box**, where they can be aimed
/// at and where they eat nothing. Withholding them would now be a
/// regression: they are the two grips that resize a short field's height,
/// which is the one thing an operator is likely to want from it.
///
/// So the promise is restated at the level it was always really about:
/// **the body belongs to the body.** That is falsifiable against both
/// mechanisms, which the old wording was not.
///
/// Asserted separately from `the_centre_of_a_short_field_is_the_body`
/// because the two are different promises: one is about where a press lands,
/// the other about what the operator is shown. A grip painted where it
/// cannot be aimed is the affordance R9 forbids, and the painter reads this
/// same list.
#[test]
fn a_short_box_keeps_its_whole_body_and_its_grips_sit_outside_it() {
    let field = Rect::from_min_size(Pos2::new(849.0, 957.3), Vec2::new(47.3, 5.9));
    let offered = grip_rects(field);

    // Six, not eight, and the arithmetic is worth writing down because the
    // number is not obvious. The field is 47.3 wide and 5.9 tall, so only
    // the vertical axis is pushed: the anchor box is 47.3 x 20. North and
    // South survive because 47.3 clears MIN_MID_GRIP_EXTENT_PX (24). East
    // and West do not, because 20 does not — they are withheld for PILING
    // onto their corner neighbours, which is a different rule from the one
    // this test is about and one the push does not and should not touch.
    let names: Vec<Grip> = offered.iter().map(|(g, _)| *g).collect();
    assert!(
        names.contains(&Grip::North) && names.contains(&Grip::South),
        "the mid-edge pair that resizes a short field's HEIGHT was withheld: {names:?}"
    );
    assert!(
        !names.contains(&Grip::East) && !names.contains(&Grip::West),
        "East/West would pile onto the corners at 20 pt of pushed height: {names:?}"
    );

    // ★ The load-bearing assertion. Every grip's GRAB region — the drawn
    // square plus its slack, which is what `grip_at` tests — must miss the
    // horizontal strip through the middle of the field. Sampled across the
    // width rather than at the centre alone, because the old defect left a
    // 1.4 x 0.5 pt hole at dead centre and a centre-only test walks straight
    // through it.
    for i in 0..=20 {
        let x = field.left() + field.width() * (i as f32 / 20.0);
        let p = Pos2::new(x, field.center().y);
        assert_eq!(
            grip_at(field, p, GripSet::all()),
            Some(Grip::Move),
            "a grip claimed the body at x offset {i}/20 of a 5.9 px-tall field"
        );
    }
}

/// ★★★ **His banana. An object 0.85 pt across can be moved.**
///
/// The report, verbatim: *"zoom in on the atoms of the banana pdf file and
/// see what happens when you try to draw a box around a molecule and move
/// it, or select the ion and move it."* Driving it produced
/// `resize-declined reason=Degenerate` on every press, because the box was
/// floored to [`crate::canvas::overlay::MIN_OUTLINE_EXTENT_PX`] = 6 pt and
/// four corner grips reaching 6 pt each covered all of it.
///
/// The fixture is the **floored** box, not the 0.85 pt one, because the
/// floor is what the operator's pointer actually meets — testing the
/// un-floored rect would test a rectangle nothing on screen corresponds to.
#[test]
fn the_smallest_object_the_shell_can_draw_is_still_grabbable() {
    let cell = Rect::from_min_size(
        Pos2::new(640.0, 480.0),
        Vec2::splat(crate::canvas::overlay::MIN_OUTLINE_EXTENT_PX),
    );
    // Every point of it, corners included — there is no part of a 6 pt box
    // an operator could be expected to aim at more carefully than another.
    for i in 0..=6 {
        for j in 0..=6 {
            let p = Pos2::new(
                cell.left() + cell.width() * (i as f32 / 6.0),
                cell.top() + cell.height() * (j as f32 / 6.0),
            );
            assert_eq!(
                grip_at(cell, p, GripSet::all()),
                Some(Grip::Move),
                "a grip claimed ({i}/6, {j}/6) of a 6 pt cell: the banana defect"
            );
        }
    }
}

/// ★ …and the grips are still *there*, outside it, so the cell can be
/// resized as well as moved.
///
/// Asserted because the cheap way to pass the test above is to stop offering
/// grips on a small box, which trades one lost capability for another.
#[test]
fn the_smallest_object_still_offers_grips_to_resize_it_by() {
    let cell = Rect::from_min_size(
        Pos2::new(640.0, 480.0),
        Vec2::splat(crate::canvas::overlay::MIN_OUTLINE_EXTENT_PX),
    );
    let offered = grip_rects(cell);
    assert!(
        offered.iter().any(|(g, _)| *g == Grip::NorthWest),
        "a small box must keep its corners: {:?}",
        offered.iter().map(|(g, _)| *g).collect::<Vec<_>>()
    );
    for (g, r) in &offered {
        assert!(
            !cell.contains(r.center()),
            "{g:?} is still anchored inside the 6 pt cell at {:?}",
            r.center()
        );
        assert!(
            grip_at(cell, r.center(), GripSet::all()) == Some(*g),
            "{g:?} is drawn where it cannot be aimed: the R9 failure"
        );
    }
}

/// ★★ **The push is exactly zero above the threshold**, which is what makes
/// applying it unconditionally safe.
///
/// If this ever fails, every comfortable selection in the product has moved
/// its grips, and nothing else in the suite would say so in those words.
#[test]
fn a_box_with_a_body_is_not_pushed_at_all() {
    for (w, h) in [
        (MIN_BODY_STRIP_PX, MIN_BODY_STRIP_PX),
        (MIN_BODY_STRIP_PX, 400.0),
        (400.0, MIN_BODY_STRIP_PX),
        (300.0, 200.0),
    ] {
        let r = Rect::from_min_size(Pos2::new(100.0, 200.0), Vec2::new(w, h));
        assert_eq!(
            grip_bounds(r),
            r,
            "a {w} x {h} box was pushed, and it did not need to be"
        );
    }
}

/// ★ The push reaches the threshold and stops there, and never runs
/// backwards as the box shrinks — so there is no zoom at which the
/// affordance jumps.
#[test]
fn the_push_reaches_the_threshold_and_stops_there() {
    let mut previous = f32::INFINITY;
    for step in 0..=40 {
        let extent = MIN_BODY_STRIP_PX * (step as f32 / 40.0);
        let r = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::splat(extent));
        let pushed = grip_bounds(r);
        assert!(
            (pushed.width() - MIN_BODY_STRIP_PX).abs() < 1e-3,
            "a {extent} pt box was grown to {} rather than to the threshold",
            pushed.width()
        );
        let push = (pushed.width() - extent) / 2.0;
        assert!(
            push <= previous + 1e-3,
            "the push went UP as the box grew, at extent {extent}"
        );
        previous = push;
    }
}

/// ★ A comfortable box is unchanged, which is what says the rule is a floor
/// and not a redesign.
#[test]
fn a_comfortable_box_still_gets_all_eight() {
    let roomy = Rect::from_min_size(Pos2::new(100.0, 200.0), Vec2::new(300.0, 200.0));
    assert_eq!(grip_rects(roomy).len(), 8);
    assert_eq!(
        grip_at(roomy, roomy.center(), GripSet::all()),
        Some(Grip::Move)
    );
}
