# `dialogs::export_tables` — the tables on the page, as CSV, Excel or OpenDocument

File ▸ Export ▸ **Tables…** (O257). The window chooses the pages and the
format: one CSV per detected table, or one Excel (`.xlsx`) or LibreOffice
(`.ods`) workbook holding every table, a sheet each.

## What it runs

`pdfcer_core::table_detect::detect_tables` finds both ruled tables (from drawn
lines) and aligned tables (from how the words line up). The engine only
detects over the **whole document** (request `G061`), so the tables are
filtered to the plan's pages afterwards. The page-level counts
(`pages_over_limit`, `pages_unreadable`) cannot be attributed to a subset of
pages. They are reported only when the plan covers every page.

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
action filters the engine's `Table`s to the plan's pages and hands them over
unchanged; every `XlsxReport` count that means something changed or was left
out becomes a receipt line.

## OpenDocument

`pdfcer_gui_base::tableexport::ods`, written here because the engine has no
ODS writer (request `G062`). Sheets are named as Excel's are.
Merges are spanned cells over covered cells, header rows bold, columns sized
from their longest line (ODF has no autofit). A cell becomes a `float` only
under `tableexport::number`'s narrow rule — optional minus, digits without a
leading zero, optional decimals, at most 15 significant digits — a strict
subset of what the Excel writer accepts, so the two formats never disagree
on a value, only on whether `1,200` is a number. Control characters XML
cannot carry are dropped and counted. The zip is the minimum ODF needs:
`mimetype` first and stored, the other parts deflated over `flate2`.

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
