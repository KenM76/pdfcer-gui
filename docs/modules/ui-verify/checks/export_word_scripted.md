# `ui-verify/checks/export_word_scripted`

Three checks of File ▸ Export ▸ Word document…, each on a window off the
desktop driven only through `ScriptedPointer`, so they run under `--no-input`.

- `export_word_without_the_mouse` — `fixtures/ruled-table.pdf` at the
  window's defaults writes a `.docx` whose table is a Word table
  (`tables=1`, `structure=layout`, `structure_fallback=no-tree`).
- `export_word_follows_the_tags` — `fixtures/tagged-report.pdf`, whose one
  table has no rules, takes its heading and table from the tags
  (`structure=tree`, `tables=1`, `headings=1`).
- `export_word_honours_its_choices` — three exports, each choosing the
  opposite of the file's default outcome:
  - `four-pages.pdf`, range `2-3` typed and page breaks off:
    `pages=2`, `page_breaks=0`;
  - `ruled-table.pdf`, tables off: `tables=0`, `table_option=0`;
  - `tagged-report.pdf`, *The page layout only*: `structure=layout`,
    `structure_fallback=disabled`.

## Steps

1. The save target is deleted, so an earlier export cannot pass for this one.
2. Clicks: `ribbon.tab.file`, the collapsed `ribbon.group.file.export` when
   the band is narrow, then `ribbon.item.file.export_word`. The window must
   publish `export-word.export`.
3. The run's choices: clicks on their regions; a typed range is a click on
   `export-word.pages.range` and then the text typed into that viewport.
4. `export-word.export`. The save picker is answered by
   `PDFCER_DIAG_SAVE_PATH`.
5. The `export-word` line must carry every expected field.
6. The file on disk must start with a zip header and name
   `word/document.xml`, the part Word opens.

`pages=` and `tables=` are the engine's `DocxReport` counts, and
`structure_fallback=` its `TaggedLayoutReport`. `page_breaks=` and
`table_option=` are the options handed to `write_docx`: the package is
Deflate-compressed and this crate has no inflater, so a page break cannot be
counted in `document.xml` itself. The engine's own tests check that the
option removes the breaks.

## Falsification

- Passing no tables to `write_docx` fails `export_word_without_the_mouse`
  with `tables=0` quoted.
- With the plan's pages, page-break and table options and structure all
  ignored in the action (every page, `DocxOptions::default()`,
  `StructureSource::Auto`), `export_word_honours_its_choices` fails all three
  runs, each quoting its line.

## What it does not cover

Whether Word opens the file, and the heading, list and header/footer
inference: those are the engine's, and its tests check them. The refusal on
pages with no text is not driven, nor is *The file's tags, however little
they cover*.
