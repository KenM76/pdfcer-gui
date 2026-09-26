# `pdfcer-gui-base/renderworker/ink`

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

## Item notes

### `const TONE_CAP`

Nothing downstream cares whether a page has 90 tones or 9,000; the questions
are *"is it one?"* and *"is it clearly more than one?"*. Capping keeps the
working set small and the trace line short.

### `const TONE_SAMPLE_TARGET`

A region raster can be 16,383 px on a side — 268 million pixels — and this
runs on **every** completed render, including the ones arriving during a live
zoom. A full scan would be a frame-rate defect introduced by an instrument,
which is a poor trade for a number whose only job is to separate blank from
not-blank.

### `fn one_dark_row_on_white_paper_is_not_uniform`

This is the assertion that makes the field useful rather than merely
present: the interesting case is not a busy drawing, it is *one line on
otherwise empty paper*, which is what a deep-zoom viewport of a CAD sheet
actually contains when it contains anything at all.

### `fn the_sample_is_bounded_however_large_the_raster`

Not a timing test — those are flaky — but a statement of the invariant
the stride exists to hold: a raster two orders of magnitude larger than
the sample target still examines about the sample target's worth of
pixels. Written as an arithmetic assertion on the stride itself so it
cannot pass by accident on a fast machine.

### `fn sampled_tone_count`

Distinct exact RGB triples, saturating at [`TONE_CAP`], over a stride chosen
so at most about [`TONE_SAMPLE_TARGET`] pixels are examined.

# Why exact triples rather than perceptual buckets

Antialiasing means a single black hairline on white paper produces dozens of
distinct greys, so counting exact triples is if anything **more** sensitive
to faint ink than bucketing would be. For a blank-detector that is the
direction to err: a false "there is ink here" makes a check ask a further
question, while a false "this is blank" makes it excuse a real defect.

# Why the answer is never zero

An empty pixmap cannot occur — the renderer refuses a zero-area region — and
returning `0` would make *"no pixels"* and *"one colour"* indistinguishable
in the trace, which is the precise ambiguity this function exists to remove.
The empty case therefore returns `1` and says so here rather than leaving a
reader to infer it.

# Cost

Bounded: `pixels.len() / TONE_SAMPLE_TARGET` sets the stride, so the loop
runs at most `TONE_SAMPLE_TARGET` times regardless of raster size, and it
exits early the moment [`TONE_CAP`] distinct tones have been seen — which on
any inked region happens within the first few hundred pixels.
