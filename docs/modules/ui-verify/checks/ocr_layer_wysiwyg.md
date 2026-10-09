# `ui-verify/checks/ocr_layer_wysiwyg`

`the_ocr_layer_draws_a_word_as_its_edit_does` — with the OCR text layer shown,
a recognised word looks the same in the layer, in the text editor while it is
typed, and in the layer again once the edit commits: same font, size,
horizontal scale, place and strength (`OPERATOR_REQUESTS.md` O289, *editing
the OCR layer must be WYSIWYG live*).

# What it drives

A copy of `fixtures/ocr-paddle-layer.pdf`, off the desktop under `--no-input`
with a `ScriptedPointer`, `PDFCER_DIAG_INVOKE` arming
`mode.edit,view.ocr_layer,edit.text`.

1. It waits for the layer's first raster (`ocr-ink-built`) and shoots the
   window.
2. A click inside the last letter of `dimensions` opens the editor with the
   caret at its end; the window is shot.
3. `XQ` is typed; the window is shot.
4. Escape commits; it waits for the next `ocr-ink-built` line (the edit epoch
   moved, so the layer is rendered again) and shoots the window.

# What it asserts

1. The layer traced a raster (`ocr-ink-built`).
2. Over the editor box (`text-edit.box`), trimmed 3 points on each side and 7
   on the right for the caret, the layer before the edit and the editor just
   opened each hold at least 20 layer-coloured pixels, the smaller count is at
   least 75% of the larger, and at most 15% of the larger count differ by more
   than 48 in any channel of their 3 x 3 mean.
3. The same over the box after typing, between the editor typed into and the
   layer once the edit committed.

The layer is the engine's raster and the editor's preview is egui's, and the
two antialias a glyph differently: compared pixel by pixel the same word
differs over about a fifth of its ink. The 3 x 3 mean removes that and keeps a
one-pixel move.

Assertion 2 is the font, size and strength agreement as an edit opens;
assertion 3 is the same as it commits, on text the fixture never held.

# What it does not claim

It does not judge the saved file; `ocr_edit_preview`
checks that the edited word stays invisible.

# Falsified

- The layer drawn one pixel to the right fails assertion 2 (72 of the word's
  pixels differ, 18%).
- The layer drawn at half its opacity fails assertion 2 (35 and 407
  layer-coloured pixels).
- The editor preview drawn at full opacity instead of the layer's fails
  assertion 2.
