# `ui-verify/checks/export_tables_choices`

`export_tables_honours_its_choices` — the Export tables window's page range,
sheet grouping and number reading each reach the workbook writer, and a
workbook choice is remembered into the next window. The window is off the
desktop and driven only through `ScriptedPointer`, so the check runs under
`--no-input`.

The fixture, `fixtures/two-ruled-tables.pdf`, is built by its
`PROVENANCE.py`: two pages, one ruled table each, with two whole numbers and
two three-decimal numbers (`1.234`, `2.500`, …) per table. Its default
workbook holds two tables on two sheets, with 4 numbers and the 4
three-decimal values kept as text and counted ambiguous.

## Runs

The three runs share one profile, so each clicks every choice it relies on.

| Run | Choices | The `export-tables` line must carry | The file |
|---|---|---|---|
| range | Excel, range `2` typed, a sheet per table, numbers auto | `tables=1 sheets=1` | no `xl/worksheets/sheet2.xml` |
| single | Excel, all pages, every table on one sheet, numbers auto | `tables=2 sheets=1 sheet_option=single` | no second worksheet |
| european | OpenDocument, all pages, a sheet per table, numbers european | `tables=2 sheets=2 numbers=8 ambiguous=0` | a zip package |

The european run also requires its window to open with `sheets=single
format=xlsx`: the single run's choices, remembered.

Each run: the save target is deleted; `ribbon.tab.file`, the collapsed
Export group when the band is narrow, `ribbon.item.file.export_tables`; the
window must publish `export-tables.export`; the run's choices (a typed range
is a click on `export-tables.pages.range` and the text typed into that
viewport); `export-tables.export`, with the picker answered by
`PDFCER_DIAG_SAVE_PATH`.

`sheets=`, `numbers=` and `ambiguous=` are the engine's report counts. The
second-worksheet test reads the zip's directory, where part names are stored
uncompressed; the ODS sheets live inside the compressed `content.xml`, so the
OpenDocument run is judged on the trace.

## Falsification

With the plan's sheet grouping and number reading ignored in the action
(`XlsxOptions::default()`, `OdsOptions::default()`) and every page exported
whatever was typed, all three runs fail, each quoting its line.

## What it does not cover

Per-page grouping and US and Off reading are not driven; the engine's tests
cover the writers under each. Whether Excel or LibreOffice open the file is
not checked.
