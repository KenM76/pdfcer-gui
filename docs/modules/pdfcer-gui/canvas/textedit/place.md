# `canvas::textedit::place` — where a press puts the caret

## The seam

Split out of [`super`] on 2026-08-21 under R2, when the text box took that
file past the 1,500-line ceiling. It is the seam the file already drew with
its own banner — *"Starting a draft"* — and it is a real subject rather than
a size-driven cut: everything here answers **where does a press put the
caret**, and nothing here knows what typing does afterwards.

## The two gestures, and why they are two

| gesture | anchor | what commits |
|---|---|---|
| **click** on existing text | [`Anchor::Run`] | `edit_text` — one show operator's text replaced |
| **click** on bare page | [`Anchor::Origin`] | `add_text` — one single-line run at a point |
| **drag** a rectangle | [`Anchor::Box`] | `add_text` boxed — a wrapped paragraph |

The third arrived on 2026-08-21, on the operator's *"I should be able to
make it multi line."* It has to be a drag, and the reason is the file format
rather than a preference: **a PDF has no paragraph.** Each visual line is its
own show operator at its own absolute position, so something must decide
where the second line starts — a width to wrap against — and a width is a
rectangle somebody draws.

## What this module refuses, and why each refusal is a sentence

[`Refusal`]'s variants are shown on the status row, never dropped. That is
`DEFECTS.md` D4a's whole lesson: the old shell's answer to a caret it could
not place was a boolean and a keyboard that stopped responding, and the
operator reported the feature as broken for weeks.
