# `dialogs::export_tables` — the tables on the page, as CSV, Excel or OpenDocument

File ▸ Export ▸ **Tables…** (O257, O284). The window chooses the pages and
the format: one CSV per detected table, or one Excel (`.xlsx`) or LibreOffice
(`.ods`) workbook holding every table. For a workbook it also chooses how the
tables share sheets and how cells are read as numbers; neither group is drawn
for CSV, to which neither applies.

A typed range is sorted and deduplicated when Export is pressed
(`TableExportPlan::new`), so the tables come out in document order, once.

The choices scroll above a fixed Export / Cancel row (`FOOTER_PTS`), so the
buttons stay on screen when the workbook groups make the body taller than the
window; `BODY_FLOOR_PTS` keeps a short window from giving the scroll area a
negative height, which draws nothing.

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
`XlsxOptions` from the plan: merges kept, header rows bold.

| Window choice | `SheetGrouping` | Engine `SheetLayout` |
|---|---|---|
| A sheet for each table (default) | `PerTable` | `PerTable` |
| A sheet for each page | `PerPage` | `PerPage` |
| Every table on one sheet | `Single` | `Single` |

| Window choice | `NumberReading` | Engine `NumberLocale` |
|---|---|---|
| Only where every country reads them the same (default) | `Auto` | `Auto`: `1.234` stays text and is counted ambiguous |
| Reading 1,234.5 | `Us` | `Us` |
| Reading 1.234,5 | `European` | `European` |
| Never | `Off` | `Off`: every cell text |

The GUI holds its own enums because the engine's are `#[non_exhaustive]`:
a window's radio list and a preferences token need a closed set. The
action hands the engine's `Table`s over unchanged; every `XlsxReport` count
that means something changed or was left out becomes a receipt line, and the
ambiguous-numbers line says how to convert them.

## OpenDocument

The engine's writer, `pdfcer_core::export::ods::write_ods`, with
`OdsOptions` from the same plan. It shares the Excel writer's sheet grouping
and number rule, so a cell is a number in the `.ods` exactly when it is one in
the `.xlsx`. `OdsReport` is disclosed as `XlsxReport` is, less `cells_truncated`:
an ODF cell has no length limit.

## Disclosure (R8b)

Everything detection had to infer is counted in the receipt, off-canvas:
tables found by alignment, headers guessed from bold type, a fill or a heavier
rule, merged cells, pages too dense to search, and pages that could not be
read. A workbook adds the cells written as numbers, ambiguous numbers kept as
text, dropped characters, and (Excel) cells cut or left out at Excel's
limits. If no tables are found, the export refuses **before** the picker
opens, so nobody chooses a file name and ends up with nothing.

## Remembered

`export_tables_pages` (`current` | `all`), `export_tables_format`
(`csv` | `xlsx` | `ods`), `export_tables_sheets` (`table` | `page` |
`single`) and `export_tables_numbers` (`auto` | `us` | `european` | `off`).

## Trace

`export-tables-open page= pages= scope= format= sheets= numbers=` when the
window opens. `export-tables format= tables= aligned= headers= merged=
numbers= ambiguous= sheets= sheet_option= number_option= dense= unreadable=
first=` on success: `numbers`, `ambiguous` and `sheets` are the writer's
counts (zero for CSV), the two `_option` fields the plan's choices. On the other paths the trace is
`export-tables-refused reason=no-tables`, `export-tables-cancelled`, or
`export-tables-failed reason=detect|write`. `export-tables-requested pages=
format= sheets= numbers=` when Export is pressed. `export-tables-remembered
saved= scope= format= sheets= numbers=` is written when the window closes on
Export. Regions: `export-tables.format.csv|xlsx|ods`,
`export-tables.pages.all|current|typed`, `export-tables.pages.range`,
`export-tables.sheets.table|page|single`,
`export-tables.numbers.auto|us|european|off`, `export-tables.export`.
