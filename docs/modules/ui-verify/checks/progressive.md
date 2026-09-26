# `ui-verify/checks/progressive`

`progressive` — **the page is never blank while a sharper one is on its way.**

# The report


> *"the screen should never be blank while waiting to render when zooming
> out - there should be at least a low resolution zoom of the newly panned
> or zoomed out area instead of just remaining blank while the higher
> definition render occurs."*

# Why this is not [`super::pan_refresh`], which already passes

That check exists for the operator's **previous** report on the same
gesture — *"it doesn't always render the new exposed area"* (O25) — and it
was a real defect with a real fix: the region was in the raster cache key
and in no staleness test, so a pan requested no render at all.

It asserts the canvas is not blank **after a render completes**. That is the
right assertion for that defect and it is blind to this one by construction:
what is being reported now is the **interval before** the render lands. The
area does render; the complaint is what is on screen while it does.

So the two checks differ in exactly one respect and it is the whole subject:
this one captures **immediately after the gesture**, without waiting for a
raster, and requires the canvas to be showing something anyway.

# Why this is measured from the TRACE and not from a screenshot

This project's standing rule is that layout and clipping defects have
exactly one oracle and it is a rendered screenshot. **This is not a layout
defect, it is a timing one**, and the interval being measured is shorter
than a window capture takes.

Three camera-based versions were built and driven before this one, and all
three were unable to fail:

1. *"is the canvas near-uniform?"* — passed, because the area a raster does
   not cover is drawn as the page's own white and a CAD sheet is ~90 % white
   anyway. Every band of every capture measured the same whether it held
   content or not.
2. *"count ink during, compare with ink once settled"* — better, and still
   passed: the two captures came back with **identical** counts every time,
   because the raster landed before the shutter.
3. The same, with the stale-texture path **deliberately sabotaged** so that
   the page had to blank. Still passed. That is the moment the method was
   abandoned rather than tuned: a check that cannot fail is not evidence.

What the application knows and a camera does not is whether the pixels it
drew are a picture of the whole visible area or of a fraction of it. It
publishes that ratio as `canvas-coverage`, every frame it changes, and the
trace is a **record** rather than a race — so the minimum over the frames
after a gesture can be read afterwards, exactly, with no shutter to beat.

Driven on a real sheet, zooming out from 3590 % held `covered=0.000` for
about twenty frames. That is the operator's blank, quantified.

# Phases

| Phase | Does | Expected |
|---|---|---|
| A | zoom in past the whole-page tier | region rendering is in force |
| B | pan a long way, capture **at once** | the canvas is not near-uniform |
| C | zoom out, capture **at once** | the canvas is not near-uniform |

## Item notes

### `const MIN_COVERED`

Not 100 %. A gesture legitimately passes through frames where the held
picture is being re-placed, and demanding perfection would fail on rounding.
Half the view is far above anything a working stand-in produces and far
below the measured failure, which was **0.000**.

### `const CLIMB_TO`

Deep enough that a raster is slow — which is what creates the interval
under test — and no deeper. The first run climbed to 3590 %% and found 52 ink
pixels on the whole canvas: at that magnification a technical drawing is
mostly the space BETWEEN lines, so there was nothing whose disappearance
could be measured. A check about losing sight of the drawing needs the
drawing in sight.

### `enum Gesture`

An enum rather than a closure because the two arms need different driver
calls with different argument shapes, and a boxed closure per gesture would
be ceremony around a two-case match.

### `fn coverage_samples`

`trace_on_change` collapses runs of identical values, so this is the
sequence of DISTINCT states the canvas passed through rather than one entry
per frame. That is what makes a minimum over it meaningful: a blank held for
twenty frames appears once, and so does a blank held for one.

### `fn ink`

## Why ink and not uniformity, and what the first version got wrong


So "blank" here does not mean "uniform", it means **the drawing is not
there**. Counting ink is what distinguishes them: a band of a CAD sheet with
its lines missing has near-zero ink, and the same band with its lines has
thousands of pixels of it, on the same white background.
