# `dialogs::export_word` — pages as an editable Word document

File ▸ Export ▸ **Word document…** (O284). The window decides four things
before the save picker opens; each is one of `pdfcer export-docx`'s options.

| Choice | Engine setting | Default |
|---|---|---|
| Pages: all, this page, or a typed range | the pages laid out and searched for tables | all |
| *Start each PDF page on a new page* | `DocxOptions::page_breaks` | on |
| *Write tables as Word tables* | `DocxOptions::tables` | on |
| *Take headings and tables from* | `TaggedLayoutOptions::use_structure` (`StructureSource`) | the tags, when they cover most of the text |
| *Recognised text*, only when the document has a pdfcer OCR layer | `ExtractOptions::with_ocr_layer` (`dialogs::recognised`) | included |

The defaults are the engine's (`WordExportPlan::new`), so pressing Export at
once writes what the window-less export wrote. The choices are not remembered
between exports.

A typed range is resolved, sorted and deduplicated when Export is pressed
(`WordExportPlan::new`): Word receives the pages in document order, once each,
which is also what `read_structure_tree_in_pages` requires.

## What it runs

`app::actions::export_word::export` with the `WordExportPlan`:

1. `taggedexport::lay_out` over the plan's pages, under the chosen
   `StructureSource`. *The page layout only* is `StructureUse::Never`: the
   tree is not consulted and the trace says `structure_fallback=disabled`.
2. Tables: none searched when tables are off. Otherwise the tree's when it
   was followed, else `detect_tables_in_pages` over the plan's pages.
3. `write_docx` with `DocxOptions` built from the plan.

## Disclosure (R8b)

The receipt is unchanged by the choices, with one addition: when *the file's
tags, however little they cover* was chosen and the file has no tags, a line
says headings and tables were judged from the page layout instead.

## Regions

`dialog:export-word` (body), `export-word.pages.all|current|typed`,
`export-word.pages.range` (the typed-range field), `export-word.page_breaks`,
`export-word.tables`, `export-word.structure.auto|tags|layout`,
`export-word.recognised.all|only|without`,
`export-word.export`.

## Trace

`export-word-open page= pages= ocr_layer_pages=` when the window opens;
`export-word-requested pages= page_breaks= table_option= structure=
recognised=` when Export is pressed. The `export-word` result line carries the engine's report,
then `page_breaks=` and `table_option=` as handed to `write_docx`, then the
structure fields.

## Checks

`export_word_without_the_mouse`, `export_word_follows_the_tags` (defaults),
`export_word_honours_its_choices` (each choice),
`recognised_text_choice_filters_word` (the recognised-text choice).
