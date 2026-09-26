# `text::offpage` — the words for **content that is in the file but not on
the sheet**

The copy for [`crate::dialogs::offpage`], and the disclosure half of the
off-page feature O23 opened.

## What this window is FOR, in the operator's own words


> *"how do I view and edit objects that are off of the page? we added this
> feature but I didn't see how to enable it."*

The **view** and **edit** halves shipped — a halo raster that paints past
the sheet, a pasteboard that lets the view centre on a point far off it, and
a cull that stopped throwing the page away when its sheet left the screen.
What none of them answer is the question that comes **first**:

> **Which of my drawings have marks outside the sheet at all?**

An operator cannot go and look at a thing whose existence they have no
reason to suspect. That is what this window is: a census, and a way to cut.

## Why this is a *Protect* control and not a *View* one

Because off-page content is the classic PDF leak, and on a CAD drawing it is
not a hypothetical one. A sheet exported with the drawing border cropped
down, a superseded revision note parked outside the frame, a customer's name
on a title block that was moved off rather than deleted — **none of it
renders, and all of it is still in the file.** It is still extractable by any
text search, still copied by any text export, and still there when the file
is emailed.

⇒ So the sentences below lead with **what is still readable**, never with a
count. `pdfcer_core::offpage::OffPageObject::text` carries the recovered
string precisely so this module can say it, and the engine's own doc comment
makes the argument: *"there are 4 off-page objects"* invites a shrug, and
*"one of them reads `SUPERSEDED — DO NOT BUILD`"* does not.

## Rule 4 — this window marks nothing on the canvas

Not one sentence here is drawn over the document, and pressing the mark
button produces `/Redact` annotations that render exactly as any other
`/Redact` annotation does. There is no provisional tint, no dashed halo, no
"off-page" badge. The disclosure is this window; the canvas is the document.

And the removal is a **mark**, not an apply. `edit.redact_apply` remains
the only place content is destroyed, which is the arm/mark/obliterate split
`crate::shell::commands::catalog::edit` argues at the redaction family's
registration. A control that scanned and deleted in one press would be a
fourth member of that family that skipped its middle step.

## Why every function here takes primitives rather than the engine's types

`pdfcer_core::offpage::PageScan` and `OffPageObject` are `#[non_exhaustive]`,
so nothing outside `pdfcer-core` can build one — which would make every test
below impossible to write, and a copy module whose sentences cannot be
asserted is a copy module that drifts. The one engine type that IS passed
through is [`OffPage`], because it is an enum: its variants can be named from
here, and describing "entirely off" as "crossing the edge" is exactly the
mistake a `bool` would invite.

## Item notes

### `fn the_progress_line_counts_from_one_and_never_overshoots`

Falsified per clause, on the boundary that matters: before any page has
been checked the line must say page **1**, not page 0 and not page 2.
A one-off here is the kind of defect nobody reports and everybody sees.

### `fn the_undo_note_carries_the_step_count`

"Undo takes these back one page at a time" without a count leaves the
operator pressing `Ctrl+Z` an unknown number of times; the count is what
turns the sentence into an instruction.

### `fn a_row_for_off_page_text_quotes_the_text_before_anything_else`

This is the module header's whole argument, asserted rather than
described: an operator who reads "3 objects" shrugs, and one who reads
the words on the off-page note does not.

### `fn the_two_placements_are_told_apart_by_their_own_words`

Falsified per clause: this asserts that *fully* does not carry the word
the *partial* sentence turns on, and the reverse. An `||` across the pair
would assert neither.
