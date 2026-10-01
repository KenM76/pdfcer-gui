use super::*;

#[test]
fn a_long_name_in_a_short_box_stands_on_its_edge() {
    let target = Rect::from_min_max(pos2(100.0, 500.0), pos2(400.0, 520.0));
    // 8 em wide, a face 1.5 em tall: height binds at 0.95 * 2 * 20 / 1.5.
    let fit = fit_typed(8.0, 1.1, -0.4, target).expect("fits");
    assert!((fit.size - 0.95 * 2.0 * 20.0 / 1.5).abs() < 1e-3, "{fit:?}");
    let bottom = fit.origin.y + 0.4 * fit.size;
    assert!(
        (bottom - (520.0 - 0.05 * 20.0)).abs() < 1e-3,
        "stands: {bottom}"
    );
    assert!((fit.origin.x - (100.0 + 0.03 * 300.0)).abs() < 1e-3);
}

#[test]
fn a_very_long_name_is_bound_by_the_width() {
    let target = Rect::from_min_max(pos2(0.0, 0.0), pos2(200.0, 100.0));
    let fit = fit_typed(40.0, 1.0, -0.3, target).expect("fits");
    assert!((fit.size * 40.0 - 0.95 * 200.0).abs() < 1e-3, "{fit:?}");
    // Small enough to sit in 90 % of the height, so it is centred.
    let (top, bottom) = (fit.origin.y - fit.size, fit.origin.y + 0.3 * fit.size);
    assert!(((top + bottom) / 2.0 - 50.0).abs() < 1e-3);
}

#[test]
fn nothing_to_place_is_no_fit() {
    let target = Rect::from_min_max(pos2(0.0, 0.0), pos2(200.0, 20.0));
    assert!(fit_typed(0.0, 1.0, -0.3, target).is_none());
    assert!(fit_typed(5.0, 1.0, -0.3, Rect::NOTHING).is_none());
}

#[test]
fn the_remembered_form_round_trips_and_rejects_a_blank_name() {
    let typed = Typed {
        face: "Segoe Script".to_owned(),
        name: "Pat Example".to_owned(),
    };
    assert_eq!(Typed::from_text(&typed.to_text()), Some(typed));
    assert!(Typed::from_text("Segoe Script\n   \n").is_none());
    assert!(Typed::from_text("Segoe Script").is_none());
}

#[test]
fn the_advance_sums_the_plan_widths() {
    let Some(face) = faces().first() else {
        // No handwriting face on this machine: nothing to measure.
        return;
    };
    let plan = plan(face, "Al").expect("plans");
    let a = advance(&plan, "A").expect("A");
    let l = advance(&plan, "l").expect("l");
    let both = advance(&plan, "Al").expect("Al");
    assert!(a > 0.0 && l > 0.0);
    assert!((both - (a + l)).abs() < 1e-6);
    assert!(advance(&plan, "Z").is_none(), "Z is not in the subset");
}
