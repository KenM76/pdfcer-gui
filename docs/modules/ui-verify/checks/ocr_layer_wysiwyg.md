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

1. It waits for the layer's `ocr-ink-built` line and shoots the window.
2. A click inside the last letter of `dimensions` opens the editor with the
   caret at its end; the window is shot.
3. `XQ` is typed; the window is shot.
4. Escape commits; it waits for the next `ocr-ink-built` line (the edit epoch
   moved, so the layer is laid out again) and shoots the window.

# What it asserts

1. The layer laid out at least 90 of the page's 96 runs in their own fonts
   (`laid=`), so the comparison is not against the stand-in.
2. Over the editor box (`text-edit.box`), trimmed 3 points on each side and 7
   on the right for the caret, the layer before the edit and the editor just
   opened each hold at least 20 layer-coloured pixels, and at most 15% of the
   larger count differ by more than 48 in any channel.
3. The same over the box after typing, between the editor typed into and the
   layer once the edit committed.

Assertion 2 is the font, size and strength agreement as an edit opens;
assertion 3 is the same as it commits, on text the fixture never held.

# What it does not claim

It does not judge the stand-in that draws a run spread over several show
operators: this fixture has none. Nor the saved file; `ocr_edit_preview`
checks that the edited word stays invisible.

# Falsified

- `ocrink::lay` answering `None` for every run fails assertion 1 (`laid=0`).
- The editor preview drawn at full opacity instead of the layer's fails
  assertion 2 (321 of the word's pixels differ, 61%).
