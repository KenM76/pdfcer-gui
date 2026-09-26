# `canvas::textedit::glyphwall` — his typo, the pin that was stopping it, and
the occurrence count that makes dropping the pin safe


> *"on page 2 there is a spelling mistake — clien instead of client. if I try
> to edit the edit is not accepted."*

This module is [`super::facewall`]'s sibling and is built on the same
principle: **the claim that matters is executable, or it rots.** `facewall`
held a limitation and went red the day the engine lifted it. This one holds a
*capability* and a *guard*, and the guard is the half that would otherwise be
untestable in the only direction that matters.

## What was actually wrong, which is not what it looked like

His producer writes **one glyph per show operator**: a thirty-six character
line is thirty-six `Tj`s, stepped along one row by x-only `Td`s. `Pass 256.0`
taught `edit_text` to match a `find` across exactly that shape, and the
engine measured the correction on his own file:

```text
"clien"->"client"  operators_spanned=5  followers_repositioned=4
```

**That capability was in this shell's pin, and his typo still failed.** The
standing diagnosis was that the shell sends the *whole run* as `find` and
that `text_extract` synthesises the spaces inside it, so the string named
characters no operator ever wrote and no matcher could ever reach.

⇒ **Measured on his own file, that diagnosis is false.** One `EditSession`
per shape, page 2, the run he reported:

| request | result |
|---|---|
| whole-run `find` **+ pin** — what this shell sent | `NotFound` |
| whole-run `find`, **no pin** | **OK**, `operators_spanned=36` |
| `"clien"` **+ pin** | `NotFound` |
| `"clien"`, **no pin** | **OK**, `operators_spanned=5` |

The whole-run `find` — thirty-six characters, spaces and all — matches
perfectly once the pin is off. Thirty-six characters, thirty-six operators:
**the spaces are in the operators.** The synthesised-space case is real and
documented (a trace of one of his CAD title-block cells showed twenty-one),
but it belongs to a different producer and was not what stopped this.

**The pin was.** `Pass 256.0`'s contract carries one clause that decides
everything here — *"a pinned request never spans"* — so a request carrying
both a `find` and a `pinned_span` is confined to the single operator the pin
names, and on his line that operator holds one character. A thirty-six
character `find` cannot match inside it. The engine was answering the
question it had been asked, correctly, every time.

## Why the fix is not simply "drop the pin"

Because the pin is the **only** thing `EditRequest` carries that can choose
between two identical strings on one page. There is no occurrence index on
the request; `pinned_span` is the whole of its disambiguation. Dropping it on
a page where the text occurs twice hands the choice to the engine's
left-to-right scan, which takes the first — and the document this was
reported against is a **signed quotation**. Editing the wrong occurrence of a
word on one of those is not a defect the operator reports; it is one he finds
later, in a document he has already sent.

So the pin does not come off at all. [`EditRequest::spanning_from`] starts the
span search **at the pinned operator** rather than at the first operator on
the page, with every other guard unchanged: `find` says *what*, the pin says
*which one*, and the run reached is the one the caret is in.

⚠ Counting the occurrences and dropping the pin when the text is unique is
**not** a weaker version of this and must not be reintroduced as a fallback.
`find_anchor` tries a **single-operator** match across the whole page before
the spanning search runs at all, so a single-operator twin anywhere on the
sheet beats a spanning occurrence above it — dropping the pin can make the
clicked run *unreachable*, not merely ambiguous. [`super::Plan::occurrences`]
carries the engine's own ruling.

## The two fixtures, and why the second one is the important one

Both are authored by `tools/gen-per-glyph-fixtures.py` with **uncompressed**
content streams, so `grep` answers *"how many show operators hold this
line?"* without running the program under test.

| fixture | shape | what it holds down |
|---|---|---|
| `per-glyph-operators.pdf` | one per-glyph run `ABC`, unique on the page | the pin and the flag go out together and the edit **lands** |
| `per-glyph-twice.pdf` | the **same** per-glyph run `ABC`, twice | the **clicked** occurrence changes and the other does not |

⚠ **Without the second, the guard is untestable in the only direction that
matters.** A build that dropped the pin unconditionally would satisfy every
assertion made against the first fixture, for ever, while silently editing
the wrong occurrence. And that build is not hypothetical — it is the obvious
simplification of this code, and the one a future session will reach for on
seeing a count it thinks is redundant.

[`the_engine_would_have_edited_the_wrong_one`] is what makes that concrete:
it asserts, against the engine directly, that an unpinned request on
`per-glyph-twice.pdf` **succeeds**, on whichever occurrence it reaches first.
So the pin below is choosing between two edits the engine would both have
made — without this a reader could believe it was decoration over something
the engine disambiguated anyway.
