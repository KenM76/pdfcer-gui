# `canvas::zoom` — the anchor rule, decided once, and the five paths that route through it

## The rule

> **A zoom holds one page point still, and that point is where the operator
> is looking: the pointer when it is over the canvas, the centre of the
> viewport when it is not. A zoom that *frames* something — a selection, a
> marquee'd region — holds that thing's centre still, at the centre of the
> viewport.**

It lives in [`anchor_point`] and in nothing else, and every zoom in the
product goes through it:

| path | anchor | how it gets here |
|---|---|---|
| Ctrl+wheel | the pointer | [`arm_anchor`], from `canvas::show` |
| Ctrl+Plus / Ctrl+Minus | pointer, else viewport centre | [`arm_for_actions`], or [`zoom_step`] |
| Ctrl+0 (actual size) | pointer, else viewport centre | [`arm_for_actions`], or [`zoom_step`] |
| Zoom to selection | the selection's centre → viewport centre | [`zoom_to_selection`] |
| Marquee zoom to region | the region's centre → viewport centre | [`zoom_to_rect`] |

`FEATURES.md` records the discrete commands as deferred *"so the rule is
decided once for all four"*. This module is that decision, and it is
deliberately a *rule about the operator's attention* rather than a rule
about arithmetic:

* **The pointer wins when it is over the canvas.** It is the only evidence
  the application has about what the operator is looking at, it is what the
  wheel path already honours (measured under 0.01 px of drift), and a
  keyboard zoom that ignored a pointer resting on the detail being
  inspected would behave differently from the wheel for no reason the
  operator could see.
* **The viewport centre when it is not.** A Ctrl+Plus pressed with the
  pointer parked over the Objects panel, off-window, or in the ribbon must
  not zoom about a point outside the page. The centre of what is on screen
  is the only other honest candidate; the page's **top-left** — today's
  behaviour, and the defect — is not a candidate at all, because it is
  where nobody is looking.
* **Framing is the same solve with a different target.** "Zoom to this
  rect" is not a different feature from "zoom about this point"; it is
  *put this point at the centre* instead of *put this point back where it
  was*. [`crate::canvas::geometry::offset_holding_anchor_at`] is the half
  they share.

## Why the anchor is a CANVAS-space point and not a screen position

Because a screen position stops naming the thing it named the instant the
zoom lands — which is the same reason [`crate::canvas::selection`] holds
identity rather than coordinates. `frac`, the fraction of the page's drawn
size that [`crate::app::state::ZoomAnchor`] carries, is
`canvas_point / extent` and therefore **independent of the zoom**: it can
be computed before the new zoom is known, which is exactly the situation
every one of these commands is in.

## The two-frame handshake, and the gate that makes it work for commands

`ZoomAnchor`'s own docs explain why it spans two frames: *"the new zoom is
not known when the wheel is seen … recording the inputs and solving later
avoids predicting a clamp we do not control."* The wheel is seen **during**
`canvas::show`, so the anchor it records is always consumed on the frame
after, by which time the action has been applied and the page's real drawn
size is known.

A **command** is not seen during `show`. Keyboard chords are dispatched at
step 1a of the frame, the ribbon at 1b, the status bar at 1b² — all *before*
the canvas draws — while the zoom action they raise is applied at step 3,
*after*. An anchor armed at step 1a and consumed unconditionally at the top
of `show` would therefore be spent on a frame that still shows the old zoom:
`display_after == display_before`, the solve is a no-op, and the anchor is
gone before the zoom it was for ever lands. That is precisely how a
plausible implementation of this feature ships doing nothing.

So consumption is gated by [`anchor_step`]: **an anchor is solved only on a
frame whose drawn page size differs from the size recorded when it was
armed** — i.e. only once the zoom has actually landed. One frame of grace is
allowed for the action to be applied, and an anchor still unspent after that
is *dropped* rather than held, because a zoom that never changed the page
size (Ctrl+Plus already at the raster ceiling) has nothing to re-anchor, and
an anchor left pending would be spent much later on an unrelated layout
change — a page step, a window resize under a fit mode — as a visible jump.

## Where the per-frame geometry comes from

An entry point called from the command dispatcher has no `Ui`, no page
rect, no viewport and no scroll offset. [`CanvasFrame`] is the canvas's own
record of those, written at the end of every `show` and read here. It is
*last drawn frame's* geometry, which is the correct "before" state in both
call orders — before `show`, the layout has not changed since; after `show`
but before the actions are applied, it **is** this frame's geometry.

## Item notes

### `fn trace_outcome`

Not de-duplicated: two identical zoom commands are two events, and a gate
that silenced the second would make a harness unable to tell a command that
ran twice from one that ran once.

### `fn framing_puts_the_regions_centre_in_the_middle_of_the_viewport`

Asserted as the outcome — where the anchored point ends up on screen —
rather than as an offset, so it checks the framing rather than the code
agreeing with itself.

### `fn a_region_at_the_page_corner_can_be_centred_because_the_pasteboard_is_there`

It used to assert the opposite: that framing a region hard against the
page's top-left saturated at offset zero, *"there is no page to the
left of or above the origin to scroll to"*, and the operator simply saw
it off-centre. That was true when the scroll content was the page and
nothing else.

**O23 made it false on purpose.** The operator asked for exactly this:

> *"I should also be able to move the view of the corner of the page to
> the center of the screen, or even all the way vertically to the
> opposite corner if I want to."*

The pasteboard is a viewport of slack on every side, so there IS
somewhere above and to the left to scroll to, and the solve's negative
page-local offset is a legitimate position rather than an over-range
one to be truncated.

It stayed asserting saturation for a while after the pasteboard
landed, because `geometry::zoom_anchor_offset` was still clamping to
the page's own range — which is `OPERATOR_REQUESTS.md` O24e, the zoom
that threw away whatever the operator had panned to. A test that pins
last year's constraint is how a stale clamp survives a feature
designed to remove it.

### `fn the_discrete_zooms_are_recognised_and_the_wheel_is_not`

The predicate [`arm_for_actions`] funnels on. A zoom action missing
from it is a command that silently keeps the old top-left anchoring —
which is the defect Phase 3.1 exists to close, reappearing one variant
at a time.

### `fn an_empty_selection_offers_no_bounds_to_frame`

The wiring above turns this `None` into [`ZoomOutcome::NoBounds`] and
raises no action; what is pinned here is that the `None` is real, i.e.
that the decline is reachable rather than a branch nothing can enter.

### `fn a_plan_past_the_ceiling_carries_the_clamped_scale_as_well_as_the_asked_one`

The second is what reaches `Action::ZoomTo`, so the status bar's
readout states the truth on the same frame — see
[`ZoomOutcome::ceiling_changed_the_answer`] on why that *is* the
ceiling's report rather than a substitute for one.
