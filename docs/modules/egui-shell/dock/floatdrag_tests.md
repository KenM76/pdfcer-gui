# `egui-shell/dock/floatdrag_tests`

## Item notes

### `fn a_float_carried_over_a_compartment_is_offered_a_drop`

Two facts at once: the report reaches the resolution at all, and the
compartment it names is the one under the point that was handed in — not the
one the panel was torn from, which is what a build that skipped the
resolution and reused [`super::DockHome`] would produce.

### `fn the_zone_aimed_at_decides_the_target`

Two claims, and the second is the one with a whole class of defect behind
it. [`super::FloatDrag::pointer`] is in the application window's own screen
points, and a caller that converted from desktop points wrongly, or a shell
that nudged what it was given, would still resolve *somewhere* and still
draw a plausible compass. So it is not enough to assert the offer equals the
grammar's answer for the point aimed at: the test also has to pick a point
whose answer differs from its neighbours', or a shifted pointer satisfies it
unchanged.

The left edge of a body is such a point. It is a [`DropZone::Left`] — a new
column beside the compartment — where the middle of the same body is a
[`DropZone::Centre`], a tab appended to it. Two different outcomes from one
rectangle, which is exactly what the five zones are for.

⚠ **What this still cannot see, and what therefore has to be driven.** The
smallest pointer error that changes the answer is the width of the edge band
— a quarter of the body, capped at `compass::EDGE_MAX_PTS`. A conversion
from desktop points that is wrong by less than that lands in the same zone
and passes every test in this file. The conversion itself is the
application's, and it is verified by driving the binary, not here.

### `fn a_release_clear_of_the_dock_leaves_it_floating`

Dragging a window around the desktop is not a dock gesture, and a build that
treated every release as one would swallow the window the moment the
operator moved it anywhere.

### `fn an_offer_not_renewed_is_over`

The report is consumed, so a frame the caller did not answer for is a frame
with no gesture — which is what makes a window closed mid-drag land nothing
rather than leave an offer standing that nothing can dismiss.

### `fn a_drag_of_a_panel_that_is_not_floating_is_declined`

A caller reads its pointer a frame behind the layout it reports against, so
a drag whose window has just been closed or docked by some other route is an
ordinary race. Offering a drop for a docked panel would move it on release.

### `fn a_tab_drag_in_flight_beats_a_reported_float_drag`

The one stand-down in the dock that input can actually reach: the other
three affordances read one `egui` pointer and exclude each other by
geometry, while this one reads a point a caller supplies and nothing stops
the two arriving on the same frame. The tab drag keeps its own caret, and
the float's release is not answered by the compartment the tab is over.
