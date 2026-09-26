# `app::actions::forms::groups` — deleting a grouping node, and the preview
that has to happen first

The apply half of [`super::FieldAction::ArmGroupDeletion`] and
[`super::FieldAction::DeleteGroup`]. The vocabulary stays in [`super`] with
the rest of the field verbs; only the logic and the store live here.

## Why this is a file rather than two more functions in [`super`]

**R2, and the seam is real rather than size-driven.** `super` is *"everything
done to a form FIELD"* — verbs that address a control by its fully-qualified
name and change what one control is or holds. This module addresses a name
that **is not a control**: a grouping node has no type, no value, no widget
and no rectangle, and the whole difficulty of the verb is that its
consequences are invisible and must therefore be *stated in advance*. That
is a different subject with a different failure mode, and it is the reason
this is the only form verb in the shell that takes **two** operator presses.

## The two-press protocol, and why it is not a confirmation dialog

```text
  press 1  ──▶  ArmGroupDeletion(Some(name))
                   │  field_group_deletion_preview(&mut session)
                   │  writes NOTHING, bumps NO epoch
                   ▼
                ARMED  ──▶  the panel draws what would go, in numbers and names
                   │
  press 2  ──▶  DeleteGroup { group }
                   │  vector_edit ▸ delete_field_group
                   ▼
                the engine's REPORT ──▶ the status bar's disclosure row
```

### Why the preview cannot happen in the panel

Because `EditSession::field_group_deletion_preview` takes **`&mut self`**,
and a panel body is handed `&OpenDoc` — a *shared* reference, which is the
compile-time expression of "no code path runs from a widget to a document"
(see `app/actions/OVERVIEW.md`). The session lives behind an `Arc`, and the
only place `Arc::get_mut` succeeds is inside the funnel, after the frame.

⇒ So the preview **is** an action, even though it changes nothing. It is the
same shape as `FieldAction::Select`: raised by a widget, applied by the
funnel, bumping no epoch and invalidating no page. What is unusual is only
that its *result* has to travel back to the panel, which is what the store
below is for.

### Why not a modal dialog, when `crate::dialogs` is full of them

Three reasons, in order of weight:

1. **The section already answers the question the dialog would re-ask.** The
   Forms panel has the parsed `/AcroForm` in hand and is already listing the
   grouping nodes. A window would have re-parsed it, at a second moment,
   producing a second answer to one question — which is exactly the shape
   `panels::forms::tab_order::register`'s header argues against for the
   Register rows, and it fails the same silent way: the list and the button
   beside it come to disagree about the set.
2. **It is where the operator already is.** They opened Field groups because
   they wanted to know what was in one. The answer belongs under the row
   they were reading.
3. **A dialog needs six files this work does not own** — a variant, a host
   arm, a window, a region, a close path and a focus policy — and the
   disclosure is the substance here, not the chrome.

## Where the armed preview is kept, and why it is a thread-local

Exactly [`crate::app::actions::disclosure`]'s answer, restated because the
reasoning has to be re-checked rather than inherited:

- It **should** be a field on `OpenDoc`, beside `selected_field`. It is
  per-document state with a per-document lifetime.
- `OpenDoc` is declared in `crate::app::state`, which sits at 1,494 of R2's
  1,500-line budget. Extending it is not a design judgement here, it is a
  file that has no room — stated so whoever splits that file knows where
  this belongs.
- It is nonetheless sound: this is **not document state**. It cannot change
  a pixel, it cannot reach a save, and nothing reads it except the section
  that drew the row. `eframe`'s update loop is one thread, so the writer and
  the reader are the same thread, and a test on another thread gets its own
  empty slot rather than another test's leftovers.

### Staleness is handled by the EPOCH, not by remembering to clear

[`Armed::epoch`] is the `OpenDoc::edit_epoch` current **when the preview was
taken**, and [`armed`] answers `None` for anything else. That one comparison
retires the preview on every path that could invalidate it, without any of
them knowing this store exists:

| what happens | epoch | the armed preview |
|---|---|---|
| the deletion is confirmed | bumps | gone — the group it named no longer exists |
| some other form edit lands | bumps | gone — its counts were taken against an older form |
| **undo / redo** | bumps | gone — and this is the one a `clear()` call would have missed |
| the operator cancels | — | cleared explicitly, by `ArmGroupDeletion(None)` |

The undo row is why the epoch rule is not merely tidy. Undo and redo do not
clear selections, and a preview surviving one would sit under a row
describing a subtree that had come back with different contents.

## Trace names

`form-group-preview` for the arm, `form-group-preview-refused` for the
refusal, and — inside the funnel closure —
**`delete-field-group-applied`**, with the `-applied` suffix.

That suffix is not decoration. `vector_edit` writes its own line for the
same edit under the bare label (`delete-field-group page=0 n=1 epoch=…`),
and trace matching is on the **exact event name**, so a driven check taking
`.last()` would read the funnel's line — which carries no `terminals=` key —
and report zero fields removed about a deletion that removed four. This
project has made that mistake twice: `text-style` and `import-form-data`,
both fixed the same way. **A module's own summary line takes a verb suffix;
the funnel's label keeps the bare name.**

## Item notes

### `static ARMED`

One rather than a map keyed by group name, and that is a decision.
Arming a second group while a first is armed **replaces** it, because
two disclosure blocks open at once in a narrow dock pane is two
destructive confirmations competing for one glance — and the operator
can only be about to press one of them.

### `const FIXTURE`

The fixture is chosen for the **cascade**, which is the case the
disclosure exists for. A one-level group would exercise the verb and
prove nothing about the number an operator cannot predict — how many
*other* nodes go with the one they named. `PROVENANCE.md` records that
`Personal.Name` sits one level shallower on purpose.

### `fn a_preview_is_taken_then_confirmed_and_retires_itself`

Written as one test rather than three because the facts it asserts are
only meaningful in sequence: a preview that is readable is worth nothing
if the deletion that follows removes a different set, and an epoch rule
is worth nothing unless something actually moves the epoch.

The four claims, in order:

1. **The preview reaches the store**, with the counts the panel draws.
2. **It is invisible at any other revision** — which is the entire
   safety argument for keeping it outside `OpenDoc`, and covers undo and
   redo without either of them knowing this store exists.
3. **The deletion removes the whole subtree in one command**, so the
   epoch moves exactly once.
4. **The armed preview retires itself** on that move, with nothing
   having called a clear.

### `fn cancelling_clears_the_block_and_edits_nothing`

Asserted separately because it is the one path that must move no
epoch: an operator who backs out of a destructive confirmation has done
nothing, and a shell that bumped the revision for it would silently
retire whatever disclosure was on screen and mark a clean document
edited.

### `fn a_terminal_name_arms_nothing`

The engine rules that `NotAGroupingNode` is a *wrong verb on a sound
document* and deliberately does not fall back to `delete_field` —
*"the two remove different amounts, and guessing which the caller meant
is exactly the sneakiness rule 4 forbids on a destructive verb."* This
asserts the shell inherits that rather than arming something.

It is unreachable from the panel, which only ever passes names out of
`AcroForm::groups`. Asserted anyway: the day a caller passes a field
name, the operator must get nothing armed rather than a confirmation
block describing a deletion of the wrong size.

### `struct Armed`

Carries the engine's own [`FieldGroupDeletion`] verbatim rather than a
flattened set of counts, because the panel needs the **names** as well as
the numbers and because re-shaping the report here would be a second
vocabulary for facts the engine already has words for.

### `fn armed`

**The panel's read** — see [`crate::panels::forms::groups`]. Returns `None`
when nothing is armed, or when the armed preview was taken against a
revision the document has since moved off.

### `fn delete`

# The disclosure is built from the ENGINE'S REPORT, not from the preview

`delete_field_group` returns a [`FieldGroupDeletion`] whose `nodes_removed`
is *"what the cascade ACTUALLY emptied, not a prediction"* — core replaces
the preview's figure with the truth and keeps a `debug_assert` for the day
the two disagree. Building the sentence from the report rather than from the
armed preview means the operator reads what happened, on the day those two
stop agreeing, instead of reading what was expected to.

# Rule 4, and why the sentence is owed rather than optional

Deleting a grouping node changes **nothing an operator can see**. The page
is identical, the canvas is identical, the raster is identical; a form field
is not drawn as such and a grouping node is not drawn at all. The disclosure
is not a courtesy on this verb, it is the only evidence that the press did
anything — which is why it names all three counts and the group.

# The armed preview is not cleared here, and does not need to be

A successful deletion bumps the epoch through `vector_edit`, and [`armed`]
filters on the epoch. A *failed* one does not bump it, so the preview
survives a refusal — which is right: the operator is looking at a block
describing a group that is still there, beside a sentence saying it was not
removed.

# The refusal is worded HERE, because `vector_edit`'s refusal arm only
traces

That arm's own comment is explicit about it: a refusal *"is deliberately not
routed"* to the disclosure row, because a disclosure is after-the-fact and a
decline is not, and *"sharing one slot would mean an undone gesture and a
completed one wearing the same wording in the same place."*

The argument is right about the **slot** and it does not license a silence
on this verb. Every consequence of a grouping-node deletion is off-canvas,
so an operator who has just read a list of four field names and pressed a
button labelled *"Delete 4 fields"* has **no evidence at all** of what
happened — a success and a refusal are the identical screen. A trace line is
not a disclosure; the operator cannot read it.

So this follows `import_data`'s precedent — the one other form verb that
words its own decline through [`crate::app::actions::record_note`] — and
resolves the slot-sharing hazard in the **wording** rather than by adding a
second mechanism: [`crate::text::forms::field_group_delete_refused`] says
*"the form is unchanged. Nothing was removed."* in the sentence itself, so
it cannot be misread as a report of a completed act however it is placed.

The epoch is captured **before** the call and is the right stamp on either
outcome: a refusal does not move it, so the sentence is current; a success
moves it, and the success path's own disclosure is stamped with the new one
by `vector_edit`.
