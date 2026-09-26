# `canvas::notepopup::controls` — everything in the window that CHANGES
# something


## ★★ Why this is the seam

It is the same one `crate::panels::comments::editor` took the same day, and
for the same reason: the rest of the pop-up **reads** — a heading, a byline,
the words, the thread, a hover tooltip, a placement calculation — and this
is the only part of it that produces an `Action`. `super::body` decides what
the window *says*; this decides what it can *do*.

⇒ That line survives the next feature. A Reply control on this surface would
land here; a caption about a group subordinate would land next door; and the
question *"which file?"* has an answer that does not depend on how many
lines are left in either.

## ★★★ TWO controls write, and they write to different things

This is the distinction the whole file is arranged around, and it is the one
an operator can most easily get wrong:

| control | what it changes | undo entries |
|---|---|---|
| *Save note*, *Remove note*, *Delete comment* | the annotation | one each |
| **Open by default** | the annotation's `/Open` **and its `/Popup`'s**, as one command | one |
| the window's ✕ (in [`super::body`]) | `super::open`'s per-document override — the screen | **none** |

The last row is the interesting one and [`open_default`]'s doc comment
carries its argument in full: reading a marked-up drawing *is* opening and
closing bubbles, so wiring that gesture to the document would fill a
reviewer's undo stack with entries that say *"looked at a comment"*.
