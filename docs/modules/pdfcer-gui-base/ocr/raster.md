# `ocr::raster` — the raster a recogniser reads

How a page becomes pixels for recognition: the render options, the size, and
the optional enlargement before reading.

## Render options

`raster_options(configured, model)` takes the settings funnel's options
(`SettingsExt::render_options` on the document's settings) and sets
`RenderOptions::with_image_magnify(magnify(model))`: `MagnifyFilter::Smooth`
for every engine but PaddleOCR-VL. An image drawn larger than its own pixels is
smoothed rather than drawn as square blocks; the recogniser reads edges, and
a block edge is a false one. The display keeps `MagnifyFilter::PointSample`,
so a scan looks exactly as it is on screen. This matches the engine CLI's own
OCR default. Every other option comes from the funnel; a run that drew with
`RenderOptions::default()` would ignore the operator's settings, which
`no_call_site_builds_its_own_options` forbids.

PaddleOCR-VL reads `MagnifyFilter::PointSample`, as the display draws. On
`fixtures/synthetic-image-only.pdf` smoothed, its whole-page read did not end
within `paddle_vl_text_is_in_the_saved_file`'s budget (12,000 frames, about
ten minutes); point-sampled it ends in seconds. Region by region
(`by_layout`) it ends either way. It resizes every region to its model's
own size, so smoothing first buys it nothing. The `ocr-started` line's
`magnify=` is `magnify(model)`, the filter that run drew with.

## Size

`raster_size(page)` is the `(width, height)` in pixels that `recognise_one`
draws `page` at: the crop box at `fitted_dpi`, through
`pdfcer_render::page_device_geometry`, so the page's `/Rotate` is applied the
same way the render applies it. `None` for a page with no area. Recognise
text uses it to show the sizes before Run.

## Enlargement

`can_upscale(model)` is false for `paddle-vl` only. PaddleOCR-VL resizes
every region to its own model's size, so an enlargement does nothing useful,
and `OcrRunner::load` refuses one with `RunnerError::UpscaleUnsupported`. This
restates that rule so the dialog draws no option the engine will refuse.

`Enlarged` is what a run reports: the `Upscale` asked for and the last page's
raster before (`from`) and after (`to`). `Enlarged::of` gives `None` for no
upscale or an identity one, so an off or ×1 run discloses nothing. The engine
enlarges inside `recognize_page`, scales the dpi with it, and maps the words
back to the unenlarged raster; this module never resamples.
