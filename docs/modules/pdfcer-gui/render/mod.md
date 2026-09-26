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
