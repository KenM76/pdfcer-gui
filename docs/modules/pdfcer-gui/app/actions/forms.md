# `app::actions::forms` — everything done to a form FIELD

A sibling of [`super::dimensions`], [`super::pages`], [`super::vector`] and
[`super::export`], and it owns both halves of its subject: the action enum
[`FieldAction`] and the apply logic every one of its variants reaches.

## Why the family is a family

Fill, select, place, author, rename, delete a field, delete one of its
widgets, register an unclaimed one, ask what deleting a grouping node would
take and then take it — every verb here shares a property nothing else in
`actions` has: **each of them addresses a control by its fully
qualified NAME, or by the widget's `ObjId`. None of them uses a paint-order
index.** That is not a coincidence of style; it follows from where the data
lives. `/AcroForm` is in the document catalog (§12.7.2), a field is reached
through it, and a widget is reached through the field that claims it — never
through the page that happens to draw it.

It is the same test the neighbouring seams pass: [`super::vector`] is the
verbs that address paint-order indices, [`super::pages`] the verbs that
address page positions. A size-driven cut would not produce this grouping
and would not survive the next variant; this one tells you where the next
verb goes without anyone having to decide.

## Doc comments concatenate silently

Two `///` blocks left contiguous above one variant render as that variant
carrying both explanations while the variant the first block was written for
carries none. Nothing warns: `cargo doc` is clean, clippy is clean, every
test passes, and a variant that has lost its documentation is
indistinguishable from one that never had any. Keep exactly one block per
variant and keep it adjacent to the variant it documents.

## Registering a form control the document lists but no field claims

The *disclosure and refusal wording* is the substantial part of that verb,
which is why it earned a module of its own before the family joined it.

## What an unclaimed widget is, and why the shell can produce one

A `/Widget` annotation in a page's `/Annots` that no entry of the document's
`/AcroForm` `/Fields` reaches. It **draws** — border, background, the whole
appearance stream — and nothing can fill it, because every filling verb
addresses a field by its fully qualified name and this box is in no field.

This project's recurring failure mode, a visible control that is silently
inert, arriving through a **document** rather than through a ribbon. The
operator clicks it, types, and nothing happens.

pdfcer makes them itself. `EditSession::insert_pages` copies everything
reachable from a page, and a page's `/Annots` reaches its widgets — but
`/AcroForm` is document-level and is not merged, so a source with 12 fields
inserted into a blank document produces 13 widgets and no form at all. The
engine measured exactly that (`examples/orphan_probe.rs`, pdfbox corpus) and
returns the count in `InsertOutcome::orphaned_widgets`.

## Two shapes, and only one of them can be put back

The engine's measurement is the reason this module has two refusal arms
rather than a success path and a shrug:

| shape | of 13 measured | carries | registering it |
|---|---|---|---|
| **merged field-widget** (§12.7.3.1) | 11 | its own `/FT`, `/T`, `/V`, `/DA` | **recovers the field exactly** |
| **bare kid** (a radio group's member) | 2 | nothing at all | **creates a new, empty field** |

The second row is `insert_pages` dropping `/Parent` from every dictionary it
copies. For a page that is correct — following it would drag the source's
whole page tree across. For a widget, `/Parent` **is** its link to its
identity, so those two arrived having lost the name `GroupOption`, the type
`/Btn`, the radio flags `0xC000` and the value `Option2`. Nothing in the
target document holds any of it.

An operator cannot see which shape a box is, and the difference decides
whether pressing Register restores something or invents something. That is
why [`crate::text::status::adopt_declined_no_name`] refuses to use the word
*restore*, and why it names re-inserting from the source as the only route
that gets the original back.

## Why this uses the funnel

`adopt_widget` writes `/AcroForm` and `/T`. It is a document edit with one
undo entry, so it goes through [`super::apply::vector_edit`] like every
other one — the render worker stopped, the mutation, the epoch bumped, the
page invalidated. Nothing here is special except the wording.

## Item notes

### `fn from`

The same reasoning [`super::vector`]'s `From` carries: which sub-enum a
verb is filed under is this module's business, and a panel button that
renames a field has none of it. `.into()` at the push, `From` here.

### `fn set_button_action`

# The disclosure this owes, and it is the whole reason `replaced` exists

`ButtonActionChange::replaced` names what was destroyed — **as a `String`,
including `"JavaScript"`, deliberately**. `pdfcer-core`'s own reasoning:
`Option<ButtonAction>` would have made a removed script inexpressible and
forced it to be reported as `None`, i.e. as *"there was nothing there"*.

A form editor overwriting another tool's work needs to know it did, and
this is the one moment it can be told. The status line carries it.

pdfcer will not write a script back. That asymmetry is deliberate and is
disclosed on the row rather than here: a `Foreign` action renders no Change
control at all, so the only way to reach this function with a script in the
way is through a route that has already said so.

### `fn correctable`

A free function taking `&EditError` so it is testable without an
`EditSession`, a document or a frame — the same shape
`crate::dialogs::insert_image`'s arithmetic was pushed into, and for the
same reason: `pdfcer_core::edit::EditError` is `#[non_exhaustive]`, so this
match needs a wildcard, and a wildcard inside a closure inside a funnel is
a place a future variant goes to be silently ignored.

Here it is one visible function with a test beside it. The wildcard means
*"anything else is a fault, not a chore"*, which is a real distinction and
the right default: a new refusal variant appearing in a future engine build
reaches the trace with its own words and does not silently acquire one of
these two sentences, which would be worse than saying nothing.
