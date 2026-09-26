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
