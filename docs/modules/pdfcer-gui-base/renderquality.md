# `pdfcer-gui-base/renderquality`

## Item notes

### `fn every_quality_has_a_distinct_stable_token`

They are what the file holds, so two quality values sharing a token
would make one of them unreachable from a hand-edited file, and a token
that changed with a display name would reset everybody's preference on
upgrade.

### `enum RenderQuality`

# What "natural" is, and why this multiplies rather than replaces

`viewer::raster_scale` is `zoom × pixels_per_point`: one raster pixel per
*device* pixel, which is the scale at which a page is exactly as sharp as
the display can show and no sharper. That is the right default and it is
what [`RenderQuality::Normal`] means.

The two other values trade against it in opposite directions, and both are
real needs on the drawings this shell is for:

- **Faster** renders at 0.75× and lets the GPU upscale. On the benchmark
  CAD sheet — 5.6 MB of dense vector site plan — that is roughly half the
  pixels and therefore roughly half the rasterisation time, at the cost of
  softness that is most visible on the thin linework such a drawing is
  made of. An operator panning around a big sheet looking for something may
  well want it; an operator checking a dimension will not.
- **Sharper** renders at 1.5×. Pointless on most content and genuinely
  better on small text over a hairline grid, where a device pixel straddles
  two strokes and neither survives.

# Why three values and not a slider

Because the useful range is narrow and the middle of it is almost always
right. A slider invites an operator to spend attention tuning a number that
will not repay it, and — more practically — every intermediate value costs a
full re-raster of every visible page to evaluate.

### `const ALL`

Worst-to-best rather than best-to-worst, so the control reads left to
right as *less … more*, which is the direction a reader expects of a
quality scale. The default sits in the middle of it, where a three-way
control wants it.

### `fn key`

Stable across releases and deliberately not the display name: a display
name is operator copy and may be reworded or translated, and a file
whose keys moved when the wording did would silently reset everybody's
preference. Same rule `egui_shell::theme::Preset::key` follows.

### `fn from_key`

`None` rather than a default, so the loader can *report* an unreadable
value rather than silently substituting one — the per-key recovery
contract in the module header.

### `const MIN_SETTLE_MS`

Zero is excluded and that is a decision. A settle of zero means *rasterise
every intermediate value of a wheel gesture*, which on a dense CAD sheet is
dozens of full-page renders producing images nobody sees — the exact cost
the debounce exists to avoid. 20 ms is short enough to feel immediate and
long enough to swallow the burst of events one wheel notch produces.

### `const MAX_SETTLE_MS`

Beyond about a second the interim scaled texture stops reading as "still
settling" and starts reading as "stuck", which is a worse impression than
the CPU cost it saves.

### `const DEFAULT_SETTLE_MS`

150 ms, measured against real CAD sheets, and the same value
`crate::app::settle::ZOOM_SETTLE` compiles in. The two must agree: a
preferences file that names no settle has to debounce exactly as the
compiled-in deadline does, or the choice changes behaviour by existing.
