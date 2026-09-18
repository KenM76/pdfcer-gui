//! Driven tests for **dragging a panel into a different compartment** — the
//! zones it is offered, the compartment the offer promises, and the layout the
//! release produces.
//!
//! # These drive `egui`, and that is the whole point
//!
//! [`super::DockLayout::move_panel`], [`super::DockLayout::resolve_drop`] and
//! [`super::DockLayout::preview_drop`] all have unit tests of their own, and
//! every one of them would pass over a build where the overlay never drew, the
//! release was never read, or the intent was raised and never applied. A verb's
//! unit tests cannot see the chain in front of it. So every test here pumps
//! real `egui::Event`s through a real `Context` and asserts on what came back.
//!
//! # The driver
//!
//! [`super::drive`]. Its header carries the warm frame and the font situation,
//! both of which govern every test here. The positive control this file owes it
//! is [`a_drag_along_its_own_strip_is_not_a_drop`], which is also the test that
//! says the two affordances do not both claim one gesture.

use egui::{Pos2, Rect, Vec2};

use super::compass::DropZone;
use super::drive::{Harness, drag_to, press, release};
use super::drop::DropTarget;
use super::geometry::{ColumnAddr, StackAddr};
use super::{Column, DockLayout, DockSide, PanelId, SideLayout, Stack};

/// Two columns on the left and one on the right — the smallest layout in which
/// every release the grammar can express is reachable.
///
/// `layers` alone in the second left column is the load-bearing part: taking it
/// out prunes that column, so a drop of `layers` anywhere re-lays the whole left
/// side. That is the case a preview read off the target's *current* rect gets
/// wrong, and [`the_outcome_is_the_layout_after_the_take_not_the_target_as_it_stands`]
/// is what measures it.
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

/// The tabs of one compartment, as strings.
fn tabs(h: &Harness, at: StackAddr) -> Vec<String> {
    h.state.layout().side(at.side).columns[at.column].stacks[at.stack]
        .tabs
        .iter()
        .map(|p| p.as_str().to_owned())
        .collect()
}

/// How many columns a side has.
fn columns(h: &Harness, side: DockSide) -> usize {
    h.state.layout().side(side).columns.len()
}

/// A compartment less its tab strip — the rectangle the compass divides.
///
/// Read through [`super::compass::body_of`] rather than subtracted here,
/// because a fixture with its own spelling of "the compartment less its strip"
/// would be aiming at zones a strip's height away from the ones the release is
/// resolved against, and would report that as a defect in the overlay.
fn body(h: &Harness, at: StackAddr) -> Rect {
    super::compass::body_of(h.state.geometry(), at).expect("the compartment was drawn")
}

/// The middle of a compartment's centre zone.
fn body_centre(h: &Harness, at: StackAddr) -> Pos2 {
    body(h, at).center()
}

/// **The control.** A drag that stays on its own tab strip is a reorder, and
/// the compass does not appear.
///
/// Two things at once, and they are the same fact from either side: the pump
/// reaches the tabs at all — without which every absence asserted below is
/// satisfied by a build that never sensed a drag — and the two affordances do
/// not both claim one gesture. An overlay that offered a five-zone compass over
/// the compartment a reorder is passing across would put a wash of colour under
/// the caret the operator is aiming with.
#[test]
fn a_drag_along_its_own_strip_is_not_a_drop() {
    let mut h = Harness::new(two_columns());
    let from = h.tab_centre(LEFT0, 0);
    let along = h.tab_centre(LEFT0, 1) + Vec2::new(8.0, 0.0);

    h.frame(press(from));
    h.frame(drag_to(along));

    assert!(h.report.tab_drag.is_some(), "the strip owns this drag");
    assert_eq!(h.report.drop_preview, None, "and the compass stands down");
    assert_eq!(h.rect(&super::report::drop_outcome()), None);
    assert_eq!(
        h.rect(&super::report::drop_zone(
            DockSide::Left,
            0,
            0,
            DropZone::Centre
        )),
        None
    );
}

/// **The gesture.** Drag a tab onto another compartment's centre and release,
/// and the panel joins that group.
///
/// Four assertions, because four different builds pass any three: one that
/// offered and never applied, one that applied and never offered, one that left
/// the panel in both stacks, and one that reported the move as a reorder — which
/// is a real distinction, because an application telling the operator "moved"
/// for a rearranged tab bar is reporting a structural change that did not
/// happen.
#[test]
fn dragging_a_tab_onto_another_compartments_centre_joins_that_group() {
    let mut h = Harness::new(two_columns());
    let from = h.tab_centre(LEFT0, 0);
    let onto = body_centre(&h, RIGHT);

    h.frame(press(from));
    h.frame(drag_to(onto));

    let offer = h
        .report
        .drop_preview
        .clone()
        .expect("the compass is offered");
    assert_eq!(offer.panel.as_str(), "pages");
    assert_eq!(offer.landing.over, RIGHT);
    assert_eq!(offer.landing.zone, Some(DropZone::Centre));
    assert_eq!(
        offer.landing.target,
        DropTarget::Tab {
            stack: RIGHT,
            gap: 1
        },
        "the centre appends"
    );
    assert!(offer.lands, "this release changes the layout");

    h.frame(release(onto));

    assert_eq!(
        h.report.moved.as_ref().map(PanelId::as_str),
        Some("pages"),
        "the release was read"
    );
    assert_eq!(h.report.reordered, None, "a move is not a reorder");
    assert_eq!(tabs(&h, RIGHT), ["properties", "pages"]);
    assert_eq!(tabs(&h, LEFT0), ["bookmarks"], "and it left the old stack");
    assert!(h.report.layout_changed, "the arrangement is worth saving");
    assert_eq!(h.report.drop_preview, None, "the drag is over");
}

/// **A release against an edge starts a column**, which is the half of the
/// grammar a tab strip cannot express at all.
#[test]
fn a_release_against_a_compartments_edge_starts_a_column_beside_it() {
    let mut h = Harness::new(two_columns());
    assert_eq!(columns(&h, DockSide::Right), 1);
    let from = h.tab_centre(LEFT0, 0);
    let body = body(&h, RIGHT);
    let onto = Pos2::new(body.right() - 4.0, body.center().y);

    h.frame(press(from));
    h.frame(drag_to(onto));

    let offer = h
        .report
        .drop_preview
        .clone()
        .expect("the compass is offered");
    assert_eq!(offer.landing.zone, Some(DropZone::Right));
    assert_eq!(
        offer.landing.target,
        DropTarget::Column {
            side: DockSide::Right,
            gap: 1
        }
    );

    h.frame(release(onto));

    assert_eq!(h.report.moved.as_ref().map(PanelId::as_str), Some("pages"));
    assert_eq!(columns(&h, DockSide::Right), 2, "a column was started");
    assert_eq!(tabs(&h, RIGHT), ["properties"]);
    assert_eq!(
        tabs(
            &h,
            StackAddr {
                side: DockSide::Right,
                column: 1,
                stack: 0
            }
        ),
        ["pages"]
    );
}

/// **A release over another stack's strip is a boundary, not a zone** — and it
/// draws the same caret the reorder does.
///
/// The compass is not offered there, because a tab strip already answers a
/// better question than "which of five": it answers *where among these tabs*,
/// and the operator aiming at a strip is aiming between two labels.
#[test]
fn a_release_over_another_stacks_strip_is_a_caret_between_its_tabs() {
    let mut h = Harness::new(two_columns());
    let from = h.tab_centre(LEFT0, 0);
    let strip = h
        .state
        .geometry()
        .strip_rect(RIGHT)
        .expect("the strip was drawn");
    let onto = Pos2::new(strip.right() - 4.0, strip.center().y);

    h.frame(press(from));
    h.frame(drag_to(onto));

    // ★ The origin strip must have let go. Every strip in the dock sits at
    // the same y, so a reorder bounded by y alone claims this gesture — drawing
    // a caret four hundred points from the pointer and never offering the drop.
    assert_eq!(
        h.report.tab_drag, None,
        "the strip the drag left does not still own it"
    );
    let offer = h.report.drop_preview.clone().expect("the drop is offered");
    assert_eq!(offer.landing.zone, None, "a strip is not a compass");
    assert_eq!(
        offer.landing.target,
        DropTarget::Tab {
            stack: RIGHT,
            gap: 1
        }
    );
    let caret = h
        .rect(&super::report::tab_caret(DockSide::Right, 0, 0))
        .expect("the caret was published");
    assert!(
        strip.contains_rect(caret),
        "the caret is on the strip it names: {caret:?} in {strip:?}"
    );

    h.frame(release(onto));
    assert_eq!(tabs(&h, RIGHT), ["properties", "pages"]);
}

/// ★ **The zones and the outcome are published as regions, and they go.**
///
/// The visible form of this affordance is a wash of colour over a
/// quadrilateral: precise to look at, and nothing a harness can assert on. So
/// the armed zone and the outcome are named, for the reason
/// `crate::dock::report`'s header gives — and a region published on every frame
/// would be furniture rather than a marker, which is why the absence after the
/// release is asserted too.
#[test]
fn the_armed_zone_and_the_outcome_are_published_and_then_go() {
    let mut h = Harness::new(two_columns());
    let zone = super::report::drop_zone(DockSide::Right, 0, 0, DropZone::Centre);
    let outcome = super::report::drop_outcome();
    assert_eq!(h.rect(&zone), None, "nothing before the gesture");
    assert_eq!(h.rect(&outcome), None);

    let from = h.tab_centre(LEFT0, 0);
    let onto = body_centre(&h, RIGHT);
    h.frame(press(from));
    h.frame(drag_to(onto));

    let offer = h
        .report
        .drop_preview
        .clone()
        .expect("the compass is offered");
    let armed = h.rect(&zone).expect("the armed zone was published");
    assert!(armed.contains(onto), "the pointer is in it: {armed:?}");
    assert!(
        body(&h, RIGHT).contains_rect(armed),
        "and it is inside the compartment it divides"
    );
    assert_eq!(
        h.rect(&outcome),
        Some(offer.rect),
        "the outcome region is the compartment the offer names"
    );

    h.frame(release(onto));
    h.warm();
    assert_eq!(h.rect(&zone), None, "the zones went with the drag");
    assert_eq!(h.rect(&outcome), None);
}

/// ★ **The zones divide the body, not the whole compartment.**
///
/// The tab strip is not part of the compass — it answers a better question, and
/// [`super::compass::body_of`] subtracts it before dividing. A compass laid over
/// the compartment *including* its strip is drawn one strip height above the
/// zones the release is resolved against: the operator aims at a painted "top"
/// band and the pointer is over the tab bar, so the release inserts a tab where
/// a split was offered. R8b's failure mode #2, in paint.
///
/// The assertion is the armed band's **top edge**, because that is what the
/// wrong mechanism cannot produce: it would start the band at the compartment's
/// top, a strip's height higher. Containment inside the compartment is satisfied
/// by both, which is why it is not the measurement.
#[test]
fn the_zones_divide_the_compartment_less_its_strip() {
    let mut h = Harness::new(two_columns());
    let body = body(&h, RIGHT);
    let strip = h
        .state
        .geometry()
        .strip_rect(RIGHT)
        .expect("the strip was drawn");
    assert!(
        strip.height() > 1.0,
        "a strip of no height would make this test vacuous: {strip:?}"
    );
    let from = h.tab_centre(LEFT0, 0);
    let onto = Pos2::new(body.center().x, body.top() + 4.0);

    h.frame(press(from));
    h.frame(drag_to(onto));

    let offer = h
        .report
        .drop_preview
        .clone()
        .expect("the compass is offered");
    assert_eq!(offer.landing.zone, Some(DropZone::Top));
    let armed = h
        .rect(&super::report::drop_zone(
            DockSide::Right,
            0,
            0,
            DropZone::Top,
        ))
        .expect("the armed zone was published");
    assert!(
        (armed.top() - body.top()).abs() < 0.5,
        "the top band starts where the strip ends: band {:?}, body {:?}, strip {strip:?}",
        armed,
        body
    );
    assert!(
        armed.bottom() < body.center().y,
        "and it is a band, not the whole compartment: {armed:?}"
    );
}

/// ★★ **The outcome is the layout after the take, not the target as it
/// stands.**
///
/// `layers` is alone in the second left column, so removing it prunes that
/// column and the first one grows to the whole side — *before* the panel
/// arrives. A preview that looked the destination up in the current geometry
/// would outline half the side and then deliver the whole of it, which is the
/// disclosure failure this project names failure mode #2, in the commonest drag
/// there is.
///
/// The width comparison is what the wrong mechanism cannot produce: it would
/// return the destination's rect unchanged, and that rect is measured here
/// before the gesture starts. The equality after the release is the other half —
/// the promise was kept, not merely different.
#[test]
fn the_outcome_is_the_layout_after_the_take_not_the_target_as_it_stands() {
    let mut h = Harness::new(two_columns());
    let before = h.stack_rect(LEFT0);
    let from = h.tab_centre(LEFT1, 0);
    let onto = body_centre(&h, LEFT0);

    h.frame(press(from));
    h.frame(drag_to(onto));

    let offer = h
        .report
        .drop_preview
        .clone()
        .expect("the compass is offered");
    assert_eq!(offer.panel.as_str(), "layers");
    assert_eq!(
        offer.landing.target,
        DropTarget::Tab {
            stack: LEFT0,
            gap: 2
        }
    );
    assert!(
        offer.rect.width() > before.width() + 1.0,
        "the outcome is the widened column, not the one on screen: {} then {}",
        before.width(),
        offer.rect.width()
    );

    h.frame(release(onto));
    h.warm();

    assert_eq!(tabs(&h, LEFT0), ["pages", "bookmarks", "layers"]);
    assert_eq!(
        columns(&h, DockSide::Left),
        1,
        "the emptied column was pruned"
    );
    let after = h.stack_rect(LEFT0);
    assert!(
        (after.width() - offer.rect.width()).abs() < 0.5
            && (after.height() - offer.rect.height()).abs() < 0.5,
        "the release delivered what the offer promised: {:?} then {:?}",
        offer.rect,
        after
    );
}

/// **A release that changes nothing is offered dimmed, not refused.**
///
/// Dropping a panel back into the middle of the group it already leads is
/// legal and permutes nothing. Refusing it would be a lie about the grammar;
/// promising a move that will not happen is the other half of the same lie. So
/// the offer stands, `lands` is false — which is what knocks the ink back — and
/// the outcome outlined is the compartment the panel is already in.
#[test]
fn a_release_that_would_change_nothing_is_offered_dimmed() {
    let mut h = Harness::new(two_columns());
    let from = h.tab_centre(LEFT1, 0);
    let onto = body_centre(&h, LEFT1);

    h.frame(press(from));
    h.frame(drag_to(onto));

    let offer = h.report.drop_preview.clone().expect("the offer stands");
    assert_eq!(offer.landing.over, LEFT1);
    assert!(!offer.lands, "this release permutes nothing: {offer:?}");
    assert_eq!(
        offer.rect,
        h.stack_rect(LEFT1),
        "and the outcome is where it already is"
    );

    h.frame(release(onto));

    assert_eq!(h.report.moved, None);
    assert_eq!(tabs(&h, LEFT1), ["layers"]);
    assert_eq!(columns(&h, DockSide::Left), 2, "nothing was pruned");
}

/// **A drag carried over a splitter is offered nothing**, and releasing there
/// docks nothing.
///
/// A splitter is recorded in no compartment's rect, so
/// [`super::DockLayout::resolve_drop`] answers `None` — the same answer as the
/// canvas. Asserted because the failure mode is the opposite of a refusal: a
/// resolution that snapped to the nearest compartment would dock a panel the
/// operator released into a gap on purpose.
#[test]
fn a_release_over_a_splitter_docks_nothing() {
    let mut h = Harness::new(two_columns());
    let seam = Pos2::new(
        (h.stack_rect(LEFT0).right() + h.stack_rect(LEFT1).left()) / 2.0,
        body_centre(&h, LEFT0).y,
    );
    let from = h.tab_centre(LEFT0, 0);

    h.frame(press(from));
    h.frame(drag_to(body_centre(&h, RIGHT)));
    // The witness: without it the absence below is satisfied by a build that
    // never offered a drop anywhere.
    assert!(h.report.drop_preview.is_some(), "a drop is on offer");

    h.frame(drag_to(seam));
    assert_eq!(h.report.drop_preview, None, "nothing is offered on a seam");

    h.frame(release(seam));
    assert_eq!(h.report.moved, None);
    assert_eq!(tabs(&h, LEFT0), ["pages", "bookmarks"]);
    assert_eq!(columns(&h, DockSide::Left), 2);
}

/// ★ **A drag pulled out over the document ends, and docks nothing.**
///
/// The canvas is where tearing a panel out will attach. Until it does, a
/// release there must leave the layout alone *and* end the gesture: a drag that
/// survived the release would carry its compass into the next frame, and the
/// one after, with no button held to get rid of it.
#[test]
fn a_release_over_the_document_docks_nothing_and_ends_the_drag() {
    let mut h = Harness::new(two_columns());
    let from = h.tab_centre(LEFT0, 0);
    let canvas = Pos2::new(760.0, 600.0);

    h.frame(press(from));
    h.frame(drag_to(body_centre(&h, RIGHT)));
    assert!(h.report.drop_preview.is_some(), "a drop is on offer");

    h.frame(drag_to(canvas));
    assert_eq!(h.report.drop_preview, None, "no compass over the document");

    h.frame(release(canvas));
    assert_eq!(h.report.moved, None);
    assert_eq!(tabs(&h, LEFT0), ["pages", "bookmarks"]);
    // And the drag is over: another frame must not resurrect it.
    h.warm();
    assert_eq!(h.report.drop_preview, None);
    assert_eq!(h.report.tab_drag, None);
}

/// **A release against a compartment's top or bottom splits its column**, which
/// is the third and last thing the grammar can do.
#[test]
fn a_release_against_a_compartments_bottom_splits_its_column() {
    let mut h = Harness::new(two_columns());
    let from = h.tab_centre(LEFT0, 0);
    let body = body(&h, RIGHT);
    let onto = Pos2::new(body.center().x, body.bottom() - 4.0);

    h.frame(press(from));
    h.frame(drag_to(onto));

    let offer = h
        .report
        .drop_preview
        .clone()
        .expect("the compass is offered");
    assert_eq!(offer.landing.zone, Some(DropZone::Bottom));
    assert_eq!(
        offer.landing.target,
        DropTarget::Stack {
            column: ColumnAddr::new(DockSide::Right, 0),
            gap: 1
        }
    );
    assert!(
        offer.rect.bottom() > body.center().y,
        "the outcome is the lower half: {:?}",
        offer.rect
    );

    h.frame(release(onto));

    assert_eq!(h.report.moved.as_ref().map(PanelId::as_str), Some("pages"));
    assert_eq!(columns(&h, DockSide::Right), 1, "one column, split in two");
    assert_eq!(h.state.layout().right.columns[0].stacks.len(), 2);
    assert_eq!(tabs(&h, RIGHT), ["properties"]);
    assert_eq!(
        tabs(
            &h,
            StackAddr {
                side: DockSide::Right,
                column: 0,
                stack: 1
            }
        ),
        ["pages"]
    );
}
