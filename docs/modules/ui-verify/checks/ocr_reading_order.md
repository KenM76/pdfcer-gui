# `ui-verify/checks/ocr_reading_order`

`recognised_text_reads_column_by_column` — a recognised two-column scan is
written column by column, so its text reads in order.

# What it drives

A scratch copy of `fixtures/ocr-two-columns.pdf` (ignores `--pdf`): one
image-only page, two columns of two paragraphs whose lines sit at the same
heights, the paragraphs opening *Apples*, *Bridges* (left) and *Candles*,
*Dolphins* (right). Off the desktop under `--no-input` with a
`ScriptedPointer`, through `ocr_scripted::recognise` and
`ocr_export_scripted::exported_text`:

1. File ▸ Recognise text… with `ocrs`, then Ctrl+S
   (`save-in-place outcome=ok`).
2. `ocr-layer-structure` has at least one block and no fewer lines than
   blocks.
3. A second launch exports the saved file to Text with
   `export-text.order.as-drawn`, which writes the content stream's order
   rather than re-inferring one, so the file reads the layer as the writer
   laid it out.
4. The four openers appear in reading order.

# Not asserted

Paragraphs. On OCR word boxes the engine makes every line its own block
(`lines=15 blocks=15` on this fixture), filed as G131; the check notes it.
Tighten step 2 to `blocks < lines` when G131 lands.

# Falsified

On the v0.78 engine build (`D:/scratch/s3/control`) the check fails at
step 2 (no structure line), and the layer it saved is one text object in
`ocrs`'s order: Candles, Apples, Dolphins, Bridges, which step 4 rejects.
