//! Driven tests for **carrying a float window back over the dock** — where an
//! offer is made, where it is refused, and what a release produces.
//!
//! # These drive `egui`, and that is the whole point
//!
//! [`super::DockLayout::move_panel`] has unit tests of its own and every one of
//! them would pass over a build where the offer never drew, the release was
//! never read, or the intent was raised and never applied. A verb's unit tests
//! cannot see the chain in front of it.
//!
//! # ★ The pointer here is NOT `egui`'s
//!
//! Every other gesture file in this crate presses and drags through
//! [`super::drive`]'s event pump. This one sends **no pointer events at all**
//! and reports the drag through [`super::DockState::set_float_drag`], because
//! that is exactly how the gesture arrives in a real application: the pointer
//! is over the float's own window, and the point handed in is the one the
//! caller converted into this window's screen points.
//!
//! That makes [`a_float_carried_over_a_compartment_is_offered_a_drop`] the
//! positive control this file owes: with no pointer events, an absence
//! asserted below is otherwise satisfied by a build that never read the report.

use egui::{Pos2, Vec2};

use super::drive::{Harness, drag_to, press};
use super::floatdrag::FloatDrag;
use super::geometry::StackAddr;
use super::{Column, DockLayout, DockSide, DropZone, PanelId, SideLayout, Stack, report};

/// Two columns on the left and one on the right — the shape
/// [`super::overlay_tests`] and [`super::tear_tests`] use, so a coordinate
/// means the same thing in all three.
fn two_columns() -> DockLayout {
    DockLayout::new(
        SideLayout::new([
            Column::new([Stack::tabbed(["pages", "bookmarks"])]),
            Column::new([Stack::tabbed(["layers"])]),
        ])
        .with_width(420.0),
        SideLayout::new([Column::new([Stack::tabbed(["properties"])])]).with_width(300.0),
    )
}

/// `["pages", "bookmarks"]`.
const LEFT0: StackAddr = StackAddr {
    side: DockSide::Left,
    column: 0,
    stack: 0,
};

/// `["properties"]`.
const RIGHT: StackAddr = StackAddr {
    side: DockSide::Right,
    column: 0,
    stack: 0,
};

/// A point in the document, clear of both sides in a 1400 × 900 window.
const CANVAS: Pos2 = Pos2::new(760.0, 600.0);

fn id(s: &str) -> PanelId {
    PanelId::new(s)
}

/// The middle of a compartment's body.
fn body_centre(h: &Harness, at: StackAddr) -> Pos2 {
    super::compass::body_of(h.state.geometry(), at)
        .expect("the compartment was drawn")
        .center()
}

/// The tabs of one compartment, as strings.
fn tabs(h: &Harness, at: StackAddr) -> Vec<String> {
    h.state.layout().side(at.side).columns[at.column].stacks[at.stack]
        .tabs
        .iter()
        .map(|p| p.as_str().to_owned())
        .collect()
}

/// A harness with `layers` already in a window of its own, warmed so the
/// remaining compartments have current rects.
fn with_layers_floating() -> Harness {
    let mut h = Harness::new(two_columns());
    assert!(h.state.layout_mut().float(&id("layers")), "it floated");
    h.warm();
    h
}

/// Report a drag of `panel` at `pointer` and run one frame.
fn carry(h: &mut Harness, panel: &str, pointer: Pos2, released: bool) {
    h.state.set_float_drag(Some(FloatDrag {
        panel: id(panel),
        pointer,
        released,
    }));
    h.frame(Vec::new());
}

/// **The control.** A float carried over a compartment is offered a drop there.
///
/// Two facts at once: the report reaches the resolution at all, and the
/// compartment it names is the one under the point that was handed in — not the
/// one the panel was torn from, which is what a build that skipped the
/// resolution and reused [`super::DockHome`] would produce.
#[test]
fn a_float_carried_over_a_compartment_is_offered_a_drop() {
    let mut h = with_layers_floating();
    let over = body_centre(&h, RIGHT);

    carry(&mut h, "layers", over, false);

    let preview = h.report.drop_preview.clone().expect("a drop is offered");
    assert_eq!(preview.panel, id("layers"), "the panel being carried");
    assert_eq!(preview.landing.over, RIGHT, "the compartment aimed at");
    assert!(preview.lands, "and the release would change something");
    assert!(
        h.rect(&report::drop_outcome()).is_some(),
        "the outcome outline is drawn, not merely computed"
    );
}

/// **The same compass, not a second one — and the point resolved is the point
/// handed in.**
///
/// Two claims, and the second is the one with a whole class of defect behind
/// it. [`super::FloatDrag::pointer`] is in the application window's own screen
/// points, and a caller that converted from desktop points wrongly, or a shell
/// that nudged what it was given, would still resolve *somewhere* and still
/// draw a plausible compass. So it is not enough to assert the offer equals the
/// grammar's answer for the point aimed at: the test also has to pick a point
/// whose answer differs from its neighbours', or a shifted pointer satisfies it
/// unchanged.
///
/// The left edge of a body is such a point. It is a [`DropZone::Left`] — a new
/// column beside the compartment — where the middle of the same body is a
/// [`DropZone::Centre`], a tab appended to it. Two different outcomes from one
/// rectangle, which is exactly what the five zones are for.
///
/// ⚠ **What this still cannot see, and what therefore has to be driven.** The
/// smallest pointer error that changes the answer is the width of the edge band
/// — a quarter of the body, capped at `compass::EDGE_MAX_PTS`. A conversion
/// from desktop points that is wrong by less than that lands in the same zone
/// and passes every test in this file. The conversion itself is the
/// application's, and it is verified by driving the binary, not here.
#[test]
fn the_zone_aimed_at_decides_the_target() {
    let mut h = with_layers_floating();
    let body = super::compass::body_of(h.state.geometry(), RIGHT).expect("drawn");
    let left_edge = Pos2::new(body.left() + 6.0, body.center().y);

    carry(&mut h, "layers", left_edge, false);

    let preview = h.report.drop_preview.clone().expect("a drop is offered");
    assert_eq!(
        preview.landing,
        h.state
            .layout()
            .resolve_drop(h.state.geometry(), left_edge)
            .expect("the grammar resolves it"),
        "the offer is the grammar's own answer, not a second reading of it"
    );
    assert_eq!(
        preview.landing.zone,
        Some(DropZone::Left),
        "the edge zone, not the body-wide one"
    );
    assert_ne!(
        preview.landing.target,
        h.state
            .layout()
            .resolve_drop(h.state.geometry(), body.center())
            .expect("the grammar resolves the middle too")
            .target,
        "a different outcome from the middle of the same compartment, so a shifted pointer could not satisfy this"
    );
}

/// **A release over a compartment docks it there, and it stops floating.**
#[test]
fn a_release_over_a_compartment_docks_it_there() {
    let mut h = with_layers_floating();
    let over = body_centre(&h, LEFT0);

    carry(&mut h, "layers", over, false);
    assert!(
        h.report.drop_preview.is_some(),
        "the witness: it was offered"
    );

    carry(&mut h, "layers", over, true);
    assert_eq!(h.report.moved, Some(id("layers")), "it docked");
    assert!(
        !h.state.layout().is_floating(&id("layers")),
        "and nothing is floating"
    );
    assert!(
        tabs(&h, LEFT0).contains(&"layers".to_owned()),
        "it landed in the compartment it was aimed at: {:?}",
        tabs(&h, LEFT0)
    );
    assert_eq!(
        h.report.drop_preview, None,
        "and the offer is gone on the frame it lands"
    );
}

/// **A release clear of the dock docks nothing.**
///
/// Dragging a window around the desktop is not a dock gesture, and a build that
/// treated every release as one would swallow the window the moment the
/// operator moved it anywhere.
#[test]
fn a_release_clear_of_the_dock_leaves_it_floating() {
    let mut h = with_layers_floating();

    carry(&mut h, "layers", CANVAS, false);
    assert_eq!(h.report.drop_preview, None, "nothing is offered out there");

    carry(&mut h, "layers", CANVAS, true);
    assert_eq!(h.report.moved, None, "and nothing docked");
    assert!(
        h.state.layout().is_floating(&id("layers")),
        "it is still in its window"
    );
}

/// **An offer lasts exactly as long as the caller renews it.**
///
/// The report is consumed, so a frame the caller did not answer for is a frame
/// with no gesture — which is what makes a window closed mid-drag land nothing
/// rather than leave an offer standing that nothing can dismiss.
#[test]
fn an_offer_not_renewed_is_over() {
    let mut h = with_layers_floating();
    let over = body_centre(&h, RIGHT);

    carry(&mut h, "layers", over, false);
    assert!(
        h.report.drop_preview.is_some(),
        "the witness: it was offered"
    );

    h.warm();
    assert_eq!(h.report.drop_preview, None, "no later frame resurrects it");
    assert_eq!(h.rect(&report::drop_outcome()), None, "nothing drawn");
    assert!(
        h.state.layout().is_floating(&id("layers")),
        "and nothing landed"
    );
}

/// **A panel that is not floating is declined, not asserted on.**
///
/// A caller reads its pointer a frame behind the layout it reports against, so
/// a drag whose window has just been closed or docked by some other route is an
/// ordinary race. Offering a drop for a docked panel would move it on release.
#[test]
fn a_drag_of_a_panel_that_is_not_floating_is_declined() {
    let mut h = with_layers_floating();
    let over = body_centre(&h, RIGHT);

    carry(&mut h, "pages", over, true);

    assert_eq!(h.report.drop_preview, None, "no offer for a docked panel");
    assert_eq!(h.report.moved, None, "and the release moved nothing");
    assert_eq!(tabs(&h, LEFT0), ["pages", "bookmarks"], "nothing permuted");
}

/// **The gesture the shell sensed for itself wins.**
///
/// The one stand-down in the dock that input can actually reach: the other
/// three affordances read one `egui` pointer and exclude each other by
/// geometry, while this one reads a point a caller supplies and nothing stops
/// the two arriving on the same frame. The tab drag keeps its own caret, and
/// the float's release is not answered by the compartment the tab is over.
#[test]
fn a_tab_drag_in_flight_beats_a_reported_float_drag() {
    let mut h = with_layers_floating();
    let from = h.tab_centre(LEFT0, 0);
    let along = h.tab_centre(LEFT0, 1) + Vec2::new(8.0, 0.0);

    h.frame(press(from));
    h.state.set_float_drag(Some(FloatDrag {
        panel: id("layers"),
        pointer: body_centre(&h, RIGHT),
        released: true,
    }));
    h.frame(drag_to(along));

    assert!(h.report.tab_drag.is_some(), "the strip owns its own drag");
    assert_eq!(h.report.drop_preview, None, "the float offered nothing");
    assert!(
        h.state.layout().is_floating(&id("layers")),
        "and its release landed nothing"
    );
}
