# `dialogs::export_tables` — the tables on the page, as CSV

File ▸ Export ▸ **Tables…** (O257). The window chooses the pages, and the
export writes one CSV file per detected table.

## What it runs

`pdfcer_core::table_detect::detect_tables` finds both ruled tables (from drawn
lines) and aligned tables (from how the words line up). The engine only
detects over the **whole document** (request `G061`), so the tables are
filtered to the plan's pages afterwards. The page-level counts
(`pages_over_limit`, `pages_unreadable`) cannot be attributed to a subset of
pages. They are reported only when the plan covers every page.

## The file

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

## Disclosure (R8b)

Everything detection had to infer is counted in the receipt, off-canvas:
tables found by alignment, headers guessed from bold type, a fill or a heavier
rule, merged cells, pages too dense to search, and pages that could not be
read. If no tables are found, the export refuses **before** the picker opens.
That way nobody chooses a file name and ends up with nothing.

## Not here

**Excel (`.xlsx`)** output needs a writer crate that is not in the engine's
lockfile, and a new dependency is an operator decision. **Word** output is
not built.

## Remembered

The page scope persists as `export_tables_pages` (`current` | `all`).

## Trace

`export-tables tables= aligned= headers= merged= dense= unreadable= first=` on
success. On the other paths the trace is `export-tables-refused reason=no-tables`,
`export-tables-cancelled`, or `export-tables-failed reason=detect|write`.
`export-tables-remembered saved= scope=` is written when the window closes
on Export.
