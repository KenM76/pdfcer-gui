//! Tests for `crate::app::status::fitting`, kept in the gui because they reach gui modules.

use crate::app::status::fitting::*;

/// A cluster whose five groups measure the widths the defect was found at.
fn measured() -> Widths {
    let mut w = Widths::default();
    w.record(Group::Page, 145.7);
    w.record(Group::Zoom, 108.7);
    w.record(Group::Fit, 298.1);
    w.record(Group::Find, 38.8);
    w.record(Group::Filter, 51.3);
    w
}

/// **The defect, as an assertion.**
#[test]
fn at_the_scale_that_found_the_defect_the_cluster_sheds_rather_than_overflows() {
    let widths = measured();
    let shown = affordable(611.0, &widths);
    assert!(
        !shown.contains(&Group::Fit),
        "the fit group is 298 pt of a 611 pt bar and must be the first to go"
    );
    assert!(
        shown.contains(&Group::Find),
        "dropping the fit group is enough on its own; Find should not also go"
    );
    assert!(
        measured_width(&shown, &widths) <= 611.0,
        "what is shown must fit: {} pt of 611, showing {shown:?}",
        measured_width(&shown, &widths)
    );
}

/// A wide bar shows everything, which is the property that stops this being
/// a regression at the size every operator actually uses.
#[test]
fn a_wide_bar_sheds_nothing() {
    assert_eq!(affordable(2000.0, &measured()), Group::ORDER.to_vec());
}

/// **Relative order is preserved under every subset.**
#[test]
fn every_width_keeps_the_groups_in_order() {
    let widths = measured();
    let mut width = 0.0_f32;
    while width < 1200.0 {
        let shown = affordable(width, &widths);
        let expected: Vec<Group> = Group::ORDER
            .into_iter()
            .filter(|g| shown.contains(g))
            .collect();
        assert_eq!(
            shown, expected,
            "at {width} pt the bar showed {shown:?}, which is out of order"
        );
        width += 3.0;
    }
}

/// **The two groups with no other home are never shed, at any width.**
#[test]
fn a_group_with_no_other_home_survives_every_width() {
    let widths = measured();
    let mut width = 0.0_f32;
    while width < 1200.0 {
        let shown = affordable(width, &widths);
        for group in [Group::Page, Group::Zoom, Group::Filter] {
            assert!(
                shown.contains(&group),
                "{} was shed at {width} pt and has nowhere else to be reached",
                group.region()
            );
        }
        width += 3.0;
    }
}

/// **Narrowing never puts a control back.**
#[test]
fn a_narrower_bar_never_shows_more() {
    let widths = measured();
    let mut previous = usize::MAX;
    let mut width = 1200.0_f32;
    while width > 0.0 {
        let count = affordable(width, &widths).len();
        assert!(
            count <= previous,
            "narrowing to {width} pt showed {count} groups, up from {previous}"
        );
        previous = count;
        width -= 3.0;
    }
}

/// The page box survives any width, including absurd ones.
#[test]
fn the_page_number_is_never_shed() {
    for width in [0.0_f32, 1.0, 50.0, 145.6, f32::NAN, f32::NEG_INFINITY] {
        assert!(
            affordable(width, &measured()).contains(&Group::Page),
            "the page number went missing at {width} pt"
        );
    }
}

/// An unmeasured bar shows everything, so a group can acquire a width.
#[test]
fn nothing_is_shed_before_it_has_ever_been_measured() {
    assert_eq!(
        affordable(10.0, &Widths::default()),
        Group::ORDER.to_vec(),
        "a bar with no measurements must show everything, or it can never take one"
    );
}

/// A poisoned measurement cannot empty the bar.
#[test]
fn a_degenerate_measurement_is_refused_rather_than_stored() {
    let mut w = Widths::default();
    w.record(Group::Fit, f32::NAN);
    w.record(Group::Find, -5.0);
    assert_eq!(w.get(Group::Fit), None, "a NaN width must not be stored");
    assert_eq!(
        w.get(Group::Find),
        None,
        "a negative width must not be stored"
    );
}

/// **Nothing this module may shed loses its last route.**
#[test]
fn nothing_sheddable_loses_its_last_route() {
    // The real registry, built exactly as start-up builds it. Not a list
    // of ids restated here: a restatement is what goes stale.
    let mut registry = egui_shell::commands::CommandRegistry::default();
    crate::shell::commands::register(&mut registry);
    let ids: std::collections::HashSet<&str> = registry.ids().collect();
    for (group, command) in SHED_ORDER {
        assert!(
            ids.contains(command),
            "`{}` may be shed from the status bar when the window is narrow, and its stated \
             remaining route `{command}` is not a registered command. Either the command was \
             renamed — fix the id here — or the group's only other home was deleted, in which \
             case it must be removed from SHEDDABLE and the bar must stop dropping it.",
            group.region()
        );
    }
    assert!(
        !SHED_ORDER.iter().any(|(g, _)| *g == Group::Page),
        "the page box has no other home and must never be sheddable"
    );
}

/// The left half leaves room for exactly the groups that are never shed, and
/// nothing before any has been measured.
#[test]
fn the_left_half_leaves_room_for_the_groups_that_are_never_shed() {
    let kept = measured_width(&[Group::Page, Group::Zoom, Group::Filter], &measured());
    assert!((floor_width(&measured()) - kept - 6.0).abs() < 1e-3);
    assert!(floor_width(&Widths::default()).abs() < f32::EPSILON);
}
