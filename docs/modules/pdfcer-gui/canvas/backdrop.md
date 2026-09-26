# `canvas::backdrop` — the low-resolution page under the sharp one


> *"the screen should never be blank while waiting to render when zooming
> out — there should be at least a low resolution zoom of the newly panned
> or zoomed out area instead of just remaining blank while the higher
> definition render occurs."*

Two things live here and they are two halves of one subject: the picture
that stops the page going blank, and the number that makes its absence
falsifiable.

## ★★★ The defect, measured before anything was built

Above the pixmap ceiling a raster is a picture of the **visible region**
rather than of the page, and `canvas::mod` places it at *its own* region's
rect — which is right, and is what stops the page lurching during a pan.
What it cannot do is cover ground the region never included. Zoom out from
deep zoom and the held picture shrinks to a speck of a big sheet; pan far
enough and it leaves the window altogether.

Driven on a real CAD sheet: zooming out from 3590 % held **`covered=0.000`
for about twenty frames**. The operator is looking at blank paper.

## ★★ Why a screenshot is the wrong oracle here, against this project's rule

`D:/dev/rag/egui/` records that layout and clipping defects have exactly one
oracle and it is a rendered screenshot. **This is not a layout defect, it is
a timing one**, and the interval is shorter than a window capture takes.
Three camera-based checks were built and driven before this module existed,
and every one of them was *unable to fail*:

1. *"is the canvas near-uniform?"* — passed, because the uncovered area is
   drawn as the page's own white and a technical drawing is ~90 % white
   anyway.
2. *"count ink during, compare with ink once settled"* — passed, with
   **identical** counts both sides: the raster landed before the shutter.
3. The same again with the stale-texture path deliberately sabotaged so the
   page had to blank. **Still passed.**

What the application knows and a camera does not is whether the pixels it
drew are a picture of the whole visible area or of a fraction of it. That is
one exact ratio, and [`publish_coverage`] publishes it.

## The fix costs no extra rasterisation

[`crate::app::state::OpenDoc::base_texture`] is not a new render. It is the
whole-page texture the shell already made, kept rather than dropped when a
sharper one replaces it, and only while it is under
[`crate::render::raster::BASE_MAX_PIXELS`]. A document opens at a fit zoom,
so the first raster of every page qualifies; the huge whole-page rasters
near the region tier never do.
