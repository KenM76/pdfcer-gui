# `canvas::ocrink` — the OCR layer, rendered by the engine

The OCR text layer is a render: the visible part of the page through
`pdfcer_render::render_page_region` with
`RenderOptions::with_invisible_text(Some(InvisibleTextPaint::new(rgb).with_only(true)))`.
In only-mode the renderer paints the page's invisible (mode 3 and 7) text in
the layer's colour on a transparent backdrop and paints nothing else — no
images, no paths, no visible text, no annotations. Clips still apply, and the
canvas's layer visibility (`OpenDoc::layer_visibility`) is passed, so a hidden
optional-content group hides its recognised text here as it does on the page.

So each word is drawn in its own font, size, horizontal scaling and position by
the same renderer that draws the page and a text edit's preview, and a word
does not change shape when an edit opens on it or commits
(`OPERATOR_REQUESTS.md` O289 item 15).

## Contract

- **Off the UI thread.** A region render interprets the whole content stream,
  which is too slow for a frame on a dense sheet. One render is in flight at a
  time; a new one cancels it (`RenderCancel`). The result is handed back
  through a shared slot and uploaded on the next frame.
- **What a raster is of** (`Want`): session, edit epoch, page, colour, layer
  generation, region and density. Any edit moves the epoch, so a committed
  edit is rendered again.
- **Region and density.** The region is the visible part of the page,
  quantised by `render::region::page_region` so a small pan reuses the raster
  already made. The density is the screen's pixels per page point, rounded up
  to a quarter octave so a small zoom change reuses it too.
- **Settling.** A render starts once the view has held still for
  `SETTLE_SECS` (0.15 s), so a wheel flick does not start one per notch — except
  when nothing of the current document state is on screen, which starts at once.
- **Never blanks.** The previous raster stays on screen, drawn at its own page
  region through the current page mapping (`render::region::region_on_screen`),
  until its replacement arrives.
- **The edited run.** The run an open edit is rewriting (its canvas rectangle,
  from `ocrlayer::edited_rect`) is cut out of the raster: the texture is drawn
  as up to four pieces around it, so no pixels are re-uploaded. The editor's
  preview (`textedit::shaped`) draws that run at the layer's opacity. After the
  edit commits, the cut stays while the raster on screen predates the commit,
  so the old word does not reappear for the length of one render.
- **Trace.** `ocr-ink-built page= epoch= px=WxH ms=` when a raster arrives;
  `ocr-ink-refused page= why=` when the engine refuses one, which leaves the
  previous raster up. The texture is counted in the texture census as
  `Surface::OcrLayer`.

## Limits

- One page: the page `OpenDoc::page_text` caches, as `canvas::ocrlayer`
  documents.
- Deep zoom: the visible region is derived through the `f32` page mapping, so
  past the deep-position tier the region is placed with that mapping's error.
  At such a zoom one glyph fills the window.

## R8b

This draws text the file holds, in a view the operator turned on; with the
layer off it draws nothing. The pixels are the renderer's own, so the layer
shows the words as the saved file lays them out.
