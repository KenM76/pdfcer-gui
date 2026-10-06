# `ui-verify/checks/invisible_text_scripted`

`added_invisible_text_is_saved_invisible` — Edit ▸ Add text with the text
pen's **Invisible** switch ticked writes the new run in rendering mode 3, the
way a scan's recognised text is stored (`OPERATOR_REQUESTS.md` O286, adding
text to the OCR layer).

# What it drives

A copy of `fixtures/layer-assign.pdf` (ignores `--pdf`), off the desktop
under `--no-input` with a `ScriptedPointer`, `PDFCER_DIAG_INVOKE` arming
`mode.edit,edit.add_text`.

1. Properties is raised by its dock tab if its body is not on screen.
2. `properties.tool.text_pen_invisible` is clicked; the `text-pen` line must
   say `invisible=1`.
3. A click on blank paper at (150, 450) must trace `text-edit-caret kind=Add`;
   `QZXW` is typed; a click at (350, 570) commits, tracing `add-text`.
4. Ctrl+S must trace `save-in-place outcome=ok`.
5. The saved file is read: every stream body is inflated where it inflates,
   and each `(QZXW) Tj` found is judged by the last `<n> Tr` between its `BT`
   and the `Tj`. Every one must be mode 3, and there must be at least one.
6. Each of those streams must open a `/pdfc_OCR` section before the show,
   with `/Engine (manual)` in its property list: the run is part of the
   page's recognised-text layer, so Remove OCR text takes it.

The engine writes `<mode> Tr` into every added run, mode 0 included, so the
visible case produces a readable `0`, not an absence.

# Falsified

Dropping `.with_render_mode(placed.pen.render_mode())` from
`app::actions::addtext::request` fails step 5 with modes `["0"]`.
Dropping its `into_ocr_layer()` fails step 6 with no section found.
