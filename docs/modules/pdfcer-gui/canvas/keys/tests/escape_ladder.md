# The Escape ladder, enumerated

[`super::super::escape`] — the function, not the file next door — is an
ordered list of `if let` arms, and the only way to hold an ordered list in
place is to enumerate it: every rung needs a case that **reaches** it and a
case that proves the rung above **did not swallow** it. That is why twelve
tests assert one short function, and why they are their own file rather
than a section of [`super`].

## The rungs, in the order Escape meets them

| Rung | Reached by | Protected from above by |
|---|---|---|
| a live drag spends the press | `an_escape_spent_on_a_drag_leaves_the_rung_alone` | — it is the top |
| a markup tool is put down | `escape_retires_the_markup_tool_before_the_region_zoom` | `an_escape_spent_on_a_markup_drag_leaves_the_tool_armed` |
| an armed region zoom retires | `escape_retires_an_armed_region_zoom_before_it_touches_the_ladder` | `an_escape_spent_on_a_drag_leaves_the_armed_zoom_alone` |
| the selection ascends a rung | `escape_ascends_a_rung_and_raises_no_action` | `escape_reaches_the_ladder_again_once_nothing_is_armed` |

The in-progress constructions — a guide drag, a circle fit, a vertex run —
each get a pair of their own, because each is abandoned *before* the rung
below it and a second press then reaches that rung.

## Not to be confused with `canvas::escape`

That module is the keyboard route **out of a canvas that drew nothing**,
and has no rungs. This one is the Escape key's precedence over the canvas's
claimants on a frame that drew normally. The two never interact; the shared
word is the key's name.

## What every case here passes, and why

`targets: None` with `model_attempted: true` — no decomposition, and the
frame asked for one — which is what lets these run without opening a file.
`page: None`, which is honest: no assertion here presses an arrow, and
[`keys::Keys::page`] exists for the nudge alone. `PickFilter::all()`, which
is what a shell that has never touched the filter hands over. Those
statements are true **of this file**; [`super`]'s header no longer claims
them of the Delete side, where a Tab case and an arrow case both exist.

## Item notes

### `fn escape_retires_an_armed_region_zoom_before_it_touches_the_ladder`

The rule this must not break is already in the file above: *"there is
already an Escape rule that must not both cancel a drag and ascend a
selection rung."* Phase 3.4 inserts a third claimant between them, so
the same discipline is asserted for the new pair: an operator who arms
a marquee zoom and changes their mind gets out of the tool **and keeps
the part they were working in**.

### `fn an_escape_spent_on_a_drag_leaves_the_armed_zoom_alone`

The one-press-one-effect rule runs in both directions: a cancelled
zoom-marquee drag must not *also* disarm the tool, or an operator who
mis-drags a zoom box has to re-arm it before they can try again.

### `fn escape_retires_the_markup_tool_before_the_region_zoom`

Both are armed at once deliberately, for the reason the guide-versus-zoom
test below states: asserting the markup tool is retired would pass on a
build that retired everything. Asserting the zoom **survives** is what
makes it a precedence test.

### `fn an_escape_spent_on_a_markup_drag_leaves_the_tool_armed`

The sharpest form of one-press-one-effect for this feature: an operator
who mis-drags a rectangle and cancels it is still holding the rectangle
tool, so their next drag draws a rectangle. Retiring the tool as well
would make every abandoned drag cost a trip back to the ribbon.

### `fn escape_abandons_a_guide_drag_before_it_touches_the_region_zoom`

The tie-break the precedence table states: retire the most transient
thing first. Both are "in flight", and the guide is the one following
the pointer *this frame* while the zoom is waiting for a drag that has
not started.

Both are armed at once deliberately. Asserting the guide is cancelled
would pass on a build that cancelled everything; asserting the zoom
SURVIVES is what makes it a precedence test rather than a "something
happened" test.

### `fn escape_abandons_a_circle_fit_before_it_puts_the_measure_tool_down`

The radius/diameter gesture has no natural end, so a pick set can sit
there for as long as the operator keeps toggling arcs into it. That
makes the two-rung rule load-bearing rather than tidy: an operator who
has picked four arcs and catches a fifth by mistake presses Escape to
correct it and must find themselves still holding the tool, with the
set cleared — not back in the select tool with everything gone.

A region zoom is armed throughout, and asserting it **survives** both
presses is what makes this a precedence test rather than a "something
happened" test: a build that retired everything on the first press would
pass the first two assertions.

### `fn escape_abandons_a_vertex_run_before_it_puts_the_markup_tool_down`

Written as a near-copy of the circle-fit test above **on purpose**, and
the copy is the point rather than duplication: the two gestures have the
same problem (a run of clicks with no natural end), were given the same
answer (two endings, one commit path), and now share a rung — so a build
that got the precedence right for one and wrong for the other is exactly
what a near-copy catches and a shared helper would hide.

The operator's case is concrete: someone clicking out a polygon round a
detail catches a seventh corner by mistake, presses Escape to correct it,
and must find themselves **still holding the pen** with the run cleared —
not back in the select tool with everything gone and the tool to re-arm.

A region zoom is armed throughout, and asserting it **survives both
presses** is what makes this a precedence test rather than a "something
happened" test: a build that retired everything on the first press would
pass the first two assertions.

### `fn a_second_escape_retires_the_zoom_the_guide_drag_protected`

Without this, the test above would pass on a build where a guide drag
permanently swallowed Escape — which is a worse bug than the one being
fixed, because it would leave the operator unable to leave any tool.
