---
name: a-window-over-the-thing-it-describes-takes-that-things-gestures
description: When one gesture on an object works and the others do not, look for a layer on top of it before looking at the gesture router. An egui Area constrained back over its own anchor eats every press on that anchor, silently.
metadata:
  type: feedback
---

**A pop-up, tooltip or panel drawn over the object it belongs to is not a
cosmetic problem. It is an input surface, and the object underneath becomes
unreachable** — with no event, no decline, and nothing in the trace.

## The evidence, 2026-09-05

The first full driven sweep filed *"an annotation can be ROTATED and cannot be
MOVED or RESIZED"*. `dragging_a_markup_moves_it` and
`the_line_weight_switch_reaches_the_resize` both failed with **no line
containing `drag` anywhere in the trace**; `rotating_a_markup_turns_it` passed.
Both checks named `canvas::interact`'s fork, articulately, and the fork was
fine.

Two lines of the same trace:

```text
ui-rect name=canvas.selection-outline rect=[[464.0 464.5] - [551.7 550.2]]
ui-rect name=notepopup.window         rect=[[498.0 465.0] - [772.0 565.0]]
```

`canvas::notepopup` places its window to the **right** of its note. It did not
fit — the canvas ends at x=772 and the window is 274 pt wide — so
`Area::constrain_to` **slid it back**, onto the note. egui resolves interaction
on the topmost layer, so the canvas response never saw the press.

## The tells, in the order they should have been read

| tell | what it means |
|---|---|
| **one gesture on an object works and the rest do not** | something is covering part of the object. The gesture that survives is the one whose affordance is drawn *outside* the covered area — here the rotate handle, which sits above the box |
| a drag produces **no line at all**, not even a decline | it never reached the code you are reading. A refusal is a line; silence is a layer, a capability gate, or a press that landed somewhere else |
| a check names a mechanism confidently | ask what it *sampled*. Both checks here named the right file and the wrong subject |

## The second fault under the same symptom, because there were two

`gesture::press_kind`'s annotation arm was gated on `markup_body` —
*is the press inside the annotation's rect?* — and **a corner grip is centred
on the corner, so half of it is outside**. A press there answered
`Some(Grip::NorthEast)` with `markup_body == false`, fell through to
`caps.edit_content` (false in Review, the mode markup is authored in) and
vanished. Page content never had the bug: its branch matches on `grip`
directly. The two later-added arms copied the body test from the *move* they
were written for.

⇒ **Two independent faults produced one symptom.** Fixing either alone would
have left the sweep still red and the diagnosis still wrong.

## What to do

- **Placement: flip, never slide.** Preferred side → opposite side →
  below/above. Accept the preferred position only when the box it implies both
  clears the anchor *and* fits inside the clip — a position that does not fit is
  one the framework will move for you, in the one direction you cannot afford.
- Separate on **x alone** where possible, so the answer cannot depend on the
  window's own height. A surface whose position depends on its own size
  oscillates, invisibly to both unit tests and screenshots.
- Reserve the **outer** width. `Frame::popup`'s margin sits outside
  `set_max_width`; 260 pt of content drew a 274 pt window.
- **Gate a resize on the grip, not on the body**, and keep the two as separate
  facts — body means move, grip means resize. Widening the body test passes the
  new test and turns every corner drag into a translation.
