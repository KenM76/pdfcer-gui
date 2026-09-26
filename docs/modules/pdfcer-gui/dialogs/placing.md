# `dialogs::placing` — **how a window offers to step aside**

`OPERATOR_REQUESTS.md` **O66**:

> *"anything we are inserting like this should have an option in its
> dialogue box to place it with the mouse instead of by positional
> co-ordinates."*

The dialog half of [`crate::canvas::placing`]. That module owns the gesture
and the pending record; this owns the button, the note, and the one derived
predicate that makes the whole arm safe.

## Opting in is three edits and no new state

```ignore
pub struct MyDialog {
    place: PlaceHandoff,          // 1. a field
    // …
}

// 2. in the body, beside the numeric controls
self.place.button(ui, PlaceKind::MyThing, REGION_PLACE);

// 3. the FIRST line of `show`, before the window is built
if self.place.hidden(ctx, PlaceKind::MyThing) { return true; }
```

★ Note what the third line does: it returns *"still open"* while drawing
nothing. The dialog is not closed — its drafts, its half-typed numbers and
its position are all exactly where they were — it simply is not built this
frame. That is the difference between stepping aside and being dismissed,
and it is the whole reason the operator's numbers survive the trip.

## ★★★ `hidden` is DERIVED, and that is the safety property

[`PlaceHandoff`] has **one** field, and it is not the hidden flag. Whether
the window is on screen is computed from
[`crate::canvas::placing::pending`] every time it is asked.

The alternative — a stored `hidden: bool` — is what the precedent this arm
generalises actually does, and it is broken. See `canvas::placing`'s header
for the Set-scale stranding case in full; the short version is that every
route out of a placement (Escape, a mode change, another tool, the document
closing) becomes a place somebody has to remember to clear a flag, and one
of them was forgotten.

With it derived there is nothing to forget. Whatever clears the pending
record — including a route written next year by somebody who has never read
this file — the window is back on the next frame.
