# `pdfcer-gui/render/ink`

`render::ink` — **is there anything in this raster?**

One function, and it exists because a check could not answer that question
and therefore answered a different one.

# The gap this closes

`ui-verify`'s `the_page_still_renders_at_every_decade_of_zoom` photographs
the window at each rung of a zoom climb and asserts the canvas is not
near-uniform. That assertion is correct and it is the oracle that caught
defect **O174** — on the operator's `/Rotate 270` sheet the canvas really
was blank white at 2,025 %, because `render::region` was handing the engine
a rectangle in canvas space where the engine documents PDF user space.

It is also, on its own, **unfalsifiable in one direction.** A near-uniform
canvas has two possible causes and the window cannot distinguish them:

| world | what happened | verdict |
|---|---|---|
| the shell lost the raster | the engine drew ink, the shell put nothing on screen | a defect |
| the paper is blank | the engine drew a blank rectangle, faithfully | not a defect |

At 5,000 % a viewport is roughly a fifth of a point across, and most
fifth-of-a-point squares of a CAD drawing contain nothing whatsoever. So on
any real document the second world is the *common* one at depth, and a check
that cannot see it reports it as the first.


# What the trace can say now

[`sampled_tone_count`] runs on every completed raster and its result is the
`ink=` field of `render-async-done`. With it:

* near-uniform canvas **and** `ink=1` → the document is blank there. Report
  it; do not fail.
* near-uniform canvas **and** `ink=30` → the engine produced a picture and
  the shell did not show it. That is the defect O174 was.

Note what is *not* claimed: this says nothing about whether the rectangle
requested was the *right* rectangle. A shell asking for the wrong blank
square still reads `ink=1`. That question belongs to `render::region`'s
calibration against `pdfcer_render::region_base_geometry_of`, which is where
O174 was actually caught, and the two instruments are deliberately
independent of each other.
