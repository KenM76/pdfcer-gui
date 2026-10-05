# `ui-verify/checks/ocr_scripted`

`recognised_text_is_in_the_saved_file` (ocrs),
`paddle_text_is_in_the_saved_file` (PaddleOCR) and
`paddle_vl_text_is_in_the_saved_file` (PaddleOCR-VL) — File ▸ Recognise text… puts
an OCR layer into the session, Ctrl+S saves it, and the saved file, reopened in
a second process, has text on its page.

# What it drives

A scratch copy of `fixtures/synthetic-image-only.pdf` (ignores `--pdf`), with
`ocr_engine` and `ocr_model` seeded to the variant's recogniser, off the desktop
under `--no-input` with a `ScriptedPointer`. Needs a packaged build: the
recogniser's models must sit beside the binary. The dialog's opening choice
(`ocr-model-start chosen=`) must be the seeded recogniser; any other choice
means this build cannot run it (its `ocr-model … runnable=no why=` line says
why) and the check SKIPs rather than pressing a Run that would run another
recogniser, or nothing. PaddleOCR-VL decodes token by token and gets 12,000
settle frames where the others get 600.

1. Read mode, File tab, then `ribbon.item.file.ocr`. At the check's 1400-px
   window the Recognise group is collapsed, so
   `ribbon.group.file.recognise.collapsed` is clicked first when the item is
   not declared.
2. `ocr-run` clicked; `ocr-started engine=` names the seeded recogniser.
3. `ocr-recognised pages=1 recognised>0`, then the edit funnel's `ocr-layer`.
4. Ctrl+S gives `save-in-place outcome=ok`; the saved bytes are longer than the
   original and start with them (an appended revision).
5. A second launch on the saved file runs Recognise text again and must trace
   `ocr-refused reason=AlreadyHasText`. That refusal is the engine's own reading
   of the saved page, so it witnesses that the text is in the file, not merely
   in the session that wrote it.

# Why the second launch

Steps 3 and 4 prove that a layer was applied and that a save happened. Only a
fresh process reading the file proves that the save carried the layer. The
untouched fixture is recognised, not refused, so the refusal cannot come from
the fixture.

# Falsified

- Not raising `Action::ApplyOcr` from the dialog's poll: all three variants
  fail at step 3 (no `ocr-layer` line).
- Handing `add_ocr_layer` an empty page list: the ocrs and PaddleOCR variants
  fail at step 4 (the save appends nothing).

The step-5 arm has not been fired by a plant: no planted defect produced a
layer that was applied and saved but absent on reopen.
