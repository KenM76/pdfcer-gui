# `pdfcer-gui/dialogs/ocr_upscale`

The **Enlarge each page before reading** group in File ▸ Recognise text….
It is for a scan whose text is smaller than the recogniser reads well.

## Contract

- **Opt-in and off by default.** Enlarging adds no detail, so a page that
  already reads well can read worse. `Enlarge::default()` is off, and an off
  group sends `upscale: None` with the run.
- **Factor** 1.5 to `MAX_FACTOR` (4) in steps of 0.5. The engine accepts
  1.0 to 4.0. ×1 changes nothing, so it is not offered. A unit test walks
  every offered step through `Upscale::new`.
- **Kernel**: bicubic (the engine's default, recommended) or Lanczos.
- **Sizes**: the first page in scope as drawn (`ocr::raster_size`) and as
  read (`Upscale::output_size`). When the enlarged page is over
  `MAX_OUTPUT_PIXELS`, a sentence says the run will stop there and asks for a
  smaller factor. A letter page at 300 dpi at ×4 is over it. The refusal
  itself is the engine's sentence at Run (`RunnerError::Upscale`); the
  warning only lets the operator avoid it.
- **Not drawn** for PaddleOCR-VL (`ocr::can_upscale`). R9: the engine refuses
  it, so there is no disabled stub, and `Enlarge::upscale` returns `None` for
  that model even when the box was ticked under another model.

## Shared with Enlarge scans

`factor_and_filter` draws the factor slider and the kernel choice. File ▸
Enlarge scans (`dialogs::deskew`, `ScanKind::Enlarge`) draws the same pair,
so both offer the same factors, the same default and the same words.

## Disclosure

When a run enlarged its pages, the result's notes add
`text::ocrupscale::enlarged`: the factor, the kernel, and the last page's
size before and after. It is off-canvas, as the engine's words come back in
page space and the layer renders as any other.

## Trace

- `ocr-started … upscale=<factor|off> filter=<bicubic|lanczos|-> magnify=smooth`
  (`trace_tokens`, and `ocr::raster::MAGNIFY`).
- `ocr-recognised … enlarged=<none|WxH->WxH>`.

Regions: `ocr-upscale` (the checkbox), `ocr-upscale-lanczos`,
`ocr-upscale-sizes`.
