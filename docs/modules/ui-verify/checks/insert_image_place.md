# `ui-verify/checks/insert_image_place`

`the_insert_window_steps_aside_so_you_can_point` — **press the button, the
window goes, click the page, the window comes back with your numbers in
it.**

# The request


> *"anything we are inserting like this should have an option in its
> dialogue box to place it with the mouse instead of by positional
> co-ordinates."*

## Why this needs a DRIVEN check and not only unit tests

Every piece of this arm is unit-tested and every piece passes in isolation:
`canvas::placing`'s arm/cancel/result cycle, `dialogs::placing`'s derived
`hidden`, `text::placing`'s sentences. What no unit test can see is the
**join** — that pressing a real button in a real window makes that window
stop being drawn, that a real click on the canvas is routed to the placement
arm rather than to the marquee underneath it, and that the window comes
back.

That is the shape this project has shipped broken before: every part tested,
the join untested, the join wrong. Eight green unit tests once sat under a
feature that did one of its fourteen steps.

## The oracle is the REGION'S ABSENCE, and that is deliberate

`dialogs::insert_image` publishes `insert-image.place` on every frame it
draws. While a placement is pending it draws nothing at all — its `show`
returns *still open* before the window is built — so the region stops being
declared.

⇒ *"The window stepped aside"* is observable as that region going, and *"it
came back"* as the region returning. Neither is an inference from silence:
step A confirms the region was there first, so an absence at step B cannot
be confused with a dialog that never opened. That confirmation is the
precaution this suite's own rule asks for — before asserting on an absent
line, ask what else happened.

## The sequence

| # | step | oracle |
|---|---|---|
| A | open the dialog with an image | `insert-image.place` declared |
| B | press *Place it on the page…* | `place-armed kind=Image`, and the region goes |
| C | click the page | `place-result kind=Image llx=… lly=…` |
| D | the window is back | `insert-image.place` declared again |

Step C asserts the **result**, not an inserted image, because the button
places nothing: it fills the numbers in and the operator still presses
Insert. Asserting an insert here would assert a different feature, and would
pass against a build that bypassed the dialog entirely.

## What this check deliberately does NOT drive: Escape

The other half of O66 is that Escape abandons a placement and brings the
window back. It is not driven here, and the reason is not laziness:

- `scale_switch`'s header records, with six runs of evidence, that **a
  keystroke is not a reliable harness primitive** in this shell. A step that
  fails half the time would make this check's real subject unreportable.
- The property is already unfalsifiable by construction. `hidden` is
  *derived* from the pending record, so whatever clears that record — the
  Escape claimant, a mode change, the document closing — un-hides the
  window. There is no per-route flag to forget, and `dialogs::placing`'s own
  test cancels through a module `PlaceHandoff` never calls and watches the
  window return.

⇒ A driven Escape would re-test a mechanism that cannot fail per route, at
the cost of a flaky step. Step D is the load-bearing observation and it *is*
driven.

## Item notes

### `const AT`

Away from the edges and away from the centre. A default placement already
sits near the middle, so a click there could pass against a build that
ignored the pointer completely.

### `const TOLERANCE_PT`

Generous against the real error and tight against the real defect. The
measured agreement is under half a point; one screen pixel at the fit zoom
this fixture opens at is about 3 pt, so a few points absorbs the click
quantisation. The mirror this check was written after is about 300 pt out,
and a centre-defaulting build is of the same order.
