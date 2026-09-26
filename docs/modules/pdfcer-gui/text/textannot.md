# `pdfcer-gui/text/textannot`

## Item notes

### `fn the_printing_distinction_is_stated_in_both_directions`

The one property this module exists to hold. A text box prints and a
sticky does not, and an operator who has them backwards has either
published a private remark or hidden a public one — neither recoverable
by noticing afterwards.

Asserted on the words rather than trusted to review, because the two
controls are otherwise identical and a copy-paste between them would be
invisible in a diff.

### `fn every_offered_stamp_size_is_named_distinctly_and_carries_its_unit`

[`every_offered_stamp_is_named_distinctly`]'s argument, with one clause
that gallery does not need: a combo shows exactly one entry when it is
closed, so a duplicate label there is not merely confusing — the
operator cannot tell which of two entries is currently selected, and the
control silently stops reporting its own state.

The unit clause is the one that would actually fire. `"12"` and
`"12 pt"` are both plausible things for a future edit to produce, they
are distinct from each other, and only one of them is readable in a list
whose first entry is a sentence.

### `fn the_size_disclosure_states_both_directions`

The operator drew a rectangle. Telling them it may widen, without
telling them it will never be shrunk, leaves them believing a stamp
might come out smaller than the space they cleared for it — which on a
title block is the difference between using the control and not.

⚠ Asserted on the words rather than on the behaviour because this is
the `text` crate: the behaviour is `StampFit::GrowToText`'s, tested
where it is called. What is tested here is that the sentence still
describes it. A future edit that softened this to *"the stamp resizes
to fit"* would be true of `ShrinkToBox` too, and this shell does not
offer `ShrinkToBox`.

### `fn only_a_foreign_text_box_appearance_owes_the_operator_a_sentence`

The guard on the section banner's table, and it is deliberately built
**positive first**. The engine's own methodology note, sent with
`Pass 258.1` and aimed at this side of the wire:

> Our first version of the foreign-appearance test asserted
> `!appearance_rebaked` — and **passed with the entire re-bake
> disabled**. A "not X" assertion is vacuous when the thing that would
> produce X is absent.

So the first four assertions here establish that these functions *can*
speak, and every `is_none()` below them is a claim about a condition
rather than about a function that never returns anything. Sabotaging
either half of the guard — deleting the `paints_its_note` term or the
`!appearance_rebaked` term — turns this test red, which is the property
a vacuous version would not have.

The two silences are owed in *opposite* directions and both are
expensive:

- a sticky or a stamp that started warning "the words printed on the
  page were NOT redrawn" describes a page that never showed those
  words, and a warning an operator learns to dismiss costs the one case
  where it is true;
- a re-baked text box that warned anyway would be the deleted lie
  restored, on a build where the page demonstrably moved.

### `fn both_disclosures_state_the_half_that_did_not_move_and_why`

The negative clause was the whole point of these sentences when they
were written and it still is: a disclosure reading only *"the comment
was changed"* is true, is what the operator already knows, and leaves
the whole of the surviving half unsaid.

The *why* clause is new, and it is what stops the sentence reading as
a defect report. On the old build the picture did not move because
pdfcer could not move it; on this one it did not move because moving it
would have thrown away a shadow, a gradient or an image that another
program drew — preservation, not failure. Asserted on *"another
program"* rather than on a verb, because the verb is rewordable
(`keeps`, `leaves`, `preserves` are all honest) while the **cause** is
not: drop that clause and the sentence reports a capability pdfcer is
missing instead of a decision it made, which is the whole difference
between this sentence and the one it replaced.

Checked on `NOT` in capitals for the same reason: it is the only token
that cannot survive a trim down to the cheerful half.

### `fn intro`

Each says **where the words end up**, which is the distinction this
module exists to preserve. The text box's says "on the page"; the sticky's
says the opposite in as many words, because an operator who believes a
sticky prints will use it for something that needed to.

### `fn bound`

Both name the engine limits stated when the burn-in landed: the face is
Base-14 Latin, so anything outside it becomes a question mark. The text
box's also names wrapping, because that is what makes the box's width a
choice rather than a formality.

### `fn stamp_label`

Title case, not the `/Name` spelling. The PDF carries `/NotApproved`;
nobody says that out loud, and a gallery that listed it would be showing
the operator the file format rather than the choice. The engine's own
appearance paints its own label — this is only how the option is *offered*.

The catch-all is required because `StampName` is `#[non_exhaustive]`, and
it returns the empty string rather than a guess: a stamp this catalog has
no prose for is one `crate::canvas::textannot::STAMPS` does not list, so
the arm is unreachable from the gallery and inventing a label would be
prose pdfcer made up about somebody else's addition.

### `fn sticky_icon_label`

Title case with a space, not the `/Name` spelling. The PDF carries
`/NewParagraph`; the operator is choosing a picture, not a name object, and
a chooser listing the run-together form would be showing them the file
format. The same rule [`stamp_label`] follows, and the same rule
`text::commands` states for every label in this shell.

# No catch-all, unlike [`stamp_label`], and the difference is the enum

`StampName` is `#[non_exhaustive]`, so that function needs a `_ =>` arm and
returns the empty string for a stamp it has no prose for. `StickyIcon` is
**not** `#[non_exhaustive]`, so this
`match` is exhaustive and an eighth icon is a **compile error here** rather
than a blank line in a chooser. That is the stronger arrangement and it is
available only because the engine's enum is closed — worth saying out loud,
because the two functions sit next to each other and look like they should
be shaped the same.

⇒ `crate::canvas::textannot::STICKY_ICONS` carries the same guarantee from
the other side, by list rather than by prose.

### `fn sticky_icon_heading`

*"Icon"* rather than *"Name"*. `/Name` is the format's word for the key
and it is also the word for four other things in a PDF; the operator is
picking a picture. `text::commands`' standing rule: a label is the
operator's vocabulary and an id is the format's.

### `fn sticky_icon_bound`

# Why this sentence has to exist

The engine's sticky author takes the icon and uses it in exactly one
place: `/Name`. The appearance it bakes is **the same dog-eared page glyph
for all seven**, and that is a decision the engine documents rather than an
omission — `annot_author` records it as *R44 choice (a)*, trade-dress
avoidance: *"pdfcer authors its OWN plain marker … never a reproduction of
Acrobat's icon set"*.

⇒ So an operator who picks *Key* sees no change in pdfcer and a key in
Acrobat. Without this sentence they meet that difference in the other
program, on a file they have already sent somebody. **It is not an R8b
breach** — applied content still renders exactly as saved content will, in
*this* reader, which is what R8b is about — and it is exactly the sort of
gap between the file and the picture this shell states rather than lets
somebody discover.

It says *"other PDF readers"* and does not name Acrobat. Naming a
competitor's product in operator copy is a claim about that product's
behaviour, and this one is true of every reader that ships Table 172
artwork rather than of one.

### `fn stamp_bound`

A disclosure rather than a limitation apologised for. Acrobat's *dynamic*
stamps bake a name and a timestamp into the appearance; pdfcer has no
identity to put in one, and the note-text exchange settled that this shell
invents no placeholder for `/T`. So a stamp claiming to be signed by
somebody would be a claim pdfcer cannot support — and saying so is cheaper
than an operator looking for the feature.

### `fn stamp_size_heading`

*"Size"* and not *"Font size"*. The operator is choosing how big the
stamp reads on the sheet; the fact that a stamp's appearance is drawn with a
font is pdfcer's business. The same rule [`sticky_icon_heading`] follows one
screen up — a label is the operator's vocabulary and an id is the format's.

### `fn stamp_size_label`

# Why the default entry is a SENTENCE and the others are numbers

[`StampSize::FitTheBox`] is not a size, it is a **policy** — *the box you
drew decides*. Listing it as a number would be a lie about what it does, and
listing it as `"Auto"` would be the file format's habit of naming a
behaviour after the fact that nobody typed a value. It says what happens.

⚠ The point sizes carry the unit. `"12"` in a list whose first entry is a
sentence reads as an item number; `"12 pt"` cannot.

### `fn stamp_size_bound`

# Why this sentence has to exist

The operator drew a rectangle and is now told a number may override it. Two
facts about that are true, neither is guessable, and one of them is a
visible change to their drawing:

  * **The box grows if the words do not fit.** Engine
    `StampFit::GrowToText`, which every entry in this chooser pairs with —
    see `crate::canvas::textannot::StampSize`'s header for why exactly one
    of the three fit policies is offered. So the rectangle is a position and
    a minimum, not a cage.
  * **The box is never shrunk.** A 12 pt label in a large box leaves the
    box large, which is what a caller who drew a deliberately big stamp
    meant.

It is **not** a disclosure of an inference under R8b rule 4, and that
distinction matters because the sentence looks like one. A grown box is
visible on the canvas *as itself* — the operator sees a wider stamp, and a
screenshot of it matches a screenshot of the saved file. This sentence is
told **before** the act, so the operator knows what the drag means, which is
the opposite end from a report about something already done.

### `fn accept_disabled`

Names the kind, because "type something first" is unhelpful next to a
gallery and impossible next to a stamp — the message only ever appears for
the two kinds that take typing, and it says which one it is about.

### `fn note_edit_disclosure`

`appearance_rebaked` is [`pdfcer_core::edit::MarkupNoteChange::appearance_rebaked`]
passed through unexamined — the engine's answer to *which half moved*, not
an inference of ours. See the section banner for the four-row table this
implements and for why `false` alone is not the condition.

States the two halves separately and in that order — what did change,
then what did not — because an operator who reads only the first clause
must not come away believing the page moved. It also says **why** the
picture was left alone, since "it was not redrawn" without "because it is
not ours to redraw" reads as a failure rather than as preservation.

### `fn note_clear_disclosure`

A separate string from [`note_edit_disclosure`] rather than a shared one,
for this module's standing reason: the two describe different acts, and a
shared sentence is one an author can reword for one and silently change for
the other. Removing is also the worse surprise — the comment is gone from
every list and the page is unchanged, so the operator has deleted the only
copy they could see and kept the one they cannot.

### `fn placed_unencodable`

# Why a substitution the operator can SEE still owes a sentence

The Base-14 fonts a stamp label and a text box are drawn in are encoded in
`WinAnsi`, which has no code point for an em dash, a curly quote, an
accented character outside Latin-1, or anything at all in Greek, Cyrillic
or CJK. The engine does not refuse those: it substitutes `?` and reports
how many, which is the right call — refusing a whole stamp because one
character came out of a word processor's autocorrect would be worse than
drawing it with a `?`.

The `?` is on the page for anyone to see, so this looks at first like
the kind of visible outcome R8b rule 4 says owes nothing. **It is not**,
and the distinction is the one that rule turns on: what the operator
cannot see is *that pdfcer did it*. A `?` in the middle of a stamp reads
as a typo they made, or as a rendering fault, and neither sends them
anywhere useful. The sentence names the cause and the count, so the act is
attributable to the program that performed it.


A separate sentence from the forms one rather than a shared helper, for
this module's standing reason: the two describe different acts on
different objects, and a shared sentence is one an author can reword for a
form field and silently change for a stamp. The forms sentence also names
the FIELD, and an annotation has no counterpart for that.
