# `app::actions::insertruns` — inserting a page list that repeats a page

## The defect it guards against

`EditSession::insert_pages(source, &[usize], position)` copies the whole list
through one import mapping. A source index named twice resolves, the second
time, to the page object already copied, so the target's `/Kids` holds one
object twice. pdfcer's own reader rejects that tree as a cycle. Release
builds have no assertion for it; the only witness is the trace line
`pages-resync-failed detail=page tree cycle … kept=N`, and the operator sees
a document whose page count did not change and a save that writes the broken
tree.

Two routes reached it: Insert from file with a range such as `1,1`, and
Insert blank pages, which inserts page 0 of a blank document `count` times.
The engine side is filed as `request_G185_insert_pages_repeated_index_aliases_one_page.md`.

## The workaround

`distinct_runs` cuts the list into consecutive runs with no repeat inside any
of them (`[0,0,0]` → `[0] [0] [0]`; `[0,1,2,1,3]` → `[0,1,2] [1,3]`). Each run
is one `insert_pages` call, so each gets a fresh mapping and fresh copies.
Later runs go directly after the pages already inserted —
`Before(first_slot + inserted)`, or `End` when that slot is past the last
page — so the result is the list in order, where the operator asked.

`insert` then folds the calls into one undo step with
`coalesce_last(calls, InsertPages { count })`. A list with no repeat is one
run and one call.

## What it reports

`Inserted` carries the summed `InsertOutcome` (counts added, flags OR-ed),
whether the fold succeeded, how many calls landed, and a later run's
refusal. A refusal on the first call is `Err` (nothing changed); a refusal
after some runs landed is `Ok` with `stopped` set, because the document has
changed and must be resynced. `pages::insert_from_view` turns `stopped` into
`text::insertblank::insert_stopped` (how many of how many arrived, and why)
and an unfolded result into `text::reface::undo_split`.

## `insert_blank`

Builds the blank source with `pdfcer_gui_base::blank::document_sized` and
inserts `vec![0; count]` through `pages::insert_from_view`. A refusal to make
the blank is traced `insert-blank-refused` and disclosed with
`text::pages::insert_failed`; otherwise it traces `insert-blank-requested`.
Driven by `ui-verify`'s `blank_pages_are_inserted_as_one_step`.
