# `acrobatprompt` — the question that comes before pdfcer lets
go of the file

`OPERATOR_REQUESTS.md` **O122**, points 5 and 6:

> *"When clicked it will check if the file has been changed (forms filled
> out for example, etc) and ask to save changes first, but if it hasn't
> changed it will note the file will be closed when opened in acrobat with
> and ok button to continue - there will be a cancel button as well."*

## Why there is a dialog at all

Because pressing this button **closes the operator's document**, and closing
somebody's document is not something to do quietly on one click.

The closing is not incidental. Acrobat takes its own lock on the file it
opens, and two editors on one PDF is how an afternoon's work disappears:
pdfcer writes its revision, Acrobat writes its own from a copy it read
before pdfcer saved, and neither program ever reports an error because
neither did anything wrong. The only defence that holds is for exactly one
program to have the file at a time — so pdfcer gives it up. That is the
operator's own instruction and it is the right call; the window exists to
say so before it happens.

## The button that is deliberately NOT here

[`crate::dialogs::unsaved`] offers *Save a copy… · Close without saving ·
Cancel*, and it is right to: there, the document is merely being closed, and
an operator who genuinely wants to abandon an experiment should be able to.

**This window offers no such thing**, and the reason is specific rather than
a general dislike of destructive buttons. Discarding here would not simply
lose the edits. It would close the document, hand Acrobat the file *as it
was before the operator started*, and leave them looking at their work's
predecessor in a program that will happily save over the original. The next
thing they press in Acrobat overwrites the very edits pdfcer discarded — so
the third button would not be *"lose this"*, it would be *"lose this, and
then bury the evidence"*.

So: **save and open**, or **cancel**. Two answers, both of which leave the
operator's work intact.

## The three shapes this one window takes

| [`crate::acrobat::Prompt`] | Heading | Buttons |
|---|---|---|
| `SaveFirst` | *Save your changes first?* | **Save and open in Acrobat** · Cancel |
| `ConfirmClose` | *This document will be closed.* | **Close and open in Acrobat** · Cancel |
| `NoFileOnDisk` | *This document has never been saved.* | Close |

One window rather than three because it is one question — *shall I hand
this over?* — asked of a document in three different states, and three
windows would be three sets of layout, focus and escape behaviour to keep
in step. The [`crate::acrobat::Prompt`] that decides which shape is a pure
function of two booleans, so *which* shape appears is asserted in
`crate::acrobat::tests` without a window.

The third row has **one** button and no Cancel. There is nothing to
cancel: nothing is going to happen either way, and a Cancel beside a
refusal invites the reading that the other button would have proceeded.

## Why it is a `Window` and not an `egui::Modal`

[`super`]'s standing answer, unchanged: this crate has exactly one dialog
idiom, `ui-verify` drives all of them the same way, and a second idiom for
one surface is a second set of behaviours to get right. What matters is not
modality but that **the destructive act does not happen until a button is
pressed**, which is a property of the control flow rather than of the
window.

## Item notes

### `const REGION_PROCEED`

One name for both, deliberately, where [`crate::dialogs::unsaved`] gives
its Save and its Save-in-place separate ones. There, the two buttons do
different things to the file on disk and a check that could not tell them
apart would pass on a build that swapped them. Here they do the *same*
thing — hand the document over — and differ only in whether a save happens
first, which [`REGION_SAVE_FIRST`] publishes on its own.

### `const REGION_SAVE_FIRST`

Its absence is the assertion a driven check wants: a run over a clean
document must show no such region at all, which is what tells "the
operator was asked to save" apart from "the operator was asked to confirm"
— two states with very similar screenshots.

### `const REGION_CANCEL`

Published by the two shapes that HAVE a Cancel and by neither the third
nor its dismiss button. The never-saved refusal offers no decision, so a
check that found a cancel region there would be asserting a choice the
operator was never given.

### `enum Outcome`

One variant, because there is exactly one way forward. Cancel parks no
outcome — see [`OpenInAcrobatDialog::was_cancelled`] — and the refusal
shape has no way forward at all.

A one-variant enum rather than a `bool` or a bare `()`, and it earns its
keep at the call site: `Some(Outcome::Proceed)` reads as an instruction,
where `Some(true)` would read as an answer to a question the reader has to
go and find.

### `fn was_cancelled`

Read by the owner for [`crate::dialogs::unsaved::UnsavedDialog::was_cancelled`]'s
reason: a Cancel parks no outcome, so a drain reports nothing, which is
indistinguishable from *"they have not answered yet"*.

### `fn answered`

The twin of [`crate::dialogs::unsaved::UnsavedDialog::answered`],
and it is here because this window carries the **same latent defect**
its neighbours shipped: [`Self::show`] answers `false` on the very
frame a button is pressed, and an owner that read that `false` as
*"this dialog is finished"* would drop the dialog — with the outcome
still inside it — before the application could take it out. The symptom
would be a *Save and open in Acrobat* that closes the question and does
nothing else, which reads as the whole application ignoring the
operator.

See [`crate::dialogs::retire`] for the rule all three now obey.

### `fn take_outcome`

Returns both, because the owner needs both and holding them apart
would let a future edit drain one without the other — which would
launch a viewer the operator was not asked about.
