# `render::ceiling` — the zoom ceiling this document *taught* the shell


> *"If this error is caused by some other limitation that will always
> happen, zoom should stop at the limit and not end up showing an error —
> the canvas will just stop zooming in and can still function. The error can
> still be shown on the bottom bar so the user has some idea as to why
> zooming stopped short of 1 trillion percent."*

Four separable requirements, and it is worth naming them separately because
three of them are *not* about rendering at all:

1. **Stop at the limit.** The zoom ceiling must come down to where the page
   can actually be drawn — [`RasterCeiling`], this file.
2. **Do not show an error.** The canvas must keep a picture, which follows
   from (1): if the zoom never goes past the wall, no render is ever
   refused, so there is nothing to paint a sentence about.
3. **Still function.** Panning, selecting, editing all keep working,
   because the zoom was clamped rather than the canvas disabled.
4. **Say why, on the bottom bar.** `crate::app::status::rasterstop`.

## Why the ceiling has to be LEARNED rather than derived

This project already has two *derived* ceilings and they are both in
[`crate::viewer::ceiling`]: the whole-page pixmap limit
(`max_zoom_for_page`, arithmetic on the page's `/MediaBox` and
`MAX_PIXMAP_EDGE`) and the `f32` positioning limit
(`SUB_PIXEL_CONTENT_EXTENT`). Both are closed-form, both are exact, and
neither is the wall the operator met.

The wall he met is inside `tiny-skia`, and it is **content-dependent**. The
engine's own measurement, quoted in `render::worker`'s `RasterizerLimit`
arm: an E-size sheet failed at scale **284,964** where a business card
reached **8,053,069** — a factor of 28 between two pages, decided by the
magnitude of the coordinates in their content streams and the size of the
intermediate buffers those produce. There is no expression of the page's
metadata that predicts it. The engine cannot predict it either; it catches
the panic and reports the scale that failed.

⇒ So the only honest source of the number is **the refusal itself**. One
render fails, the shell writes down the scale, and that page never goes
that far again.

## The property that makes one observation enough

Both raster refusals are overflows of a product of *the page's own extent*
and *the scale*. They are therefore **monotonic in the scale**: a page that
refused at `s` refuses at every scale above `s`. That single fact is what
turns one failure into a permanent ceiling instead of a guess — and it is
recorded on [`crate::render::worker::RefusalKind::BeyondRaster`] as the
variant's defining property, because it is the reason that variant exists
apart from `Other`.

## Why the ceiling is BELOW the scale that failed, and by how much

A ceiling *at* the failing scale would be a ceiling the page cannot draw
at: the refusal happened there. So the learned value is backed off by
[`BACKOFF`].

The back-off is 0.75 rather than something finer, and the reasoning is the
shape of the failure rather than a taste for round numbers. `BadRasterSize`
has a sharp boundary (a pixel count crossing `MAX_PIXMAP_EDGE`) and 0.99
would be enough for it. `RasterizerLimit` does not: it is a `usize` index
computed from a path's transformed coordinates, and the engine's reply
called the panic text *"third-party text and explicitly not a contract"* —
which means the shell cannot know how far below the first observed failure
the last *success* lies. A quarter of an order of magnitude is about two
wheel notches at these magnitudes, invisible against a zoom of 28 million
percent, and it buys a margin no arithmetic here can justify precisely.

And it does not have to be right, only safe-in-the-limit: the ceiling
**ratchets**. [`RasterCeiling::learn`] keeps the minimum, so if 0.75 of the
first failure still refuses, the second refusal lowers it again. The
sequence converges downward and costs one dead render per step — which is
why a conservative factor is preferred to an optimistic one, but why
neither can be wrong for long.

## Why it is per page and keyed on the page's epoch

Per page, because the limit is 28× different between two pages of the same
document (above). Per **epoch** — `crate::app::state::pageepoch::PageEpochs`
— because an edit can change the content that caused the overflow: deleting
the one enormous path raises the wall, and a remembered ceiling would then
be a magnification limit the document no longer has. A stale entry is
discarded rather than cleared, for the reason
`crate::app::actions::last_edit_disclosure` gives about its own epoch
comparison: *state that must be cleared is state that will one day be shown
against the wrong document.*

## What this module deliberately does NOT do

It does not touch the zoom. Learning is a record; *applying* the record is
[`crate::viewer::zoom_ceiling`]'s job, and pulling the current zoom back to
it is `crate::render::settle::absorb`'s, at the one point a refusal is
absorbed.
Keeping the three apart is what lets the arithmetic be unit-tested with no
document, no renderer and no frame.

## Item notes

### `fn a_page_that_has_never_refused_has_no_learned_ceiling`

The load-bearing half of this is the `None`: every document the operator
opens is in this state, and a ceiling that defaulted to a number would
cap a zoom no measurement has anything to say about.

### `fn the_ceiling_only_ever_ratchets_down`

All three in one test because they are one property — `learn` keeps the
minimum — and separating them would let a change satisfy two and break
the third.

The third clause is the one most likely to be got wrong and the most
consequential: a page can be refused at a *larger* scale than one
already learned (a render ordered before the ceiling took effect, or a
stale request landing late), and raising the ceiling on that evidence
would undo the correction the operator is standing at.

### `fn a_degenerate_scale_teaches_nothing`

A non-finite or non-positive scale cannot come from a real render, but a
ceiling of zero or NaN would make the page unzoomable rather than
bounded — a far worse failure than the one being guarded against, and
one that would look like the document being broken.

**A plausibly SMALL scale is deliberately not rejected here**, and the
reason is worth stating because it looks like a hole. A raster scale
below 1.0 is an ordinary render of a page zoomed out below 100 %, so a
floor in this function would reject real measurements. The case that
would be catastrophic — `BadRasterSize` firing because the pixmap is
*empty* rather than too large, which happens at a tiny scale and would
pin the zoom near zero — is excluded one layer up, where the width and
height are actually known: see `crate::render::worker`'s
`BadRasterSize` arm, which categorises an empty pixmap as
`RefusalKind::Other` precisely so it can never arrive here.

### `fn a_refusal_on_one_page_says_nothing_about_another`

Stated as a test because the alternative is cheap, tempting and wrong:
one ceiling for the document would be correct only if every page hit the
rasterizer's wall at the same scale, and the engine's own measurement
says two pages of one file differ by a factor of 28. A document-wide
ceiling learned from the E-size sheet would cap the business card at
3.5 % of where it can actually be drawn.
