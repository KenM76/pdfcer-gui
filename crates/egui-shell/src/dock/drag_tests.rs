//! Driven tests for **dragging a tab along its own strip** — the gesture, the
//! preview it publishes, and the layout the release produces.
//!
//! # These drive `egui`, and the reason is the whole point
//!
//! [`super::model::DockLayout::reorder_tab`] has unit tests of its own, and
//! they would all pass over a build where the tab never sensed a drag, the
//! caret never appeared, or the release was read from a `Response` that a drag
//! ending off the strip never produces. A verb's unit tests cannot see the
//! chain in front of it. So every test here pumps real `egui::Event`s through a
//! real `Context` and asserts on what came back.
//!
//! # The driver
//!
//! [`super::drive`]. Its header carries the warm frame and the font situation,
//! both of which govern every test here. The positive control this file owes it
//! is [`a_press_that_does_not_move_activates_the_tab_and_reorders_nothing`].

use egui::{Event, Pos2, Vec2};

use super::drive::{Harness, press, release};
use super::geometry::StackAddr;
use super::{Column, DockLayout, DockSide, PanelId, SideLayout, Stack};

/// Three panels in one stack on the left, and nothing on the right.
fn three_tabs() -> DockLayout {
    DockLayout::new(
        SideLayout::new([Column::new([Stack::tabbed([
            "pages",
            "bookmarks",
            "layers",
        ])])])
        .with_width(320.0),
        SideLayout::default(),
    )
}

/// The stack under test.
const STACK: StackAddr = StackAddr {
    side: DockSide::Left,
    column: 0,
    stack: 0,
};

/// The tab order of the stack under test, as strings.
fn order(h: &Harness) -> Vec<String> {
    h.state.layout().left.columns[0].stacks[0]
        .tabs
        .iter()
        .map(|p| p.as_str().to_owned())
        .collect()
}

/// The panel whose body the dock is drawing.
fn active(h: &Harness) -> String {
    h.state.layout().left.columns[0].stacks[0]
        .active_panel()
        .map_or_else(String::new, |p| p.as_str().to_owned())
}

/// **Step 0's deliverable.** The dock publishes where it put every
/// compartment, by address, and the addresses agree with each other.
///
/// The nesting assertions are the ones that matter: a record whose tab rects
/// were correct but whose stack rect named a different compartment would hit-
/// test a drop into the wrong stack, and every rect in it would still look
/// plausible read one at a time.
#[test]
fn the_dock_publishes_where_it_drew_every_compartment() {
    let h = Harness::new(three_tabs());
    let g = h.state.geometry();

    assert!(!g.is_empty(), "nothing was recorded");
    let stack = g.stack_rect(STACK).expect("the stack was drawn");
    let strip = g.strip_rect(STACK).expect("the strip was drawn");
    assert!(stack.contains_rect(strip), "the strip is inside its stack");

    let tabs: Vec<_> = g.tabs_of(STACK).collect();
    assert_eq!(tabs.len(), 3, "three tabs drawn: {tabs:?}");
    for (i, r) in &tabs {
        assert!(
            strip.contains_rect(*r),
            "tab {i} is inside the strip: {r:?}"
        );
        assert_eq!(g.tab_at(r.center()), Some(STACK.tab(*i)));
        assert_eq!(g.stack_at(r.center()), Some(STACK));
        assert_eq!(g.strip_at(r.center()), Some(STACK));
    }
    // Drawn left to right, in tab order.
    assert!(tabs[0].1.left() < tabs[1].1.left() && tabs[1].1.left() < tabs[2].1.left());
    // Nothing on the right side, so nothing is recorded there.
    assert_eq!(g.stack_at(Pos2::new(1390.0, 400.0)), None);
}

/// **The control.** A press that does not move is still a click, so the tab
/// activates and nothing is reordered.
///
/// It is also what proves the event pump reaches the tabs at all: every other
/// test in this file would pass vacuously against a dock whose tabs were never
/// under the pointer.
#[test]
fn a_press_that_does_not_move_activates_the_tab_and_reorders_nothing() {
    let mut h = Harness::new(three_tabs());
    assert_eq!(active(&h), "pages");
    let target = h.tab_centre(STACK, 2);

    h.frame(press(target));
    h.frame(release(target));

    assert_eq!(
        h.report.activated.as_ref().map(PanelId::as_str),
        Some("layers")
    );
    assert_eq!(h.report.reordered, None);
    assert_eq!(order(&h), ["pages", "bookmarks", "layers"]);
    assert_eq!(active(&h), "layers");
}

/// **The gesture.** Drag the first tab past the last and it lands last.
///
/// Three assertions, because three different builds pass any two of them: one
/// that previewed and never applied, one that applied and never previewed, and
/// one that moved the tab but left [`Stack::active`] pointing at whatever is
/// now at the old index — which switches the panel on screen as a side effect
/// of rearranging tabs.
#[test]
fn dragging_a_tab_past_the_last_one_moves_it_to_the_end() {
    let mut h = Harness::new(three_tabs());
    let from = h.tab_centre(STACK, 0);
    let past_the_end = h.tab_centre(STACK, 2) + Vec2::new(12.0, 0.0);

    h.frame(press(from));
    h.frame(vec![Event::PointerMoved(past_the_end)]);

    let preview = h.report.tab_drag.clone().expect("a drag is in flight");
    assert_eq!(preview.panel.as_str(), "pages");
    assert_eq!(preview.from.tab, 0);
    assert_eq!(preview.gap, 3, "the boundary past the last tab");

    h.frame(release(past_the_end));

    assert_eq!(
        h.report.reordered.as_ref().map(PanelId::as_str),
        Some("pages")
    );
    assert_eq!(order(&h), ["bookmarks", "layers", "pages"]);
    assert_eq!(active(&h), "pages", "the visible panel did not change");
    assert!(h.report.layout_changed, "the arrangement is worth saving");
    assert_eq!(h.report.tab_drag, None, "the drag is over");
}

/// **The other direction.** The boundary flips at a neighbour's centre, not at
/// its edge, so a drag that has passed the middle of tab 0 lands before it.
#[test]
fn dragging_a_tab_leftwards_lands_it_at_the_boundary_it_has_passed() {
    let mut h = Harness::new(three_tabs());
    let from = h.tab_centre(STACK, 2);
    let left_of_the_first = h.tab_centre(STACK, 0) - Vec2::new(12.0, 0.0);

    h.frame(press(from));
    h.frame(vec![Event::PointerMoved(left_of_the_first)]);
    assert_eq!(h.report.tab_drag.as_ref().map(|d| d.gap), Some(0));

    h.frame(release(left_of_the_first));
    assert_eq!(order(&h), ["layers", "pages", "bookmarks"]);
}

/// **A drag released where it started changes nothing** — and says so by
/// reporting no reorder rather than by reporting one that moved zero tabs.
#[test]
fn a_drag_released_where_it_started_is_not_a_reorder() {
    let mut h = Harness::new(three_tabs());
    let from = h.tab_centre(STACK, 1);
    // Far enough to pass egui's click threshold, not far enough to leave the
    // tab's own half of the strip.
    let nudged = from + Vec2::new(8.0, 0.0);

    h.frame(press(from));
    h.frame(vec![Event::PointerMoved(nudged)]);
    let preview = h.report.tab_drag.as_ref().expect("the drag was sensed");
    assert!(
        !preview.lands(),
        "the caret is dimmed here, because this release permutes nothing: {preview:?}"
    );

    h.frame(release(nudged));
    assert_eq!(h.report.reordered, None);
    assert_eq!(order(&h), ["pages", "bookmarks", "layers"]);
}

/// ★★ **A drag pulled off the strip ends, and reorders nothing.**
///
/// Two failures in one test, because they are the same mistake seen from
/// either side.
///
/// The first is a caret nobody can get rid of: a release read from the tab's
/// own `Response` never arrives when the pointer left the widget, so the drag
/// survives into the next frame, and the next, with the strip still painting
/// its caret and the cursor still `Grabbing`.
///
/// The second is a reorder the operator did not ask for. A boundary is
/// resolved from x alone, and x is perfectly well defined over the middle of
/// the document — so a drag pulled down into the page would quietly rearrange
/// the strip it left. What that gesture means is [`super::tear`]'s subject and
/// is asserted there; what it must never mean is a permutation of the strip the
/// pointer has left, and that is what this measures.
#[test]
fn a_drag_pulled_off_the_strip_ends_and_reorders_nothing() {
    let mut h = Harness::new(three_tabs());
    let from = h.tab_centre(STACK, 0);
    let past_the_end = h.tab_centre(STACK, 2) + Vec2::new(12.0, 0.0);
    let off_in_the_canvas = Pos2::new(900.0, 600.0);

    h.frame(press(from));
    h.frame(vec![Event::PointerMoved(past_the_end)]);
    // The witness. Without it every assertion below is satisfied by a build
    // that never started a drag at all — which is what an absence assertion
    // with no positive control always measures.
    assert!(h.report.tab_drag.is_some(), "a drag is in flight");

    h.frame(vec![Event::PointerMoved(off_in_the_canvas)]);
    assert_eq!(h.report.tab_drag, None, "no caret over the document");

    h.frame(release(off_in_the_canvas));
    assert_eq!(
        h.report.reordered, None,
        "a release off the strip is no reorder"
    );
    // The panel left the strip, and the two that stayed are in the order they
    // were in. A tear that also permuted its neighbours would satisfy
    // `reordered == None` — the intent is not raised — while leaving the strip
    // rearranged, so the remaining order is asserted and not merely the count.
    assert_eq!(order(&h), ["bookmarks", "layers"]);
    // And the drag is over: another frame must not resurrect it.
    h.warm();
    assert_eq!(h.report.tab_drag, None);
}

/// **The band is wider than the strip**, so an ordinary horizontal drag with a
/// few points of vertical wander is still a reorder.
///
/// The failure it refuses is a caret that blinks in and out along a gesture
/// that never left the tab bar to the eye — which is what bounding the reorder
/// by the strip rectangle exactly produces.
#[test]
fn a_drag_that_wanders_a_little_below_the_strip_is_still_a_reorder() {
    let mut h = Harness::new(three_tabs());
    let from = h.tab_centre(STACK, 0);
    let strip = h
        .state
        .geometry()
        .strip_rect(STACK)
        .expect("the strip was drawn");
    let just_below = Pos2::new(h.tab_centre(STACK, 2).x + 12.0, strip.bottom() + 6.0);

    h.frame(press(from));
    h.frame(vec![Event::PointerMoved(just_below)]);
    assert_eq!(h.report.tab_drag.as_ref().map(|d| d.gap), Some(3));

    h.frame(release(just_below));
    assert_eq!(order(&h), ["bookmarks", "layers", "pages"]);
}

/// ★ **The caret is published as a region, and it is where the tab will land.**
///
/// The operator's wording for this feature is *"clear markers of where it is
/// going to move to"*, and a marker drawn at the wrong boundary is worse than
/// none: it is a promise the release does not keep. So this asserts the
/// position, not merely that something was drawn — which the report's own
/// `gap` field would already have said.
///
/// It also asserts the caret **goes**. A region that is published on every
/// frame is not a marker, it is furniture, and a harness reading it as a
/// change would then see a drag in flight forever.
#[test]
fn the_caret_marks_the_boundary_the_release_will_use_and_then_goes() {
    let mut h = Harness::new(three_tabs());
    let name = super::report::tab_caret(DockSide::Left, 0, 0);
    assert_eq!(h.rect(&name), None, "no caret before the gesture");

    let from = h.tab_centre(STACK, 0);
    let over_the_last = h.tab_centre(STACK, 2) + Vec2::new(12.0, 0.0);
    let tab2 = h
        .state
        .geometry()
        .tab_rect(STACK.tab(2))
        .expect("tab 2 was drawn");
    let strip = h
        .state
        .geometry()
        .strip_rect(STACK)
        .expect("the strip was drawn");

    h.frame(press(from));
    h.frame(vec![Event::PointerMoved(over_the_last)]);

    let caret = h.rect(&name).expect("the caret was published");
    assert!(caret.height() > 0.0 && caret.width() > 0.0, "{caret:?}");
    assert!(
        strip.contains_rect(caret),
        "the caret is on the strip: {caret:?} in {strip:?}"
    );
    // Gap 3 is past the last tab, so the caret sits at its right edge.
    assert!(
        (caret.center().x - tab2.right()).abs() < 2.0,
        "caret at {}, tab 2 ends at {}",
        caret.center().x,
        tab2.right()
    );

    h.frame(release(over_the_last));
    h.warm();
    assert_eq!(h.rect(&name), None, "the caret went with the drag");
}

/// **The caret at the first boundary is drawn whole, not half clipped.**
///
/// Boundary zero is the left edge of the first tab, which is the strip's own
/// left edge, so a caret centred on it has half its width outside the `Ui` that
/// clips it. The operator then sees the one marker in the strip drawn at half
/// the weight of every other, at the end where a faint marker is hardest to
/// tell from none.
///
/// The containment has no tolerance, and that is the point: an epsilon here
/// passes on the build this test refuses.
#[test]
fn the_caret_at_the_first_boundary_is_drawn_whole() {
    let mut h = Harness::new(three_tabs());
    let name = super::report::tab_caret(DockSide::Left, 0, 0);
    let strip = h
        .state
        .geometry()
        .strip_rect(STACK)
        .expect("the strip was drawn");
    let tab0 = h
        .state
        .geometry()
        .tab_rect(STACK.tab(0))
        .expect("tab 0 was drawn");
    assert!(
        (tab0.left() - strip.left()).abs() < 0.5,
        "tab 0 starts at {}, inside a strip starting at {} — this test needs them flush",
        tab0.left(),
        strip.left()
    );

    let from = h.tab_centre(STACK, 2);
    let before_the_first = h.tab_centre(STACK, 0) - Vec2::new(12.0, 0.0);
    h.frame(press(from));
    h.frame(vec![Event::PointerMoved(before_the_first)]);

    let caret = h.rect(&name).expect("the caret was published");
    assert!(
        strip.contains_rect(caret),
        "the caret is whole and on the strip: {caret:?} in {strip:?}"
    );
    assert!(
        (caret.width() - 2.0).abs() < 0.01,
        "the caret keeps its weight: {caret:?}"
    );

    h.frame(release(before_the_first));
    assert_eq!(order(&h), ["layers", "pages", "bookmarks"]);
}

/// **The two boundaries that change nothing are the tab's own edges**, and the
/// caret is dimmed at exactly those.
///
/// Stated as its own test because the painting reads one predicate and the
/// reorder reads another — `gap` against `from.tab` here, `gap > from`'s
/// off-by-one in the model — and the pair a reader most wants proved is that
/// "dimmed" and "the order did not change" name the same boundaries.
#[test]
fn the_caret_dims_at_the_two_boundaries_against_the_dragged_tabs_own_edges() {
    let at = |tab: usize, gap: usize| super::TabDragPreview {
        panel: PanelId::new("pages"),
        from: super::PanelAddress {
            side: DockSide::Left,
            column: 0,
            stack: 0,
            tab,
        },
        gap,
    };
    assert!(!at(1, 1).lands(), "against its own left edge");
    assert!(!at(1, 2).lands(), "against its own right edge");
    assert!(at(1, 0).lands());
    assert!(at(1, 3).lands());
    // Tab zero has only one no-op boundary on its left, and it is boundary
    // zero: there is no gap below it to confuse with one.
    assert!(!at(0, 0).lands());
    assert!(!at(0, 1).lands());
    assert!(at(0, 2).lands());
}
