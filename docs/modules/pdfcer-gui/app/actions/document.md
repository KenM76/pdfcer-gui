# `app::actions::document` — the actions that decide WHICH document is on
screen, and the two guards the destructive ones share

## Why this is a file of its own

`apply`'s match has a block at the top whose own comment reads: *"The
three actions that are about WHICH document is open, matched BEFORE the
guard below."* That is the seam this file is cut along. Everything below
that block acts **on** the open document;
everything in it decides **which** document is open, or whether there is
one. That is a different subject with a different failure mode — the arms
below can be wrong about a page, and these can be wrong about an afternoon's
work.

## Which arms carry the guards, and which must not

| arm | discards a document? | guards |
|---|---|---|
| `apply_open` | **no** — it adds a tab | none |
| `apply_open_with_password` | **no** — the same, after a retry | none |
| `apply_new` | **no** | none |
| `apply_new_sized` | **no** | none |
| `apply_close` | yes — the one on screen | both |
| `apply_close_document` | yes — the one whose tab was clicked | both |
| `apply_close_other_documents` | yes — every other tab | both, by delegation |
| `apply_reread_with_duplicate_keys` | **yes** — replaces it with a fresh parse of its own bytes | both |

Open and New park what is open and add a tab. Nothing is discarded, so the
question a guard on them would ask — *"your unsaved edits will be lost"* —
would be **false**, and a confirmation that says something untrue is how an
operator learns to dismiss confirmations unread. The protection belongs
where the loss happens, and the loss happens in more than one place, which
is exactly what
[`tests::every_action_that_discards_a_document_asks_about_unsaved_edits`]
is shaped to notice.

## The two guards, in order, and why the order is not interchangeable

Both closing arms ask the same two questions in the same sequence:

| # | question | answer | why first / second |
|---|---|---|---|
| 1 | **Is a save in flight?** (`PdfcerApp::save_pending`) | decline outright, trace it | the document's bytes are mid-write; there is no answer the operator could give that would make proceeding safe |
| 2 | **Are there unsaved edits?** (`DialogsState::ask_unsaved`) | **ask**, and resume afterwards | there is an answer the operator can give, and it is theirs to give |

Reversing them would put a question in front of an operator whose answer
cannot be honoured — they would press *Close without saving* and be declined
anyway, which reads as a broken button.

**They are two predicates, not one, and conflating them is the mistake this
file exists to prevent.** `crate::app::lifecycle::save_pending` carries the
whole argument: it asks *"is a save in flight"*, is permanently `false`
because `file.save_copy` is synchronous, and is **not** *"are there unsaved
edits?"* — a successful save-a-copy leaves the document exactly as unsaved
as it was, because the copy went somewhere else. `dialogs::ocr`'s
`UnsavedEdits` refusal reads `edit_epoch != 0` and would break the moment
somebody merged them.

## The failure mode this file's shape closes

The two guards look alike from a distance, and an arm that has one reads as
an arm that is protected. `file.close`'s tooltip promises the operator
*"You are asked what to do about unsaved edits first."* A guard that is
present, well argued, correct, **and answering the other question** looks
exactly like the guard that promise needs — which is how an arm comes to
destroy every edit made since the file was opened, silently, with a doc
comment above it explaining the guard it does have. Keeping every such arm
in one file, under the table above, is what makes the mismatch visible.

## An arm can discard a document without closing anything

`apply_reread_with_duplicate_keys` is the arm whose destruction is
invisible in its own shape. It does not say *close*. The tab does not go
away. The path, the name and the page count are all identical afterwards.
What is gone is **every edit made since the file was opened**, because the
engine's intervention is a re-load rather than a patch — *"a decision made
during parsing is not a value that can be edited afterwards, because the
discarded one was never built into the document"* — so the document that
comes back is a fresh parse of the bytes on disk with an empty undo stack.

That is the same failure mode from a new direction: **a reader has no
reason to ask whether this arm needs the guards**, because nothing about it
looks destructive. It is in this file, in the table above, and inside the
test's destructive set for that reason and no other — the test's predicate
names the re-read verb as well as the two close verbs, because *"names a
close verb"* is only a proxy for *discards a document*.

[`tests::every_action_that_discards_a_document_asks_about_unsaved_edits`]
is the half of this that fails when a new arm arrives without the guards.
It asserts the property rather than a count of arms, because the count was
never what mattered.

## Item notes

### `fn every_action_that_discards_a_document_asks_about_unsaved_edits`

# What is asserted, and what deliberately is not

Not *"there are N arms in this file and all of them call
`ask_unsaved`"*. A count is not the property, and an arm that adds a tab
rather than replacing one must **not** ask — a guard there would state
something false to the operator, so a test keyed on the count would
demand a lie the moment the count moved.

The property is **an arm that can destroy a document must ask first**,
and that is what is checked: any body naming a destructive verb must
also name both guards, in order. The counts survive only as floors — an
instrument that cannot fail detects nothing — and they are floors on
*both* populations, so neither "no arms were found" nor "no destructive
arms were found" can pass silently.

# Why it reads the source rather than driving the functions

Driving them is not possible in a unit test and the reason is the point:
three of the five end in `open_path` / `new_document` /
`new_document_sized`, which build real `EditSession`s, and the two that
do not would pass trivially by having no document to ask about. A
behavioural test here would exercise the **absence** of the guard's
precondition rather than the presence of the guard.

Crude, and deliberately so — the same trade this project made for the
settings-coverage gate. A crude check that fails when the guard is
dropped beats an exact one that cannot run.
