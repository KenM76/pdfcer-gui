# `egui-shell/dock/floatwin`

## Item notes

### `fn float_title`

Falling back to the id rather than to a generic word, for
[`super::ctx::Ctx::describe`]'s reason: a window called "Panel" is
a window the operator cannot tell from another window called
"Panel", and an unregistered panel is a bug whose symptom should
name itself.

### `fn draw_header`

Returns the `Response`, which goes two places. To the application's
tab-menu handler — so the float window's menu and the dock tab's
menu are literally the same menu, resolved through the same
registry against the same conditions. And to [`super::floatgrab`],
which reads the drag half as the gesture that carries the window back
over the dock; the strip is the only surface in the window whose
pointer `egui` still reports once the cursor has left it.

### `fn draw_builtin_header`

Returns whether the button was pressed, and the strip's own
`Response` — which [`super::floatgrab`] reads for the carry, so a
consumer that has adopted no tab-menu handler still gets the gesture.
See the call site for why the built-in offer is Dock rather than
Close.

### `fn apply_float_intents`

Mirrors [`super::apply`]'s shape exactly — clone, mutate, compare —
so "did anything change" has one implementation per call and cannot be
forgotten by a new intent.

### `fn each_panel_gets_its_own_viewport`

A shared `ViewportId` is a shared OS window, and the symptom would
be one panel drawing over another rather than anything that looks
like an id collision.

### `fn an_unmoved_float_window_does_not_mark_the_layout_dirty`

The geometry intent is raised every frame for every open float. If
it reported a change every time, `layout_changed` would be true on
every frame a panel was floating and the application would rewrite
`layout.ron` continuously — a save path driven by the frame rate
rather than by the operator.

### `fn a_float_window_whose_panel_draws_nothing_is_reported_empty`

This is the falsification of [`FloatFrameReport::empty_bodies`]
written as a test rather than performed by hand: the same fixture,
the same call, and a body that does nothing at all. Every other
number the frame produces still says the window is fine —
`drawn` holds the panel and `DockFrameReport::floats_undrawn` is
zero — which is exactly why this field had to exist.

⚠ If this ever reports an empty list, the guard has stopped
guarding and a floated panel can ship as a blank window with a
title bar, which is R9 broken at the scale of a whole window.

### `fn a_float_window_whose_panel_allocates_anything_is_not_empty`

One allocation deliberately, not a full panel: the honest floor is
*anything at all was allocated*, because a panel with nothing to say
is required by R9 to say so in a **sentence** rather than to draw a
blank — so one sentence is the minimum legitimate content, and a
measurement that called it empty would fail every correct panel on a
document that gives it nothing to list.

The sentence cannot be spelled as a sentence *here*; see the
comment at the allocation for the reason, which is what makes a
label-based spelling of this test fail against a working dock.

### `fn a_docked_surfaces_intent_is_ignored_by_the_float_path`

`apply_float_intents` shares an `Intent` enum with the docked path;
a stray one must be a no-op rather than a second implementation of
a splitter drag.

### `fn a_panel_docked_back_by_another_route_forgets_its_window`

The opened-once flag is what makes a float window open at its stored
position exactly once instead of being dragged back to it every frame.
A flag that is never cleared is not a visible failure on the frame it
happens — it is a window that, the *next* time the operator floats that
panel, opens wherever the platform feels like putting it.

Two routes dock a panel without this function seeing it: the *Dock all*
command, which edits the layout directly, and a
[`super::floatdrag`] drop, which [`Dock::show`] applies **earlier in
the same frame**. Both leave the panel out of the float list this
function draws from, which is why the sweep reads the list of windows
opened rather than the list of windows about to be drawn.

The second frame here takes the empty-float-list early return, so this
also pins the sweep to a position before it rather than after.
