# `ui-verify/checks/block_nav`

`arrow_keys_walk_between_blocks` — **the cursor moves to the next block of
text**, driven on a real drawing.

# What this is for


> *"there was an acrobat feature in the original pdfcer-gui that attempted to
> reassemble individual lines into paragraphs and the cursor would move to
> the next block of text using the navigation keys."*

It is **salvage**: the shell this project replaces did it, and this one had
never bound Up or Down at all — its caret is a character index into one run,
and a single run has no line above it.

# ★★ Why the assertion is a CHANGE OF RUN and not a caret movement

Because those are different facts and only one of them is the feature. A
build that moved the caret within the run it was already in would look
identical from the outside — the caret moves, the draft changes, a trace line
appears — and would have done nothing the operator asked for.

So `text-caret-step` carries **both** run indices and this check asserts they
differ. That is `DEFECTS.md` D14's rule applied to a navigation key: *a trace
line must carry the number a wrong build would get wrong.*

# ★ Why a real drawing, and why this check would pass vacuously on a fixture

The whole point is crossing from one recognised block to another, which needs
a page with **more than one line of text in more than one place**. This
project's own fixtures are one-run pages and blank sheets by construction —
and `FEATURES.md` records what that cost the last time it was forgotten:

> *"a check that drives a document this project authored tests the shape this
> project imagined, and the operator's documents are the only ones with the
> shape that broke."*

A page with only one line of text is a fact about the fixture, so this check
**skips** on it rather than passing.

# The chain

| # | link | its own test |
|---|---|---|
| 1 | a click on real text opens a draft anchored to a run | `text_edit_on_a_real_drawing` |
| 2 | Up/Down reach `typing`'s arrow arm at all | nothing — they were unbound until today |
| 3 | the character index survives the hop to a byte offset and back | `blocks` — on `"café"`, and not on a page |
| 4 | **the caret lands in a DIFFERENT run** | nothing |
| 5 | the draft it left is committed rather than discarded | nothing |

Link 5 is the quiet one: a caret that leaves a run with unsaved keystrokes in
it would silently drop them, which is this project's defining defect class.
It is not asserted here — the check does not type before navigating,
deliberately, because doing so would put an edit on the operator's document
to prove a navigation. Named rather than claimed.
