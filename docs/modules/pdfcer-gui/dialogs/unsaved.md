# `dialogs::unsaved` — the question `file.close` has been promising to ask
since it shipped

## The defect


> *"Close the document. **You are asked what to do about unsaved edits
> first.**"*

Nothing asked. [`crate::app::actions::Action::Close`] consulted
`PdfcerApp::save_pending`, which is permanently `false` by design, and then
called `close_document()`, which sets `Status::Empty` and drops the
`EditSession`. **Every edit made since the file was opened was discarded,
silently, with no prompt and no undo.** The same held for
[`crate::app::actions::Action::Open`], `New` and `NewSized`, each of which
replaces the open document.

It is the worst defect this project has found: it destroys work, it destroys
it on the operator's own instruction so it never looks like a crash, and the
surface **told them it would not happen**.

## Why `save_pending` was not the bug, and must not become the fix

The obvious repair is to make `save_pending` return `edit_epoch != 0`. That
would be wrong, and `crate::app::lifecycle`'s own header says why in
advance: `save_pending` asks *"is a save **in flight**"* — is there a moment
at which the bytes on disk are a partial revision and the `EditSession` the
writer is reading from must not be dropped. `file.save_copy` is
**synchronous**, entered and finished inside one `apply` call with no frame
drawn in between, so that state genuinely cannot occur and the honest answer
genuinely is `false`.

*"Are there unsaved edits?"* is a **different question with a different
answer**, and conflating them would have broken the live consumer that
module names: `dialogs::ocr`'s `UnsavedEdits` refusal reads `edit_epoch != 0`
directly, for the good reason that a successful save-a-copy leaves the
document exactly as unsaved as it was — *the copy went somewhere else*.

So this is a **second** predicate beside the first, not a redefinition of it,
and the two guards compose: a save in flight declines outright; unsaved edits
ask. See [`PendingIntent`].

## The button that is not "Save"

Every three-way close prompt an operator has ever seen offers *Save · Don't
save · Cancel*, and this one **cannot**, because this build has no Save.
`file.save` is in `crate::shell::manifest::PLANNED`, blocked on autosave and
crash recovery; the only writer is `file.save_copy`, which writes a **new
file somewhere else** and leaves the open document untouched and still
unsaved.

Labelling that button *Save* would be the same class of lie as the tooltip
that started this: an operator would press it, see a file-save dialog, name
a file, press Close, and find that the document they were editing still has
its original contents on disk. They would have lost nothing — the copy is
real — but they would believe something false about which file their work is
in, which is worse than losing it *and knowing*.

So the button says **"Save a copy…"**, and the sentence beside it says what
that means for the file they came from. When `file.save` lands, a fourth
button joins it and this paragraph gets shorter; nothing else here changes.

## Why cancelling the file picker cancels the whole thing

[`Outcome::SaveCopy`] runs the save and **only resumes the intent if a file
was actually written**. A cancelled picker means the operator changed their
mind mid-transaction, and the least surprising reading of that is *"leave my
document alone"* — not *"close it anyway, unsaved"*, which would be a
destructive act reached by pressing Cancel.

## Why the dialog is not a `Modal`

egui 0.35 has `egui::Modal`, and this is the one surface in the crate with a
genuine claim on it. It is deliberately not used, for the reason every
dialog in [`super`] is a `Window`: this crate has exactly one dialog idiom,
`ui-verify` drives all of them the same way, and a second idiom introduced
for one surface is a second set of layout, focus and escape behaviours to
get right. What matters here is not modality but that **the destructive
action does not happen until a button is pressed**, which is a property of
the control flow, not of the window.

## Item notes

### `fn an_unedited_document_is_not_asked_about`

The property that keeps this from being a nag. `edit_epoch == 0` is the
state a document is in from the moment it opens until the first edit
lands, which is most of the time an operator spends with a file open —
and a confirmation on every close of an unread drawing is exactly the
"nagging" the operator named as having made the old shell worse.

### `fn the_four_intents_do_not_share_a_sentence`

Asserted as a **relation** rather than against the literals: the point
is that no two intents share a sentence, because an operator who pressed
Open and is asked about closing will read the prompt as being about a
control they did not touch. Comparing against the strings themselves
would pass just as well if all four returned the same one.

### `fn both_kinds_of_new_ask_one_question`

They are one act with two entry points — `dialogs::new_document` is the
size chooser in front of the same replacement — so asking two different
questions about them would be describing a distinction the operator
cannot see.

### `fn the_answer_fires_once_and_carries_its_intent`

The second `take` returning `None` is what stops the owner resuming the
intent on every frame after one press — which on `PendingIntent::Open`
would re-open the same file forever, and on `Close` would fight anything
the operator opened next.

### `fn an_answer_is_visible_until_it_is_taken_and_not_after`

It is asserted against `take_outcome` rather than alone, because the
property that matters is that the pair agrees about what "parked" means:
`true` too late keeps an answered window on screen forever, `false` too
early loses the operator's decision.

### `fn cancelling_answers_nothing`

The two have to be separable: a window that closed *and* answered would
make the ✕ destructive, and the ✕ is the control an operator presses
reflexively to make a surprise go away.

The [`Self::answered`] half is what lets [`crate::dialogs::retire`]
drop a dismissed window on the frame it closes rather than holding it
open waiting for an answer that is never coming.
