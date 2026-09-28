# `ui-verify/checks/export_tables_scripted`

`export_tables_without_the_mouse` — File ▸ Export ▸ Tables… writes the one
ruled table in `fixtures/ruled-table.pdf` in each format, chosen by clicking
its radio. The window is off the desktop and driven only through
`ScriptedPointer`, so the check runs under `--no-input`.

The fixture is built by `fixtures/ruled-table.PROVENANCE.py`: three columns by
three rows of stroked rules, a word header and four unambiguous numbers.

## Steps, once per format (CSV, Excel, OpenDocument)

1. The save target is deleted, so an earlier export cannot pass for this one.
2. Clicks: `ribbon.tab.file`, `ribbon.item.file.export_tables`,
   `export-tables.format.<csv|xlsx|ods>`, `export-tables.export`. The save
   picker is answered by `PDFCER_DIAG_SAVE_PATH`.
3. A new `export-tables` line must carry `format=` equal to the clicked radio
   and `tables=1`.
4. The file on disk must hold the format's signature: the header row
   `Item,Qty,Mass` for CSV; a zip header plus `xl/worksheets/` for Excel; a
   zip header plus the ODF spreadsheet mimetype for OpenDocument.
5. For both workbooks, `numbers=` must be above zero.

## Falsification

Clicking the `csv` radio in the `xlsx` pass fails step 3 with the traced
format quoted.

## What it does not cover

Whether Excel or LibreOffice open the file. Both workbook packages are the engine's, and its tests check their structure.
