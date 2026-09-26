# `canvas::zoom` — the anchor rule, decided once, and the five paths that route through it

## ★ The rule

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

## ★ The two-frame handshake, and the gate that makes it work for commands

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
