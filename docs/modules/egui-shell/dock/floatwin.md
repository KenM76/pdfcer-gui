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

### `const BODY_MARGIN_PTS`

The `Ui` a viewport callback receives is the window's root and nothing
pads it, so without this every control in the panel touches the frame.
A constant rather than a theme metric because it participates in nothing
that could feed back into it.

### `struct FloatFrameReport`

Deliberately a separate type from [`super::DockFrameReport`] rather
than more fields on it: the two are produced by two calls at two points
in the frame, and a single struct would have half its fields stale
whichever of the two a caller happened to read.

### `fn show_floating`

Call this from the application's top-level frame, beside its
dialogs — see the module header for why it cannot live inside
[`Dock::show`].

# What it does per open float, in order

1. Decides the window's position, honouring a remembered one only
   if it is still plausible ([`float::honour_position`]).
2. Opens or updates the viewport, asserting the position only on
   the frame the window opens.
3. Paints the background, because nothing else will.
4. Draws the header strip and offers its `Response` to the
   application's tab-menu handler — the same seam a tab uses.
5. Calls `body`.
6. Reads the window's geometry back and records it as an intent.
7. Applies every intent afterwards, in one place, exactly as
   [`Dock::show`] does.

### `fn split_header`

A pure function so the arithmetic can be asserted without a window, and
so the degenerate case has a named answer: a window too short to hold
both gives the header nothing and the body everything. **The body
wins**, because a header with no body is a window showing nothing at
all, whereas a body with no header is still a panel — and the operator
can still close it, because the OS window's close button is not drawn
by us.

### `fn viewport_id`

Derived from the panel id rather than counted: `ViewportId` is what
`egui` keys the OS window on, so two panels sharing one would be two
panels in one window, and a counter would give a panel a different
window depending on what else happened to be floating when it was
floated.

Salted with a prefix so a panel called `"print"` cannot collide with an
application dialog of the same name — the two id spaces are independent
and neither knows about the other.
