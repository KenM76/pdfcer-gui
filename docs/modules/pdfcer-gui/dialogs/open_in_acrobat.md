# `dialogs::open_in_acrobat` — the question that comes before pdfcer lets
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
