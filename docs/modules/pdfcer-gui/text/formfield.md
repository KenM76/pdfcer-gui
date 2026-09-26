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

### `fn name_label`

It reads differently for a radio button, and that is the most important
wording decision in this file. For every other kind the name identifies
**this control**; for a radio it identifies **the group**, and two radios
sharing it is what makes them exclusive. An operator who reads the same
label on both will place three radios that are all separately tickable and
wonder why.

### `fn tooltip_note`

Not a warning and not conditional on the box being empty: it is a fact about
what a tooltip *does*, which is entirely invisible on screen. Rule 4's
surviving half asks for exactly this — report what cannot be seen, and do
not nag about it.

### `fn password_hover`

Salvaged in substance from the old shell's `form_field_password_tooltip`,
which exists because a masked box reads as "secure" to anyone not told
otherwise. It is not: the value is stored as plain text in the file, and
anybody with the file can read it. Getting this wrong is the difference
between a UI convention and a false security claim.

### `fn radio_group_note`

The single sentence that stops the most common form-authoring mistake:
placing three radio buttons with three different names and getting three
independent tick boxes that happen to be round.

### `fn sort_hover`

Worth a hover because the answer is surprising: the flag asks the *viewer*
to sort, so what the operator typed and what a reader sees can differ, and
pdfcer is not the one doing it.

### `fn required_hover`

Not in pdfcer. The flag is a request to whatever software submits the form,
and nothing stops a document being saved with the field empty — which is
worth saying, because "required" reads as a guarantee.

### `fn background_remove_entry`

Worded as what the box will look like rather than as undoing a choice,
because before placement there is no file yet for a key to be removed from
— what the operator is picking is how the box will arrive.

### `fn border_colour_note`

Black, not *nothing*. `WidgetChrome::stroke` resolves an unstated `/BC`
AND an empty one to the same black, so the honest sentence is that the
outline will be drawn — and the way to have none is a border width of 0,
which is the control directly above this one. Because those two states are
indistinguishable before placement, one sentence covers both rather than
two that would have to claim a difference the engine does not make.
