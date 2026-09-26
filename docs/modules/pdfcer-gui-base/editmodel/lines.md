# `editmodel::lines` — the caret's arithmetic inside a MULTI-LINE draft


The operator, `OPERATOR_REQUESTS.md` **O127**:

> *"also can the enter key create new lines when we are editing or creating
> text?"*

Enter now inserts a line break in every draft that can hold one — a dragged
box and a clicked point alike. That is one keystroke, and it is the smaller
half of the change. **A draft that can hold two lines is a draft whose caret
has to be able to reach both**, and before this module Up, Down, Home and End
either did nothing or did something wrong:

| key | what it did in a box draft | why |
|---|---|---|
| Up / Down | **nothing at all** | `blocks::step` returns `false` for anything but `Anchor::Run`, and the arm had no fallback |
| Home / End | jumped to the start or end of the **whole draft** | the fallback assignment is `caret = 0` / `caret = len`, which is right for one line and wrong for two |

⇒ Both are the shape this project keeps finding: a key the operator presses,
a thing that does not happen, and nothing saying so. Adding the line break
without these four would have shipped a multi-line editor you cannot move
around in.

## Why this is not `blocks`, which already walks lines

Because they walk **different lines**, and confusing the two would move the
caret to another part of the sheet mid-word.

| module | a "line" is | the model |
|---|---|---|
| [`super::blocks`] | a line of the **page** — a title-block row drawn as five separate show operators is one line | `pdfcer-core`'s `EditableTextModel`, which needs an extraction of the whole page (measured at **336 ms** on the benchmark CAD sheet) |
| this module | a line of the **draft** — the text between two `\n`s the operator typed | the `String` in `egui::Memory`, and nothing else |

`blocks` is for a caret anchored to text that is already on the page, where
the question *"what is above this?"* is a question about the document. This
is for text that does not exist yet, where the only lines that exist are the
ones the operator typed — so the answer is arithmetic on a string, costs
nothing, and needs no document at all.

That is also why every function here is a **pure function of `(&str,
usize)`**. The same discipline as [`super::caret`] and for the same payoff:
every rule below is proved without a window, a document or a decomposition.

## The unit is a CHARACTER, everywhere, and it is not a detail

`super::Draft::caret` is documented as a character index because a keystroke
moves the caret by one character and `é` is one keystroke and two bytes.
Every index in and out of this module is therefore a character offset into
the draft, and the split is done with `chars()` rather than `find(b'\n')`.
A byte-indexed version of any of this compiles, passes a test written in
ASCII, and puts the caret inside a multi-byte character on the first drawing
with an accent in it — which is exactly what
[`tests::a_caret_survives_an_accent_on_every_line`] exists to prevent.

## What is deliberately NOT here

**A remembered goal column.** In a real text editor, pressing Down twice
through a short line returns you to the column you started in, because the
editor remembers where the *first* press began. That is a second piece of
state on the draft, and this module answers one press at a time — the caret
lands at the same column on the next line, clamped to its length, and a
second press keeps whatever column that left. Named rather than left to be
discovered, exactly as [`super::blocks::neighbour`] names the same omission
for the page-level walk.

## Item notes

### `fn a_single_line_draft_behaves_exactly_as_it_did`

The regression guard for the whole module: the overwhelmingly common
draft is one line, and this change must not alter what any key does to
it. Home is 0, End is the length, and there is nowhere to go up or down
to.

### `fn a_shorter_line_clamps_rather_than_overshooting`

The case a naive implementation gets wrong by landing past the end of
the line, which is an index into the *next* line's text and puts the
caret somewhere the operator did not press.

### `fn a_trailing_break_makes_a_line_to_stand_on`

The state an operator is in the instant after pressing Enter, and the
one an off-by-one drops: if the final empty line did not exist, Home,
End and Up would all answer about the line *above* the caret, and
Backspace would appear to delete the wrong thing.

### `fn a_caret_survives_an_accent_on_every_line`

The one arithmetic property that cannot be seen in an ASCII test and
panics in production. `café` is four characters and five bytes; every
offset this module produces is a *character* index, so a byte-indexed
implementation would put a line boundary inside the `é` and the first
`String` splice after it would panic.

Asserted by round-tripping every position on every line rather than by
spot-checking one, because the failure is off-by-one and an off-by-one
hides wherever it is not looked at.

### `fn an_overlong_caret_lands_at_the_end`

A draft is written from the pointer handler, the keystroke handler and
the diagnostic seam within one frame, so an index one step behind its
string is a reachable state and must not be a panic.

### `fn spans`

The one decomposition every other function here is built on, so *"what is a
line"* is answered once. A line runs from just after the previous `\n` to
just before the next one; the `\n` itself belongs to neither, which is what
makes [`end_of_line`] land *before* the break rather than on the first
character of the next line.

Always at least one entry, including for an empty draft: a draft with no
characters still has one line, the empty one the caret is sitting on.
Returning an empty `Vec` there would make every caller write the same
`if lines.is_empty()` guard, which is the shape a function should absorb.

A trailing `\n` produces a **final empty line**, and that is correct
rather than tolerated: an operator who has just pressed Enter is standing on
a new, empty line and expects Home, End, Up and Backspace to behave as if
they are on it — because they are.

### `fn locate`

Clamped to the end of the text, so a caret index left over from a longer
draft answers the last line rather than panicking. That is the same choice
[`super::caret::backspace`] makes for the same reason: a draft is edited
from several places in a frame and an index can legitimately be one step
behind the string it addresses.

### `fn offset_of`

The inverse of [`locate`], and the reason the two exist as a pair: a
vertical move is *"read the column here, write the same column there"*, and
a second derivation of either half is how a caret comes to land one
character out on lines containing a wide glyph.

Clamping the column to the target line's length is what makes Up and Down
behave the way every editor does when the line above is shorter: the caret
goes to its end rather than past it.

### `fn up`

`None` rather than "stay put", so the caller can tell *"the caret moved
nowhere"* from *"there is nothing above"* — the distinction
[`super::blocks::step`] had to add a whole second trace line for, and the
one that decides whether a key event has been consumed.

### `fn start_of_line`

The LINE's start, not the draft's, which is the whole difference this
module makes to that key. On a one-line draft the two are the same answer,
so the behaviour the operator already had is unchanged by construction.

### `fn is_multi_line`

The predicate the key handler branches on, named rather than spelled
`text.contains('\n')` at four call sites. One statement of *"multi-line"*
means the four keys cannot come to disagree about when they are in it — the
same argument `canvas::tool::space_held` makes about a predicate with two
claimants, which cost this shell its space bar for two days.
