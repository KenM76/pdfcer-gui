# `raster_wall::backout` — part C: the way back down

## The clause

`OPERATOR_REQUESTS.md` **O220**, in his own words:

> *"when the error occurs it prevents me from pressing ctrl and using the
> zoom wheel to zoom back out — I have to click the zoom out control on the
> bottom bar."*

and, from the same report, what he expects instead: *"zoom out and it will
draw again."*

## Why this is a separate part and not three more lines in part B

**Every `scroll_at_held` in this check's other two parts rolls direction
`1`.** Parts A and B climb. A check that only ever pushes a control in one
direction cannot see a handler that is dead in the other, and *"it prevents
me from pressing ctrl and using the zoom wheel to zoom back out"* names a
gesture being taken away **at the wall specifically** — which is a state no
other check stands in.

⚠ **This is not the only `-1` in the harness and must not be described as
one.** [`crate::checks::off_sheet`] wheels out of the `nothing-visible`
blank, and its own header calls itself *the only driven coverage of
`canvas::escape::offer` anywhere* — see "What this part does not reach"
below, which is where that claim is still true.

It reuses part B's climbed state rather than re-climbing, because reaching
the wall costs sixty-eight or more Ctrl+wheel notches on a 5.7 MB vector
drawing, and a second check that pays that again is minutes of wall clock
for a state part B is already standing in.

## The two states part B can leave, and why both are this part's subject

| part B's exit | what stands | what the gesture has to do |
|---|---|---|
| the learned ceiling held | the canvas is DRAWING, clamped | the first batch out must lower the zoom |
| `canvas-unavailable` | the canvas is drawing NOTHING | wheeling out must make it draw again |

The second is O220's literal state and the stronger measurement: it is the
one where both of `canvas::present::show_in`'s early returns sit above every
input handler in that function, so without `canvas::escape`'s hatch there is
no pointer gesture left at all. Part B treats it as NOT MEASURED —
correctly, since its own subject is the clamp — and hands it here.

⚠ **Neither state may be turned into a SKIP by this part.** Part B has
already measured something real by the time control reaches here; an `Err`
would discard its findings along with the whole check.

## The oracle, and why it is not a screenshot

`canvas` and `canvas-unavailable` share one trace slot, so exactly one of
them is the standing verdict and [`unavailable_now`] decides which by line
number. That yields both halves of the operator's sentence from one read:

* the zoom fell — `canvas … zoom=` is lower than where he was stuck;
* it draws again — a `canvas` line now stands *above* the
  `canvas-unavailable` line that was the verdict when this part began.

While the canvas is blank, [`latest_canvas`] returns a **fossil**: the last
frame that drew, which is the zoom the operator last had on screen. That is
deliberately the baseline. The view's own zoom went on climbing above it
while the screen was blank, so requiring the zoom to fall *below the fossil*
asserts the wheel brought him back past the point it stopped drawing — not
merely that some number moved.

## What this part must not do

⚠ **It must never touch the status bar's zoom-out control.** That control is
the workaround the operator was left with, so a check that reaches for it
measures the workaround and reports the defect fixed. The only gesture here
is Ctrl+wheel at the canvas.

## What this part does not reach, and which arm any given run measured

**Which of the two rows above a run lands on is not this part's
choice — it is whatever part B left — so a PASS here is not a claim about
both.** The note this part writes names the arm in as many words, and that
note is the only place the distinction is legible; read it before quoting a
green run at anything.

Since O218's ceiling absorbs a `BeyondRaster` refusal into a learned limit,
the dense drawing's climb now ends **drawing** rather than blank, so the
ordinary path is the one usually exercised and `canvas::escape::offer` is
not on it at all. ⇒ **O220's literal precondition — *"when the error
occurs"*, i.e. `reason=render-failed` — has no driven coverage from here**;
`off_sheet` covers the hatch's other arm, `nothing-visible`, and nothing
covers `render-failed`. The operator still reports reaching that state, so
it is reachable and this harness cannot yet reach it on demand; O221's
document-count dependence is the standing candidate for the lever.
