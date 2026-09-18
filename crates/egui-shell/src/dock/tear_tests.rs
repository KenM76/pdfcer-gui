//! Driven tests for **dragging a panel out of the dock** — where a window is
//! offered, where it is not, and what a release out there produces.
//!
//! # These drive `egui`, and that is the whole point
//!
//! [`super::DockLayout::float`] and [`super::DockLayout::float_at`] have unit
//! tests of their own, and every one of them would pass over a build where the
//! outline never drew, the release was never read, or the intent was raised and
//! never applied. A verb's unit tests cannot see the chain in front of it.
//!
//! # The driver, and the two controls this file owes it
//!
//! [`super::drive`]. Its header carries the warm frame and the font situation,
//! both of which govern every test here.
//!
//! [`a_drag_along_its_own_strip_offers_no_window`] is the positive control: it
//! says the pump reaches the tabs at all, without which every absence asserted
//! below is satisfied by a build that never sensed a drag.
//! [`a_drag_carried_back_onto_the_dock_docks_it`] is the second: it says the
//! same gesture, carried out and back, ends as a dock.
//!
//! # ★ What this file does NOT measure, and where that is measured instead
//!
//! Three affordances read one drag, and only one of them may answer it. Every
//! absence asserted here — *no window over a compartment*, *no window under a
//! reorder caret*, *a return to the dock docks* — is delivered by the single
//! geometric predicate [`super::tear::outside_the_dock`], not by the
//! stand-downs and the settlement ordering that also appear to deliver it.
//! Planting a defect in either of those leaves all eight tests green, because
//! no input reaches them.
//!
//! That is written down rather than fixed because both are wanted in a release
//! build. What measures the implication they rest on is the assertion in
//! [`super::drag::settle`], which every test in this file runs through.

use egui::{Pos2, Vec2};

use super::drive::{Harness, drag_to, press, release};
use super::geometry::StackAddr;
use super::{Column, DockLayout, DockSide, PanelId, SideLayout, Stack, report};

/// Two columns on the left and one on the right — the same shape
/// [`super::overlay_tests`] uses, so a coordinate means the same thing in both.
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

/// `["layers"]`, alone in its column.
const LEFT1: StackAddr = StackAddr {
    side: DockSide::Left,
    column: 1,
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

fn id(s: &str) -> PanelId {
    PanelId::new(s)
}

/// **The control.** A drag that never leaves its own tab strip is a reorder,
/// and no window is offered.
///
/// Two facts at once, and they are the same one from either side: the pump
/// reaches the tabs, and the tear does not claim a gesture the strip has
/// already claimed. An outline appearing under a reorder caret would be
/// offering to make a window out of a tab the operator is nudging one place
/// along.
#[test]
fn a_drag_along_its_own_strip_offers_no_window() {
    let mut h = Harness::new(two_columns());
    let from = h.tab_centre(LEFT0, 0);
    let along = h.tab_centre(LEFT0, 1) + Vec2::new(8.0, 0.0);

    h.frame(press(from));
    h.frame(drag_to(along));

    assert!(h.report.tab_drag.is_some(), "the strip owns this drag");
    assert_eq!(h.report.tear, None, "no window under a reorder caret");
    assert_eq!(h.rect(&report::tear_outline()), None, "and none drawn");
}

/// **Carried out of the dock, the drag offers a window at the pointer.**
///
/// The outline is the disclosure: a release that made a window with no warning
/// would be the whole gesture happening after the fact. It is drawn *around*
/// the pointer rather than beside it, so the thing about to be made is under
/// the hand making it.
///
/// The control is the first gesture: over another compartment the compass has
/// it and the tear does not, so the outline appearing a frame later is a fact
/// about where the pointer went and not about a build that offers one always.
#[test]
fn a_drag_carried_out_of_the_dock_offers_a_window() {
    let mut h = Harness::new(two_columns());
    let from = h.tab_centre(LEFT0, 0);

    h.frame(press(from));
    h.frame(drag_to(body_centre(&h, RIGHT)));
    assert!(h.report.drop_preview.is_some(), "the compass has it");
    assert_eq!(h.report.tear, None, "and the tear does not");

    h.frame(drag_to(CANVAS));
    let tear = h.report.tear.clone().expect("a window is offered");
    assert_eq!(tear.panel, id("pages"), "the panel that was pressed");
    assert_eq!(
        h.report.drop_preview, None,
        "and the compass has stood down"
    );
    assert!(
        tear.rect.contains(CANVAS),
        "the outline is under the pointer, not beside it: {:?}",
        tear.rect
    );
    assert_eq!(
        h.rect(&report::tear_outline()),
        Some(tear.rect),
        "the outline drawn is the one published"
    );
}

/// **A release out there makes the window, where the outline promised it.**
///
/// The stored position is asserted against the *reported* one rather than
/// against a coordinate computed here: the two are in different spaces — the
/// outline is in this frame's screen points and the window is placed in desktop
/// points — and the claim worth making is that the offer and the outcome are
/// the same quantity, not that a headless frame happens to put the origin at
/// zero.
#[test]
fn a_release_outside_the_dock_makes_the_window_the_outline_promised() {
    let mut h = Harness::new(two_columns());
    let from = h.tab_centre(LEFT0, 0);

    h.frame(press(from));
    h.frame(drag_to(CANVAS));
    let promised = h.report.tear.clone().expect("a window is offered").at_pts;

    h.frame(release(CANVAS));
    assert_eq!(
        h.report.floated,
        Some(id("pages")),
        "the release floated it"
    );
    assert_eq!(h.report.moved, None, "and docked it nowhere");
    assert_eq!(tabs(&h, LEFT0), ["bookmarks"], "it left the strip");
    assert_eq!(tabs(&h, RIGHT), ["properties"], "and no side gained it");

    let f = h
        .state
        .layout()
        .float_of(&id("pages"))
        .expect("it is floating");
    assert_eq!(f.pos_pts, Some(promised), "opened where it was let go");
}

/// **The window remembers the compartment it was torn from.**
///
/// The half of [`super::float`]'s central decision that a drag has to honour as
/// much as the command does: docking it back puts it where it came from, and a
/// tear that recorded no home — or recorded the address *after* the removal
/// pruned its column — would put it somewhere else.
#[test]
fn the_torn_panel_remembers_the_compartment_it_came_from() {
    let mut h = Harness::new(two_columns());
    // `layers` is alone in the second left column, so floating it prunes that
    // column: an address read after the removal names whatever slid into it.
    let from = h.tab_centre(LEFT1, 0);

    h.frame(press(from));
    h.frame(drag_to(CANVAS));
    h.frame(release(CANVAS));

    let home = h
        .state
        .layout()
        .float_of(&id("layers"))
        .expect("it is floating")
        .home;
    assert_eq!(
        (home.side, home.column, home.stack, home.tab),
        (DockSide::Left, 1, 0, 0),
        "the home is the address it had BEFORE the removal pruned the column"
    );
}

/// **The dock's own edge handle is not a tear zone.**
///
/// A side's width handle lies outside the rectangle the geometry records for
/// that side — it sits on the edge facing the document — so a predicate asking
/// only *"is the pointer inside a side"* would lay a few points of tear zone
/// down the whole height of the dock's inner edge, which is the strip every
/// drag crossing from the dock to the document passes through.
#[test]
fn the_dock_edge_handle_is_not_a_tear_zone() {
    let mut h = Harness::new(two_columns());
    let from = h.tab_centre(LEFT0, 0);
    let handle = h
        .rect(&report::side_splitter(DockSide::Left))
        .expect("the left side drew its width handle")
        .center();

    h.frame(press(from));
    h.frame(drag_to(CANVAS));
    assert!(h.report.tear.is_some(), "the witness: a tear is reachable");

    h.frame(drag_to(handle));
    assert_eq!(h.report.tear, None, "no window offered on the edge handle");

    h.frame(release(handle));
    assert_eq!(h.report.floated, None, "and none made");
    assert_eq!(tabs(&h, LEFT0), ["pages", "bookmarks"], "nothing moved");
}

/// **The offer is published while the button is down and gone after it.**
///
/// An affordance whose whole form is an outline drawn during a gesture cannot
/// be captured after the gesture, so the report is the checkable half — and a
/// report surviving the release would be a promise of a window the operator has
/// already been given.
#[test]
fn the_outline_is_published_and_then_goes() {
    let mut h = Harness::new(two_columns());
    let from = h.tab_centre(LEFT0, 0);

    h.frame(press(from));
    h.frame(drag_to(CANVAS));
    assert!(h.report.tear.is_some(), "offered while the button is down");

    h.frame(release(CANVAS));
    assert_eq!(h.report.tear, None, "and gone on the frame it lands");

    h.warm();
    assert_eq!(h.report.tear, None, "no later frame resurrects it");
    assert_eq!(h.rect(&report::tear_outline()), None, "nothing drawn");
    assert_eq!(h.report.tab_drag, None, "and the drag is over");
}

/// **A drag carried out of the dock and back in again docks, and makes no
/// window.**
///
/// The end-to-end claim the other tests each hold one end of: an offer made on
/// one frame is not a commitment, and the affordance that answers the release
/// is the one live on the frame the button came up.
#[test]
fn a_drag_carried_back_onto_the_dock_docks_it() {
    let mut h = Harness::new(two_columns());
    let from = h.tab_centre(LEFT0, 0);

    h.frame(press(from));
    h.frame(drag_to(CANVAS));
    assert!(h.report.tear.is_some(), "the witness: it was offered");

    let onto = body_centre(&h, RIGHT);
    h.frame(drag_to(onto));
    assert_eq!(h.report.tear, None, "the tear stood down on the way back");

    h.frame(release(onto));
    assert_eq!(h.report.moved, Some(id("pages")), "it docked");
    assert_eq!(h.report.floated, None, "and made no window");
    assert!(
        !h.state.layout().is_floating(&id("pages")),
        "nothing is floating"
    );
}

/// **No window over another compartment's tab strip either.**
///
/// A strip that is not the drag's own is the compass's stripless case — an
/// insertion caret between two of that stack's tabs — and it lies at the very
/// top of the compartment, a few points inside the dock's outer edge. A
/// predicate testing the pointer against each compartment's *body* rather than
/// against the side would offer a window along every tab bar in the dock.
#[test]
fn a_drag_over_another_strip_offers_no_window() {
    let mut h = Harness::new(two_columns());
    let from = h.tab_centre(LEFT0, 0);
    let other = h.tab_centre(RIGHT, 0);

    h.frame(press(from));
    h.frame(drag_to(CANVAS));
    assert!(h.report.tear.is_some(), "the witness: it was offered");

    h.frame(drag_to(other));
    assert!(h.report.drop_preview.is_some(), "the caret has the gesture");
    assert_eq!(h.report.tear, None, "and no window is offered");
}
