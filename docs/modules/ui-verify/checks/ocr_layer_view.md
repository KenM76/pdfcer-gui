# `ui-verify/checks/ocr_layer_view`

`ocr_layer_view` — **View ▸ Display's OCR switch paints the invisible text,
and its slider's two ends are the page alone and the text alone.**

## What it guards

`view.ocr_layer` sets `ViewState::ocr_overlay`. While it is set, the canvas
veils the raster towards paper white and paints every run holding an
invisible glyph in the operator's OCR colour, both at one alpha taken from
the slider (`ocr_blend`). The slider is drawn only while the switch is on.

## How it drives

1. It opens `fixtures/ocr-layers.pdf`. Page 1 draws `Visible page one` at
   y = 720 and carries `recognised one` at render mode 3, y = 700.
2. It selects Read mode and the View tab, then captures the window and counts
   ink pixels (`pixels::ink_run_into`) in each line's glyph band, mapped from
   PDF points through `CanvasMapping`. The drawn line must have ink (the
   control arm; without it the check is SKIPPED) and the recognised line must
   have none. The slider must not be declared.
3. It presses `ribbon.item.view.ocr_layer`. The recognised line must gain ink
   and `ribbon.item.ocr_blend` must be declared.
4. It presses the slider's rail and drags well past its right end: the trace
   must read `ocr-blend set=1.000`, the drawn line must have no ink and the
   recognised line some. Then past its left end: `set=0.000`, the recognised
   line blank and the drawn line inked again.
5. It presses the switch again; the slider must no longer be declared.

## Not covered

- The ink metric counts a pixel at least 60 (sum of RGB) below the band's
  dominant colour. A veil stopping a few parts in 255 short of opaque would
  leave the drawn text too faint to count, so the exactness of the right end
  rests on `set=1.000` and the `veil_alpha` unit tests, not on the pixels.
- The status-bar line and the "this page was never recognised" disclosure.
- The OCR colour setting (a separate row).

## Falsification

- Suppressing `ocrlayer::draw_text` fails step 3: `on` reads 0 px on the
  recognised line.
- Dividing the slider's percent by 110 instead of 100 fails step 4: the right
  end reads `set=0.909`.
