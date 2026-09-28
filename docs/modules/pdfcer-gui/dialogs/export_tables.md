# `dialogs::export_tables` — the tables on the page, as CSV, Excel or OpenDocument

File ▸ Export ▸ **Tables…** (O257). The window chooses the pages and the
format: one CSV per detected table, or one Excel (`.xlsx`) or LibreOffice
(`.ods`) workbook holding every table, a sheet each.

## What it runs

`pdfcer_core::table_detect::detect_tables_in_pages` finds both ruled tables
(from drawn lines) and aligned tables (from how the words line up) on the
plan's pages only, so the page-level counts (`pages_over_limit`,
`pages_unreadable`) are about the pages chosen.

## CSV

- **One CSV per table.** When there is one table, the picked path is the file.
  When there are more, each goes to `stem-p<page>-t<n>.csv`: the page is
  1-based, and `n` is the table's order on that page, top to bottom.
- **RFC 4180 with CRLF rows.** A field is quoted when it holds a comma, a
  quote, a line break or leading or trailing spaces. A multi-line cell stays
  as one field.
- **UTF-8 with a byte-order mark.** Without the BOM, Excel reads the file in
  the machine's code page and mangles °, ± and ⌀.
- **Merged cells.** The text goes in the top-left field and the rest of the
  merged area is left empty. CSV has no way to express a merge.

## Excel

The engine's writer, `pdfcer_core::export::xlsx::write_xlsx`, with
`XlsxOptions::default()`: a sheet per table named `Table n`, merges kept,
header rows bold, and numbers under `NumberLocale::Auto` — a cell whose value
depends on the reader's convention (`1.234`) stays text and is counted. The
action hands the engine's `Table`s over unchanged; every `XlsxReport` count that means something changed or was left
out becomes a receipt line.

## OpenDocument

The engine's writer, `pdfcer_core::export::ods::write_ods`, with
`OdsOptions::default()`. It shares the Excel writer's sheet grouping and number
rule, so a cell is a number in the `.ods` exactly when it is one in the
`.xlsx`. `OdsReport` is disclosed as `XlsxReport` is, less `cells_truncated`:
an ODF cell has no length limit.

## Disclosure (R8b)

Everything detection had to infer is counted in the receipt, off-canvas:
tables found by alignment, headers guessed from bold type, a fill or a heavier
rule, merged cells, pages too dense to search, and pages that could not be
read. A workbook adds the cells written as numbers, ambiguous numbers kept as
text, dropped characters, and (Excel) cells cut or left out at Excel's
limits. If no tables are found, the export refuses **before** the picker
opens, so nobody chooses a file name and ends up with nothing.

## Not here

**Word** output is not built.

## Remembered

`export_tables_pages` (`current` | `all`) and `export_tables_format`
(`csv` | `xlsx` | `ods`).

## Trace

`export-tables-open page= pages= scope= format=` when the window opens.
`export-tables format= tables= aligned= headers= merged= numbers= dense=
unreadable= first=` on success. On the other paths the trace is
`export-tables-refused reason=no-tables`, `export-tables-cancelled`, or
`export-tables-failed reason=detect|write`. `export-tables-requested pages=
format=` when Export is pressed. `export-tables-remembered saved=
scope= format=` is written when the window closes on Export. Each format radio
publishes the region `export-tables.format.csv|xlsx|ods`.
