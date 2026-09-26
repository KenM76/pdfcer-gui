# `pdfcer-gui/app/actions/extract`

## Item notes

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
