# render — turning a page into pixels, and pixels into a texture

Two modules with one seam between them, and the seam is the reason the
split exists:

| module | runs on | knows about |
|---|---|---|
| [`worker`] | a background thread | `pdfcer-render`, never egui |
| [`raster`] | the UI thread | egui textures, never rasterization |

**Rasterization can happen on any thread; texture upload cannot** — it
needs an `egui::Context`, which belongs to the UI thread. That single
fact is why [`worker::RenderWorker`] returns a `Pixmap` rather than a
`TextureHandle`, and why [`raster::texture_from_pixels`] exists as the
other half of it.

Both files are Class A salvage from
`D:\Dev\pdfce\crates\pdfce-gui\src\`: `render_worker.rs` (466 code lines
plus 116 test lines) and `raster.rs` (363 code lines). Their
documentation is carried across rather than paraphrased, because it
records measured evidence — 28.9 ms to cancel a render against
10,367 ms to let one finish; a real CAD sheet at ~10 s at 1× and ~58 s
at 2× — that cannot be re-derived by reading the code and that decides
the design.

## What is deliberately NOT here yet

- **A thread pool** for thumbnails and adjacent-page prerender. The
  worker is single-slot by design (see [`worker::RenderWorker`]); a pool
  is a different structure and arrives with the page rail at stage S3.
- **The thumbnail cache.** It was part of the salvaged `raster.rs`, and
  it belongs with the Pages panel that consumes it, not with a canvas
  that has no rail.
- **A display list.** `BENCHMARK.md`'s single biggest win, and
  explicitly post-fold-in work. It would replace what happens *inside*
  the worker, not the worker.

## Item notes

### `mod ceiling`

Its header carries why this one ceiling cannot be derived the way the two in
[`crate::viewer::ceiling`] are: the wall is inside `tiny-skia` and is
content-dependent, measured 28x apart on two pages of the same document, so
the only honest source of the number is a refusal that has already happened.

### `mod halo`

Its header carries the one-sentence cause — `render_page` sizes its pixmap
to the `/CropBox`, so nothing culls the content, there are simply no pixels
out there — and why a halo that would not fit is declined rather than
clamped.

### `mod ink`

Its header is the record of an afternoon spent proving the shell innocent by
hand: a near-uniform canvas has two causes, and until this module existed
the harness could see only one of them.

### `mod offpage`

`render_page_region` accepts a rectangle outside the `/CropBox` by
construction and is untested there; a shell feature built on an unexercised
engine path is one whose first failure looks like a shell defect.

### `mod pressure`

A texture upload that fails for want of graphics memory raises
`GL_OUT_OF_MEMORY` on a flag that `egui_glow` reads only under
`debug_assertions`, so in a release build it is completely silent and
presents as an empty rectangle drawn at full frame rate. Its header carries
the frame boundary the whole module is built around — an upload ordered in
one frame is performed at the end of it and its error is first readable at
the top of the next — and why attribution refuses to guess.

### `mod raster`

Its header carries the y flip, which is the half that goes wrong: a missed
flip shows the opposite end of the page, which at deep zoom looks like a
blank raster rather than a coordinate error.

### `mod region`

Its header carries O174: this module handed `render_page_region` a
canvas-space rectangle, which is right for an upright page at the origin and
wrong for a turned one, so the operator's `/Rotate 270` sheet jumped and
distorted above the whole-page → region crossover and nowhere below it.

### `mod strategy`

Its header carries the constraint that shaped it: panning at full detail is
a property of rasterizing the WHOLE PAGE, and region rendering would cost
it. So the region path engages only above the pixmap ceiling, where the
whole-page path cannot work at all — nothing is taken away to pay for it.
