# `app::actions::insertblank` — Pages ▸ Insert blank…

The engine has no blank-page verb. `insert_blank` builds a one-page blank
document of the chosen size with `pdfcer_gui_base::blank::document_sized`
and inserts the list `vec![0; count]` through `pages::insert_from_view`, the
funnel every page insert uses.

That is one `insert_pages` call. `EditSession::insert_pages` gives a source
index named more than once its own complete copy, so `count` repeats of page
0 are `count` independent page objects and one undo step.

A refusal to make the blank is traced `insert-blank-refused` and disclosed
with `text::pages::insert_failed`; otherwise it traces
`insert-blank-requested`. Driven by `ui-verify`'s
`blank_pages_are_inserted_as_one_step`, which fails on
`pages-resync-failed` (a page tree the reader rejects, as one page object
listed three times would be) and on an insert one Ctrl+Z does not undo.
