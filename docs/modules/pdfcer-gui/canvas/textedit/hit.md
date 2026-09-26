# `canvas::textedit::hit` — where the pointer is inside the editor box

## What this is

One fact, published once a frame by [`super::paint`] and read by everything
that needs to know whether a pointer event belongs to the draft: **the
editor box's rectangle, and the galley that was drawn inside it.**

## Why the galley has to be shared rather than re-derived

Because *"which character is under the pointer"* and *"where is the caret
drawn"* must be the **same** derivation, and this module exists to make that
true by construction rather than by two functions agreeing.

The alternative is to lay the draft out a second time in the click handler.
That looks identical and drifts the moment anything differs — a wrap width
computed slightly differently, a font resolved on a different frame, a
`TextStyle` read from a `Ui` with different spacing. `super::paint`'s header
already records what that cost once: a caret derived from the *page's* glyph
advances while the text was drawn in the shell's font, drifting further the
more the operator typed, and the fix was to delete the second derivation.

So the galley that was **drawn** is the galley that is **hit-tested**, and
`Galley::cursor_from_pos` is the inverse of the `Galley::pos_from_cursor`
the caret is painted with. One layout, two questions.

## Why a frame late is not a bug here

`paint` runs after `interact` in the frame, so a pointer handler reads the
rectangle and galley **the previous frame** produced. That is correct rather
than tolerated:

- The operator can only press on something they can **see**, and what they
  can see is the previous frame.
- A press on the frame the editor box first appears has nothing to hit, and
  answering `None` there is right: the box was not on screen when the button
  went down.

It is the same argument `canvas::markup::ink` makes for reading the gesture
machine's answer before advancing it, and the opposite of a stale-coordinate
bug — the coordinate is *deliberately* the one the operator was looking at.

## What this deliberately does not do

It does not decide what a press **means**. That is [`super::keys`] for a
draft and `canvas::clicking` for the page, and both ask this module the same
question and act on it differently.

## Item notes

### `fn index_at`

Clamped by the galley: a position above or left of the text answers 0, a
position past the end answers the length. That is what every text field
does and it is what makes a drag that runs off the end select to the
end rather than stopping.

`CCursor::index` is a **character** index, which is the unit
`super::Draft::caret` is documented in. `Galley` also speaks in rows and
byte offsets; taking either would compile and would put the caret inside
a multi-byte character on the first document with an accent in it.

### `fn read`

# Why nothing clears this

Because every caller asks *"is the pointer inside the box"* and a stale
rectangle from a draft that has ended answers that question wrongly only if
the pointer is inside a box that is no longer drawn — and the callers all
gate on a **live draft** first, which is the fact that actually ended. A
clear step would be a second place to forget.
