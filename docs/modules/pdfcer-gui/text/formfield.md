# `text::formfield` — every string the form-field placement dialog shows

One area of the catalog described in [`crate::text`]'s header, covering
[`crate::dialogs::formfield`] — the pop-up that collects a control's details
after it has been placed on the page.

It sits beside [`crate::text::forms`] rather than inside it because the two
answer opposite questions. That file is about **filling** a form somebody
else authored, and its copy is dominated by disclosures about what a document
does or does not support. This one is about **authoring**, and its copy is
dominated by labels for choices the operator is making right now.

## The vocabulary rule this file follows

Every label here is the word the operator's other programs use, not the word
the PDF specification uses. The standing tie-breaker — *make it work the way
other programs do* — applies to vocabulary as much as to behaviour, and the
spec's names for these things are unusually bad for a UI:

| spec | here | why |
|---|---|---|
| choice field (`/Ch`) | drop-down list | nobody outside the spec says "choice field" |
| `/TU` | tooltip | "alternate field name" describes the mechanism |
| `/AS` on state | value when ticked | "on state" is a name in a dictionary |
| comb | equal cells | the word means nothing; the picture is obvious |

## What is deliberately NOT here

The field-name stems (`Text`, `Check Box`, `Group`, …) that auto-generated
names are built from. Those are `/T` strings written into the file and keyed
on by form-filling scripts and FDF imports; translating them would rename
every field for an operator running a different language, invisibly, until an
import failed. They are literals on `FormFieldKind::name_prefix`.

## Item notes

### `fn only_the_radio_asks_for_a_group_name`

The wording that prevents the most common form-authoring mistake. Tested
rather than left to review because "Name" is the obvious label, it is
correct for four of the five kinds, and unifying them would look like a
tidy-up rather than a regression.

### `fn the_password_hover_does_not_imply_security`

A masked box reads as secure to anyone not told otherwise, and the value
really is plain text in the file. This is a false-claim guard, not a
style test.

### `fn no_string_here_claims_a_button_cannot_be_given_an_action`

It said pdfcer *"cannot yet give it something to do"*.
`set_button_action` shipped on 2026-08-30 and that sentence stayed on
screen for two days, because **nothing in this repository fails when a
capability lands**. The engine's own reply had warned in as many words:
*"if your surface tells the operator that pdfcer never authors an action,
it is now saying something untrue in the direction that matters."*

The replacement is `text::buttonaction`, which says what the button WILL
do. What is asserted here is the guard that would have caught the
staleness: **no string a push button's rows draw may claim a button
cannot be given an action.** A sentence that reintroduces the claim
fails here rather than shipping.
