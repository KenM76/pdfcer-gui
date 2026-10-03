# `pdfcer-gui/app/actions/extract`

## Contract

`extract(doc, pages, labels, separations)` asks for a path, writes the pages
through `pageops::extract_with_labels` and records the receipt (or the
failure) as the document's note. It returns whether a file was written, and
`PageAction::ExtractPages`' *delete afterwards* runs only on `true`, so a
cancelled picker or a failed write never costs the operator pages.
`separations` is the Settings ▸ Pages policy, the same one a delete obeys.

The `extract` trace line carries `pages=` (written), `asked=` (requested),
`labels=keep|drop` (the choice), and the engine's `labels_dropped=0|1` and
`label_ranges=N` from `AssembleReport`, so a choice that did not reach the
engine shows as the line disagreeing with itself.

## Item notes

### `fn the_labels_choice_reaches_the_new_file`

Pages 1-2 of `fixtures/labelled-pages.pdf` (labels i ii 1 2): kept, they
read i, ii in the new file; dropped, they read 1, 2 and no range is stored.
Pages 3-4 would read 1, 2 either way, so they cannot tell the two apart.

### `fn suggested_path`

`<stem>-pages.pdf` beside the document, which is
`crate::app::save::suggested_path`'s shape with
[`crate::text::files::extract_pages_suffix`] in place of `-copy`. It is a
separate function rather than a shared one taking a suffix because that one
is private to a module this work may not edit, and because the two are
allowed to diverge: an extraction of pages 3–7 could one day suggest a name
that says so, and a save-a-copy never could.

# It is never the file that was opened

The promise [`crate::text::files::extract_pages_suffix`] makes, as a
mechanism. The extension is forced to `.pdf` for `save_copy`'s reason: the
bytes are a PDF whatever the source was called, and `SHEET.PDF` extracting
to `SHEET-pages.PDF` would be one more way for a downstream tool to disagree
about case.

### `fn scratch`

`std::env::temp_dir` rather than a path in the repository, for
`crate::app::save`'s stated reason: a test that writes beside the
fixtures leaves a file somebody eventually commits.

A **copy** of `super::super::pages`' helper rather than an import, and
deliberately so: it is five lines, and a test helper reaching across a
module boundary makes two suites fail together for reasons neither is
about. The directory name differs from that one's for the same reason —
two suites writing into one scratch directory is a shared mutable state
nobody declared.

### `fn edit`

The four-step protocol is not re-run here — there is no render worker in
a unit test and nothing else holds the `Arc` — but the step the
assertion depends on is: the mutation. `an_extraction_carries_unsaved_edits`
is the only consumer, and what it needs is a session with an edit in it.

### `fn an_extraction_writes_exactly_the_pages_it_was_given`

The round trip in the smallest form a unit test can hold, and the same
shape as `app::save`'s: write it, re-open it from disk through the
loader the application uses, and count. A build that wrote the whole
document — the plausible wrong answer, since `extract` and `save_copy`
both produce "a PDF beside the original" — passes any check that only
asks whether a file appeared.

### `fn an_extraction_carries_unsaved_edits`

Decision 018, asserted rather than trusted: the view handed to
`pageops::extract` is the **session's**, so a rotation made in this
sitting is in the file that comes out. A build that passed the loaded
`Document` instead would produce a valid file with the right page count
and the edit silently missing — which the test above would not catch.

### `fn the_suggested_extract_name_is_never_the_source_file`

`save::suggested_path`'s guarantee, for the second write-destination
this shell asks about. An operator who accepts the suggestion without
reading it must not overwrite the drawing they are extracting from.
