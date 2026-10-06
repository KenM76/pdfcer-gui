# `ui-verify/checks/ocr_export_filter`

`recognised_text_choice_filters_word` and
`recognised_text_choice_filters_text` — the Word and Text export windows'
recognised-text choice decides what is written.

# What it drives

`fixtures/ocr-layers.pdf` (ignores `--pdf`): two pages, each with visible
text and an OCR layer pdfcer wrote, and on page 2 a look-alike layer with
another `/Producer`. Off the desktop under `--no-input` with a
`ScriptedPointer`, through `ocr_export_scripted::exported_text`, twice per
format:

1. Read mode, File tab, the export item, then
   `export-<format>.recognised.without`, then Export.
2. The window's open line must say `ocr_layer_pages=2`.
3. The file must hold `Visible page one`, `Visible page two` and `not ours`
   (the look-alike counts as page text), and neither `recognised one` nor
   `recognised two`.
4. The same with `export-<format>.recognised.only`: the two layer words
   present, the three page-text strings absent.

`ocr_export_scripted` asserts the other side: on a document whose invisible
text is not a pdfcer layer the window counts 0 and draws no choice.

# Falsified

Dropping `.with_ocr_layer(plan.ocr_layer)` from both export actions fails
both variants at step 3, with `recognised one` and `recognised two` present
though excluded.
