# `ui-verify/checks/ocr_layer_kept`

`an_edited_ocr_layer_stays_a_layer` — editing a recognised word inside an OCR
layer pdfcer wrote keeps the layer's marked section (`OPERATOR_REQUESTS.md`
O286 item 5, the layer-boundary half; engine G125).

# What it drives

A copy of `fixtures/ocr-layers.pdf` (two pages, one pdfcer layer each, and a
decoy from another producer on page 2), off the desktop under `--no-input`
with a `ScriptedPointer`, `PDFCER_DIAG_INVOKE` arming
`mode.edit,view.ocr_layer,edit.text`.

1. A click at (90, 703) inside page 1's layer word `recognised one`, End,
   `XQ` typed, Escape commits, Ctrl+S must trace `save-in-place outcome=ok`.
2. File ▸ Remove OCR text is pressed (its collapsed group opened first when
   the item is not drawn).

# What it asserts

1. The saved stream holding `XQ` is a whole layer: it opens with
   `/pdfc_OCR` and `/Producer (pdfcer)` on its first line and ends with
   `EMC`. A marker part-way through a stream is not found by the engine's
   `read_marker`, so a folded layer still shows its marker and fails here.
2. The last `remove-ocr-layers-applied` line says `removed=2 pages=2`: the
   edited page's layer is still found, and the decoy is still left alone.

# Falsified

Driven against a build pinned before the engine kept the layer's stream
(v0.5.0-dev.20261006.2, engine v0.78), both assertions fail: the edit folds
the layer's stream into the page's first `/Contents` stream, and Remove OCR
text reports `removed=1 pages=1`.
