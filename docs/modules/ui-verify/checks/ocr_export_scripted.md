# `ui-verify/checks/ocr_export_scripted`

`ocr_layer_exports_to_word` and `ocr_layer_exports_to_text` — File ▸ Export ▸
Word document… and Text… on a page carrying an OCR layer write the layer's
words into the exported file.

# What it drives

`fixtures/ocr-layer.pdf` (ignores `--pdf`): a picture with an OCR layer in
rendering mode 3 and a visible stamp beside it. Off the desktop under
`--no-input` with a `ScriptedPointer`, `PDFCER_DIAG_SAVE_PATH` answering the
save dialog.

1. Read mode, File tab, the export item (through
   `ribbon.group.file.export.collapsed` when the band folds the group), then
   the window's Export button with its defaults.
2. `export-word` / `export-text` is traced.
3. The written file is read: the `.txt` as it is, the `.docx`'s
   `word/document.xml` inflated out of the zip (`miniz_oxide`; the engine's
   writer puts sizes in each local header, so the local headers are walked in
   order).
4. The text holds `VISIBLE CONTROL STAMP`, which proves the page was read, and
   every one of `SITE`, `PLAN`, `REVISION`, `DRAWING`, which only the OCR layer
   spells.
5. The window's open line says `ocr_layer_pages=0`: the fixture's layer is not
   one pdfcer wrote, so the recognised-text choice is not offered.

`exported_text` is the drive and the file read, shared with
`ocr_export_filter`, which clicks a recognised-text choice before Export.

# Why this fixture and not a recognition

A recognition needs model weights beside the binary and produces whatever the
model reads. The fixture's layer is written in the engine's own emission
shape (`fixtures/ocr-layer.PROVENANCE.md`), so the check runs in any build and
its words are fixed. That a real recognition reaches the saved file is
`ocr_scripted`'s claim; this check starts from a saved layer.

# Falsified

Dropping every run with an invisible glyph after extraction, in
`taggedexport::lay_out` (Word) and the text export's extraction: both variants
fail at step 4, naming the four missing words. The stamp arm has not been
fired by a plant.
