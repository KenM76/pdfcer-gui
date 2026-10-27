# `ui-verify/checks/ocr_editable`

`editable_ocr_takes_the_words_out_of_the_scan` — File ▸ Recognise text…'s
*Editable text and images* reaches the engine as `OcrOutput::EditableText`,
the written page is the editable output, the copy saves, and a second run on
the saved page is refused.

# What it drives

`ocr_scripted::recognise_choosing` with `ocrs` on a copy of
`fixtures/synthetic-image-only.pdf`, off the desktop under `--no-input`.

1. Clicks `ocr-output-editable`, Run, then Ctrl+S.
   - `ocr-started` carries `output=editable`.
   - `ocr-layer-output` carries `editable=` ≥ 1, `visible=` ≥ 1 and
     `images-rewritten=` ≥ 1 — the engine's `OcrLayerReport` totals, so the
     scan was rewritten and the words are visible text.
   - `save-in-place` carries `outcome=ok`.
2. Relaunches on the saved copy, clears `ocr-skip` (the page now draws text)
   and runs again: `ocr-layer-refused` carries the engine's
   `EditableLayerPresent` words, *carries editable OCR text*.

# Not driven

Remove OCR text on an editable page (left alone, `remove-ocr-layers-editable-left`;
refused when nothing else matched) is not driven.

# Falsified

On the release before the choice existed the dialog draws no
`ocr-output-editable` region and the check FAILs (an absent choice is the
defect, so it is not left to the harness's SKIP).
