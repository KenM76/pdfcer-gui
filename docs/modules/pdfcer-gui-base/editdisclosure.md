# `pdfcer-gui-base/editdisclosure`

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

### `struct EditDisclosure`

`pdfcer-core`'s vector verbs return `Result<Vec<String>, EditError>`, and the
`Vec<String>` is the **disclosure list**: prose the surgery owes because it
had to change an operator's *form* to express their request. Rule 4 says a
disclosure belongs on an operator-visible surface, so routing this list only
to `PDFCER_DIAG` would record it without disclosing it.

# Why this is shaped like [`crate::panels::forms::edit::FillDisclosure`]

Because it is the same fact in a different verb, and the precedent had
already settled every question this one raises: a note about an edit,
stamped with the epoch the edit produced, read by a surface that draws it
only while it still describes the document on screen. Building a second
mechanism beside that one would give the status bar two ways to learn the
same kind of thing, and the second would be the one that forgot to retire
itself.

# What it deliberately does NOT carry: the verb's name

A `FillDisclosure` carries the **field name**, because a fill raised from
the Forms panel happens in a list of forty rows and the sentence is read
somewhere other than where the value was typed. The vector verbs have no
such gap: the gesture that raises one is a drag on the object the sentence
is about, the sentence appears on the next frame, and core's own wording
(*"This shape…"*, *"This point…"*) is written for exactly that reading.

The only name available here is [`vector_edit`]'s `label` — `move-node`,
`delete-objects` — which is a **trace token**, not operator copy. Putting it
on screen would either ship a hyphenated internal identifier to an operator
or require a second catalog translating trace tokens into English, which is
a second vocabulary for the verbs the ribbon already names.

### `fn last_edit_disclosure`

**The status bar's read** — see [`crate::app::status`]. Returns `None` when
the last edit disclosed nothing, was on another document, or has since been
undone or superseded.

# It cannot be live at the same time as a fill disclosure

Both are keyed on [`OpenDoc::edit_epoch`], and one edit bumps the epoch
once. So the epoch on screen was produced by exactly one edit, which was
either a form edit (recording a `FillDisclosure` and no `EditDisclosure`)
or a vector edit (the reverse). The bar therefore never has to arbitrate
between two disclosure lines competing for one row: the mutual exclusion is
a property of the epoch, not a rule anybody has to enforce.

### `fn record_edit_disclosure`

Written unconditionally by [`vector_edit`], including the overwhelmingly
common empty case. Overwriting with `None` is not required for correctness
(the epoch filter above already retires a stale sentence) and is done
anyway, so the slot never holds a note whose only defence against being
shown is an integer comparison that a future undo implementation could get
wrong.

### `fn record_note`

The narrow public door onto the same slot [`record_edit_disclosure`] writes,
and it exists for exactly one caller: `canvas::interact`, when a click with
the caret tool armed **cannot** place a caret. That is not an edit — nothing
was written, no epoch moved — so it has no disclosure list to ride in on, and
without this it would have nowhere to be said.

It has to be said somewhere. `DEFECTS.md` D4a records the old shell's
handling of the same case: a `cross_run` flag that *"silently disables the
whole typing loop"*, so the operator pressed keys and nothing happened. A
limit stated in a sentence is a limit; the same limit stated by a keyboard
that stops responding is a bug report.

**`epoch` is the CURRENT one, not a new one**, and that is what makes the
lifetime right without anything remembering to clear it: the sentence is
visible from now until the next real edit moves the epoch past it, which is
the same rule `vector_edit`'s own stamp follows and for the same reason its
comment gives.

### `fn record_notes`

[`record_note`]'s plural, and it exists because the slot holds one
disclosure rather than a queue: a second `record_note` **replaces** the
first rather than joining it, so a caller with two things to say has to say
them in one call or lose one.

That is not hypothetical. `crate::app::save::save_in_place` records a
receipt naming the file it wrote, and — for a signed document — owes a
second sentence about what the write did to the signature
(`crate::text::signature`). Two `record_note` calls would have shown
whichever ran last and silently dropped the other, and the one it dropped
would have been chosen by statement order rather than by importance.

The order of `notes` is the reading order and the caller owns it. The
bar joins them with a single space behind one lead-in
(`crate::text::status::edit_disclosure_line`), so the first sentence is the
one an operator reads if they read only one.

**`epoch` is the CURRENT one**, exactly as [`record_note`]'s is: the
sentences are visible from now until the next real edit moves the epoch
past them, which is what retires them without anything having to remember
to.
