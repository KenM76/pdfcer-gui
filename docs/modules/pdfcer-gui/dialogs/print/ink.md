# `dialogs::print::ink` — which parts of a rendered sheet actually carry ink

## The question this module exists to answer, and why it is asked HERE


> *"can you make it so the red pattern you put over the page if it is going
> to print beyond the printable borders is only over the areas that extend
> beyond the printable page? Our drawing get drawn 1:1 and the area that
> isn't printed is just empty border."*

The print preview hatches the **whole overhanging region** whenever the
placement reports a clip. `Placement::clipped` is a *geometric* verdict —
the page box exceeds the printable rectangle — and on a CAD sheet printed
1:1 the part that exceeds it is empty paper. So the hatch shouted about
losing something on every drawing, and nothing was being lost.

**That is a disclosure which is technically true and practically
false, which is the worst kind.** An operator who sees the same red band on
every 1:1 drawing learns to ignore it, and then does not see it on the one
sheet where the border really does have a title block in it. A warning that
fires on the common harmless case trains the operator out of reading it.

## Why the raster, and NOT an engine content bounding box

`OPERATOR_REQUESTS.md`'s O113 row offers two routes and leans toward the
first: an engine verb returning a page's ink extent (a real content bbox,
not the `/MediaBox`), described there as *"the honest general answer"*; or
sampling the preview raster this shell has already rendered, described as
*"cheaper … and approximate at the edges."* The row has the ranking
backwards, and it is worth stating why rather than merely choosing.

**A content bounding box is a RECTANGLE, and a rectangle is conservative in
exactly the wrong direction for this question.** Ink at two opposite
corners of a sheet produces a bbox that covers everything between them,
including an overhang band with no ink anywhere in it. The engine verb
would therefore report *"content reaches into the overhang"* on the very
drawing shape O113 is about — a title block at the bottom-right and a
revision block at the top-right make a bbox spanning the whole right edge,
whether or not the strip that will actually be cropped holds a single
stroke. It would be a *different* proxy for the operator's question, not a
better answer to it.

[`super::preview::texture_for`] already calls
`pdfcer_render::render_page_with_view` and holds the resulting
`rendered.pixmap` — **the very pixels that are about to print**, rendered
through the same `RenderOptions` the spooler uses (see
[`super::render_options`]). The question *"is anything lost?"* is a
question about those pixels. Sampling them answers it **directly** rather
than by proxy, and the "approximate at the edges" caveat is bounded and
stated: see [`CELLS_LONG_SIDE`] and [`InkMask::ink_extent`], which are
approximate only in the direction of hatching **slightly more** than is
strictly lost, never less.

A real engine content-extent verb would still be worth having for other
questions (auto-crop, fit-to-content, trim detection). It is not the better
answer to *this* one.

## What "ink" turned out to MEAN, measured rather than assumed

This was the one thing in the change that could be silently backwards, so
it was verified against real rasters before a line of the algorithm was
written. Two measurements, both taken by rendering through the same entry
point the preview uses, `render_page_with_view` with the default
`RenderOptions`:

**1. The page is composited on OPAQUE WHITE, so alpha says nothing.**
`pdfcer-render` starts the page group fully transparent (ISO 32000-1
§11.4.7 — the page group is *isolated*, so its initial backdrop is `U`) and
then adds the paper in one final pass, `flatten_page_group_over_white`,
which sets **every** pixel's alpha to 255. `RenderOptions::backdrop`
defaults to `PageBackdrop::White`, and `super::render_options` never calls
`with_backdrop`, so the preview always gets the white-composited result.

Measured on three documents (`src/app/assets/blank-a4.pdf`,
`fixtures/a1-titleblock.pdf`, `fixtures/paragraph.pdf`): **`non_opaque = 0`
on all three**, every pixel alpha 255. A transparency test would therefore
find no ink anywhere and hatch **nothing, ever** — the failure mode is
silent, and it would look like the fix working perfectly.

**2. Blank paper is NOT reliably `(255, 255, 255)`, and this is the
finding that sets the threshold.** The blank A4 template renders as
`(255, 255, 255, 255)` in every pixel, as expected. But
`fixtures/a1-titleblock.pdf` — a large-format CAD sheet, i.e. exactly the
document population this operator prints — renders its paper as
**`(249, 249, 249)`**: 236,443 of its 250,916 pixels, against only 7,246
at pure white. The sheet carries a near-white background fill.

⇒ A naive `min(R, G, B) < 255` ink test classifies **97% of that sheet as
ink**, including every square millimetre of its empty border, and hatches
the entire overhang. It would reproduce O113's defect exactly while
appearing to fix it. [`INK_MAX_LEVEL`] exists because of that number, and
its doc comment carries the measurement.

## Rule 4 — this is disclosure, and it stays on the preview

`pdfce_FeatureRequests/README.md` rule 4, clause 2: *"No badge, tint, red
flag, dashed outline or 'provisional' layer drawn into the page view."*
The hatch is painted by [`super::preview::paint`] into the **print
dialog's preview canvas**, over the diagram of a piece of paper. Nothing in
this module or its caller touches a `RenderOptions`, an `EditSession`, a
staging buffer or a saved byte; the pixmap it reads is borrowed immutably
and dropped. **The page as rendered for printing is bit-for-bit unchanged
by this change** — the one-line test is that a screenshot of what prints is
identical before and after, and it is identical because the print path
never calls into here at all.

Making the hatch *narrower* also moves in the safe direction for clause 2:
there is strictly less non-content drawn over the operator's page than
before.

## Item notes

### `const CELLS_LONG_SIDE`

# What this number buys, and what it costs, as arithmetic rather than a
# guess

The mask is a downsample of the preview raster, whose longest side is
capped at `super::preview::MAX_SIDE_PX` = 2200 px. At 256 cells on the long
side, one cell is at most `2200 / 256 ≈ 8.6` raster pixels on a side.

**Memory.** The grid is `Vec<bool>`, one byte per cell. The worst case is a
square page, `256 × 256 = 65,536` cells = **64 KiB**. The raster it
describes is up to `2200 × 2200 × 4` bytes = **19.3 MiB**, so the mask
costs **0.33%** of the picture it summarises, and is built once per raster
rather than once per frame. A bit-packed grid would take it to 8 KiB and is
deliberately not done: 64 KiB against 19 MiB is not a cost worth trading
legibility for, and this module's whole value is that a reader can check
its arithmetic.

**Spatial resolution, in the units the operator cares about.** On a US
Letter sheet (612 × 792 pt) the long side is 792 pt, so a cell is
`792 / 256 ≈ 3.1 pt ≈ 1.1 mm`. On an ANSI E sheet (2448 × 3168 pt) a cell
is `3168 / 256 ≈ 12.4 pt ≈ 4.4 mm`. Both are far finer than any margin
decision an operator makes from a preview, and the error is **always in the
direction of hatching slightly more** than is strictly lost — see
[`InkMask::from_rgba_premultiplied`], where a cell is inked if *any* pixel
in it is.

**Why not simply test the raster pixels directly and skip the mask?** The
overhang band would be re-scanned on every frame, at up to a few million
pixels a frame, for an answer that cannot change until the raster does. The
mask is computed exactly once per raster and cached beside it under the
same key — see `super::preview::PreviewKey` and the field it keys.

### `fn is_ink`

See [`INK_MAX_LEVEL`] for the measurement behind the threshold, and
[`InkMask::from_rgba_premultiplied`] for why alpha is checked at all when
every pixel the preview produces today is opaque.

`min(R, G, B)` rather than a luminance: a saturated colour has a low
minimum channel even when its luminance is high — a pure yellow
`(255, 255, 0)` is 89% luminance and unmistakably ink — so the minimum is
what catches coloured linework, which is most of what a CAD sheet is made
of. A luminance test would have to be tuned per hue to see the same marks.

### `fn raster`

Opaque white is what `render_page_with_view` actually produces (this
module's header records the measurement), so a fixture built any other
way would be testing a raster the preview never sees.

### `fn a_blank_overhang_is_not_hatched_even_though_the_page_is_full_of_ink`

> *"Our drawing get drawn 1:1 and the area that isn't printed is just
> empty border."*

The page carries plenty of ink — a title block in the middle — and the
placement would report a clip, because the page box does exceed the
printable rectangle. The band that will actually be cropped is blank,
so nothing is hatched. This is the request, in one assertion.

### `fn one_inked_spot_in_the_overhang_hatches_that_spot_and_no_more`

The same page as the test above with a single small mark added out in
the border — a stray revision stamp, a pdf-dimension leader that ran
past the frame (rule 15: CAD-exported page content, not a ce dimension
pdfcer authored),
the corner of a title block. The extent returned must cover that mark
and must not spread to the rest of the band.

### `fn near_white_cad_paper_is_not_ink`

`fixtures/a1-titleblock.pdf` renders its paper as `(249, 249, 249)` —
236,443 of 250,916 pixels. Under the intuitive `< 255` test that whole
sheet, empty border included, is ink and the hatch covers everything:
O113's defect, reintroduced by a wrong definition of the word.

The measured value is written into the fixture rather than described,
so if either the constant or the exporter's paper colour moves, this
says which.

### `fn ink_is_decided_by_colour_and_not_by_alpha`

Every pixel the preview renders is alpha 255 — measured on three
documents. A mask built on alpha would return `None` for every region of
every page, hatch nothing ever, and look exactly like the fix working.
This pins that the test used is the colour one.

### `fn the_extent_never_stops_short_of_the_ink`

A pixel is an area. Assigning it to the single cell its top-left corner
lands in leaves the far edge of that pixel outside the cell whenever the
grid does not divide the raster — so the reported extent stops short of
the ink by up to one pixel, in the direction that under-discloses a
loss. See [`InkMask::from_rgba_premultiplied`] for the measured case.

This sweeps a one-pixel mark across a whole row of a raster whose width
(400) is not a multiple of the grid (256), and asserts at every position
that the extent contains the pixel's **full** span. A top-left mapping
fails this at roughly a third of the positions.
