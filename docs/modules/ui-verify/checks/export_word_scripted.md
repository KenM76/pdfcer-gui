# `ui-verify/checks/export_word_scripted`

`export_word_without_the_mouse` — File ▸ Export ▸ Word document… writes
`fixtures/ruled-table.pdf` as a `.docx` whose table is a Word table. The
window is off the desktop and driven only through `ScriptedPointer`, so the
check runs under `--no-input`.

## Steps

1. The save target is deleted, so an earlier export cannot pass for this one.
2. Clicks: `ribbon.tab.file`, the collapsed `ribbon.group.file.export` when
   the band is narrow, then `ribbon.item.file.export_word`. The save picker
   is answered by `PDFCER_DIAG_SAVE_PATH`.
3. An `export-word` line must carry `tables=1`.
4. The file on disk must start with a zip header and name
   `word/document.xml`, the part Word opens.

## Falsification

Passing no tables to `write_docx` fails step 3 with `tables=0` quoted.

## What it does not cover

Whether Word opens the file, and the heading, list and header/footer
inference: those are the engine's, and its tests check them. The refusal on a
document with no text is not driven.
