# `ocr-paddle-layer.pdf` — provenance

**The output of `paddle_text_is_in_the_saved_file`** (`tools/ui-verify`): File ▸
Recognise text… with the Paddle engine on a copy of
`fixtures/synthetic-image-only.pdf`, saved, frozen here. Regenerate by running
that check and copying its saved PDF over this file; then re-measure every
number below, because a different recogniser build lays the words out
differently.

48,614 bytes, MD5 `07ddd4ce2eadb23bb225986b3e639542`. One page,
`/MediaBox [0 0 306 396]`, `/Producer (pdfcer)`.

## Why it exists beside `ocr-layer.pdf`

`ocr-layer.pdf` is hand-built: base-14 Helvetica, one `Tf` size per line. This
file is the layer as the engine writes it after a real recogniser: one show
operator per word, each with its own size and `Tz` horizontal scale fitted to
the box Paddle found, in the font the engine embeds for OCR text. A drawing of
the layer that ignores the font, size or `Tz` looks right on the hand-built
file and wrong on this one.

## Measured facts a check relies on

- The extractor finds 96 runs carrying invisible text.
- The word `dimensions` is drawn from x = 66.8 to 117.1 pt on the baseline
  y = 336.5 pt, at 13.75 pt; its extracted run box reaches x = 123.9.
- Its runs are each one show operator, so an edit opens on a single word; the
  editor declines to split a line (Enter) here, because the words differ in
  size and scale.

## Who depends on this file

`tools/ui-verify/src/checks/ocr_layer_wysiwyg.rs`.
