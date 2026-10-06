# `ui-verify/checks/ocr_edit_preview`

`an_ocr_word_is_previewed_in_its_own_font_in_the_layer_colour` — editing a
word of a scan's recognised text, with the OCR text layer shown, previews it
live in the word's own font and in the layer's colour, in place of the layer's
drawing of that word, and the saved word is still invisible
(`OPERATOR_REQUESTS.md` O286, WYSIWYG editing of the OCR layer).

# What it drives

A copy of `fixtures/ocr-layer.pdf`, off the desktop under `--no-input` with a
`ScriptedPointer`, `PDFCER_DIAG_INVOKE` arming
`mode.edit,view.ocr_layer,edit.text`.

1. A click at (102, 704), just inside the end of the 14 pt word `SITE`, puts
   the caret after its `E`. End is not used: it goes to the end of the line,
   which in a scan's layer is another word's run.
2. `XQ` is typed and the window is shot.
3. Escape commits; Ctrl+S must trace `save-in-place outcome=ok`.

# What it asserts

1. The last `text-edit-shaped` line has `shaped=1` and `chars=6`, and no
   `text-edit-preview-fallback` names a reason: the preview is the word's own
   font, not the stand-in.
2. The last `ocr-layer-held` line has `runs=1`: the layer left exactly the
   edited word to the preview, so it is neither drawn twice nor are its
   neighbours hidden.
3. In the screenshot, the band holding `SITEXQ` has at least 20 pixels of the
   layer's hue (red and blue well above green) and fewer dark pixels than a
   quarter of those, and the band holding the neighbouring `DRAWING` still has
   at least 20.
4. The saved file shows `(SITEXQ)` under rendering mode 3, judged by
   `invisible_text_scripted::modes_showing`.

# What it does not claim

The fixture's layer has no `/pdfc_OCR` marked section, so nothing here says
whether that section survives an edit; `an_edited_ocr_layer_stays_a_layer`
(`ocr_layer_kept`) drives that on a layer pdfcer wrote.

# Falsified

- Painting the preview in the run's own fill instead of `colour32` fails
  assertion 3 (18 layer-coloured, 38 dark pixels).
- Making `ocrlayer::holds` always false fails assertion 2 (`runs=0`).
