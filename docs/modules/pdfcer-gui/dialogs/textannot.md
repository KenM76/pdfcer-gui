# `dialogs::textannot` — the words half of a text-bearing annotation

The second half of the place-then-type gesture. The canvas has taken a
rectangle (or a point); this asks what goes in it, and **nothing reaches
the document until Accept.**

## ★ Why a dialog, when markup authors on release

`crate::dialogs`' header draws the line: *"a dialog is a single transaction
with a start and an end… a panel is somewhere an operator dips in and out
of while working."* Typing a callout is unmistakably the first — it begins
when the box is drawn, it ends when the words are accepted or abandoned,
and there is nothing to dip back into afterwards.

The alternative — an in-place editor drawn over the page, the way a word
processor would — was rejected on the standing rule that **nothing floats
over the canvas** except the Find bar, which is a documented exception the
operator granted for one surface. It would also have needed a caret, a
selection and a hit test over text this shell does not own, which is a text
editor rather than a dialog.

## ★ It is deliberately NOT modal to the document

The reference line stays drawn, the page stays where it was, and the dialog
is `default_pos` rather than anchored so it can be dragged aside. An
operator writing a callout is usually looking at the thing they are calling
out, and a window pinned over it would make them close the window to read
what they were annotating.

## The three kinds meet three different questions

| kind | what this asks |
|---|---|
| text box | *what should it say?* — a multi-line field, because a callout wraps |
| sticky note | *what is the note?* — the same field; the words live in a popup rather than on the page, and the window says so |
| stamp | *which stamp?* — a gallery, and **no text field at all** |

The stamp's absence of a field is the important one. `manifest/markup.rs`
recorded the blocker as *"a stamp with no chooser has no operand"*, and the
converse is just as true: a stamp with a free-text field is a text box with
a border, and offering both would be two controls for one feature with no
way for an operator to tell which they wanted.
