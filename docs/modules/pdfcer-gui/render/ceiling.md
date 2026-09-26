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
