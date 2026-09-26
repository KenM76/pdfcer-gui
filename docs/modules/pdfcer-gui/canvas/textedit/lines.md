# `canvas::textedit::lines` — the caret's arithmetic inside a MULTI-LINE draft


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
