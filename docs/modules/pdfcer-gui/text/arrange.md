# `text::arrange` — every sentence the three ways of *arranging* something
on the page can owe

Three gestures share this catalog because they share a subject — **something
that is already on the page, moved without being redrawn**:

| gesture | module | what it changes |
|---|---|---|
| the **arrow-key nudge** | [`crate::canvas::moving::nudge`] | where the mark sits, by a whole point at a time |
| **Bring to front / Send to back** and their one-step twins | [`crate::app::dispatch::markup`] → [`crate::app::actions::reorder`] | which mark is drawn on top where two overlap |
| the **pointer drag** | [`crate::canvas::moving`] | where page content sits, continuously |


They are one file rather than two for the reason [`crate::text::rotating`]
and [`crate::text::resizing`] are two: those hold sentences for gestures
whose *arithmetic* differs and whose refusals therefore differ. These two
refuse for the **same three reasons in the same words** — no markup is
selected, the mark is locked, the mode may not author markup — and a reader
who found "this mark is locked" written twice, once per gesture, would be
reading the beginning of a drift.

## Why the lock sentence is NOT
[`crate::text::panels::properties::markup_locked`]

That one is on screen from the moment a locked mark is selected and it reads:

> *"This mark is locked by the document, so its **appearance** cannot be
> changed here. You can still delete it."*

It is a true sentence about **restyling**, which is the surface that draws
it. §12.5.3 Table 165 bit 8 is wider than that — *"do not allow the
annotation to be deleted or its properties (including position and size) to
be modified"* — so a nudge is refused by the same bit for a reason that
sentence does not state, and an operator who read *"its appearance cannot be
changed"* and then pressed an arrow would have been told about the wrong
half of the flag.

⇒ Borrowing it would have been the smaller edit and the misleading one. The
two sentences name the same bit and different consequences, which is the
case this project's *"one fact, one wording"* rule does **not** cover: the
fact is shared, the consequence is not.

Noted for whoever owns that string: it says *appearance* where the bit
says *appearance, position and size*. Correcting it is not this file's to
make.

## Why a refused nudge is worth a sentence at all, when a refused Delete
is not

[`crate::canvas::keys`]' Delete rung declines a locked annotation **to the
trace**, and its argument is good: the Properties panel is already saying
why, from the moment the annotation was selected, so a status line would be
the same fact arriving later and worse.

That argument does not transfer, and the difference is which sentence is on
screen. The panel's standing sentence for a locked mark is about *appearance*
(above) and its Delete row's is about *deleting*. **Neither one says a mark
cannot be moved.** So an operator who nudges a locked stamp has been told
nothing relevant before the press, and silence after it is the shape this
project keeps finding — a key that works everywhere else doing nothing here,
with no way on screen to learn why.

## The one contradiction inside this file, named rather than left
for an operator to find

[`not_a_markup`] answers an arrow key pressed on page content, and it says:

> *“The arrow keys nudge a selected markup. **Drag this with the pointer
> instead.**”*

[`run_has_no_position_of_its_own`] and [`run_would_drag_the_next_line`]
answer a drag on **one line inside a block of text** that this particular
file will not let pdfcer move. So there is exactly one selection — a text
run entered at the Part rung, in a document written that way — for which
the first sentence sends the operator straight into the second. Press an
arrow, be told to drag, drag, be told to press Escape.


**It is recorded and not “fixed”, deliberately.** The obvious repair is to
widen `not_a_markup` with an *unless it is one line of text* clause, and that
would make the common case — a whole object, a whole annotation, anything the
pointer really does move — read like a legal notice in order to pre-empt a
case the very next gesture explains properly. Widening a true sentence is
writing a new sentence, and the new one would be worse for everybody who is
not in the narrow case.

⇒ What the two owe each other is that they stay **findable together**, which
is what this file is for. Whoever reworks either one now has to read this
paragraph first.

## The rule every sentence follows

[`crate::text::rotating`]'s, inherited verbatim: **name the thing the
operator can see, never the thing pdfcer models.** They can see a stamp, a
cloud, a highlight and the order two of them overlap in; they cannot see
`/Annots`, an `ObjId` or an indirect reference. A sentence in the file
format's vocabulary reads as an internal error, whatever it says.
