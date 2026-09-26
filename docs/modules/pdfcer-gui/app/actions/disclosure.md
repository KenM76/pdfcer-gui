# `pdfcer-gui/app/actions/disclosure`

## Item notes

### `static LAST_EDIT`

# Why a thread-local, and why that is sound rather than smuggled

The same answer `crate::panels::forms::edit`'s `LAST_FILL` gives, and
for the same reason it is worth restating rather than cross-referencing
away: it *should* be a field on [`OpenDoc`], beside `edit_epoch`,
dropped with the document. `OpenDoc` is declared in
`crate::app::state`, which this work may not extend, so the constraint
is a **territory boundary rather than a design judgement** — stated
here so whoever lifts it knows what the preferred shape is.

Why it is nonetheless sound: this is not document state. It is a note
about an edit that has already gone through the funnel, it cannot
change a pixel of the page, and nothing reads it except a bar deciding
whether to draw a sentence. It is correctly scoped too — `eframe`'s
update loop is one thread, so the writer and the reader are the same
thread, and a test on another thread gets its own empty slot rather
than another test's leftovers (which a `static Mutex` would hand it).

Staleness is handled by the `epoch` rather than by clearing: the
sentence is shown only while it describes the revision on screen, so an
undo silences it without anything having to remember to.
