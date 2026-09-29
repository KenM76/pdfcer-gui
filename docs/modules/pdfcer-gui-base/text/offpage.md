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

### `fn intro`

It states the **consequence** before the subject, because the subject
("objects outside the page box") is a thing an operator has no prior reason
to care about, and the consequence is the whole reason they should.

### `fn blank_overhang_note`

`PageScan::inkless_overhang` counts pictures whose placement crosses the
page edge but whose samples beyond it are all paper, or fully transparent —
the state `redact-offpage` leaves a picture it cannot move. The engine
decodes those samples and leaves such pictures out of `objects`, so a
cleaned sheet scans clean.

That is an inference the operator cannot see: a picture still straddles the
edge and is not listed. The sentence says so, once, in the results window,
and only when the count is not zero. It says pictures, because only images
are decoded; line work and text are judged by geometry alone.

### `fn object_row`

The recovered text comes FIRST when there is any, before the kind and
before anything else. See the module header: the string is the disclosure and
everything else is bookkeeping.

### `fn placement_word`

The wildcard arm is not laziness. [`OffPage`] is `#[non_exhaustive]`, so a
third kind of overhang the engine adds tomorrow must neither stop this shell
compiling nor be described as one of the two it is not.

### `fn kind_word`

A `match` rather than a capitalisation, so a token this shell has never met
renders as itself rather than as a mangled English word. The engine documents
`path`, `text` and `image`; anything else is new, and showing it verbatim
means a report about it names the real token.

### `fn unreadable_row`

*"No findings"* and *"I could not look"* must not print the same way —
the engine says so in `scan_document`'s own doc comment and returns the two
separately for exactly this reason. A page that will not decode is the one
place this window must not imply a clean bill.

### `fn scanning`

This window is the one place in this shell that does real work **after**
it has opened, one page per frame, because a whole-document decomposition of
the operator's benchmark drawing measures 469 ms per sheet and a 36-sheet set
would freeze the program for seventeen seconds. So the progress line is not
decoration: without it the window would sit there listing nothing while the
answer was still being computed, which reads exactly like "nothing found".

`done` is a count of pages already checked, so the page being worked on is
`done + 1` — the number is a position in the walk, not an index.

### `fn marked_undo_note`

Rule 4's surviving half. The engine records **one command per page**,
because there is no verb that authors redaction marks across a page range —
so one press of this window's button leaves the operator with N undo steps
rather than one, and a single `Ctrl+Z` takes back one sheet's marks and
leaves the rest. That is invisible: the marks all appeared at once, so
nothing on screen suggests they will not leave at once.

Returned only for a multi-page mark. On one page the sentence would be true
and useless, and a disclosure that fires when there is nothing to disclose is
how an operator learns to stop reading them.

### `fn marked_disclosure`

The count is of BANDS, not of objects, and the sentence says so. A page
with six off-page objects in one corner gets one band over that corner, and
an operator told "6 marks" who then counts four in the review list would be
right to distrust the program.

### `fn marked_skipped`

Second sentence, never instead of the first: the mark SUCCEEDED, and the
residual belongs beside that rather than in place of it. Rule 4's ordering,
the same as `crate::text::redact::mark_covers_image`'s.

### `fn marked_refused`

Distinct from [`marked_skipped`], which is about pages the **scan** could
not read, and the two can both be true of one press. This one means the scan
found off-page content on a sheet and `add_redaction` then declined to
author the mark — a certified document, an encrypted one, a degenerate band.
Collapsing them into one sentence would tell an operator to go and look at a
page for the wrong reason.
