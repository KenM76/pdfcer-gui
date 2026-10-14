# `ui-verify/checks/ocr_upscale`

`enlarging_pages_reaches_the_recogniser` — File ▸ Recognise text…'s
*Enlarge each page before reading* reaches the engine, at the factor and with
the kernel chosen.

# What it drives

`ocr_scripted::recognise_choosing` with `ocrs` on a copy of
`fixtures/synthetic-image-only.pdf`, off the desktop under `--no-input`,
clicking `ocr-upscale` (the box) then `ocr-upscale-lanczos` before Run. The
factor is the group's default, ×2.

1. `ocr-upscale-sizes` is drawn once the box is ticked.
2. `ocr-started` carries `upscale=2 filter=lanczos magnify=smooth`.
3. `ocr-recognised` carries `enlarged=WxH->2Wx2H` and `recognised=` above 0.

`enlarged=` comes from `OcrRunner::upscale` on the runner that read the
pages, so it is the engine's state, not the dialog's.

# Not driven

The group is not drawn for PaddleOCR-VL, which refuses enlargement; that is
covered by `ocr_upscale`'s unit tests, as driving it needs the VL add-on.
The too-large warning (`MAX_OUTPUT_PIXELS`) is unit-tested too.

# Falsified

`.with_upscale(request.upscale)` dropped from the run options: the run still
asks for ×2 and the recogniser reports `enlarged=none`, so step 3 fails.
