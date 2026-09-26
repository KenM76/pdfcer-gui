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

## Item notes

### `const RUN`

Three characters rather than his thirty-six, because the property under test
is *"more than one show operator"* and three is the smallest number that also
exercises a middle operator — an edit that spans only the first and last
would pass on a matcher that never looked between them.

### `const STRADDLED`

The changed region therefore covers all three show operators, so the
engine cannot narrow the rewrite to one of them. See
[`a_change_that_straddles_operators_keeps_the_spanning_form`].

### `fn page_runs`

Separate from [`page_text`] and not a convenience: with two identical
strings on one page, *which* one changed is expressible only as a position
in this list. A concatenated string can say the fix is present; it cannot
say the clicked run is the one that has it, and that is the whole question
`span_from_pin` exists to answer.

### `fn the_fixtures_runs_are_written_one_glyph_per_operator`

If it were one operator the exact-pin path would be taken, the `find` would
be dropped, and every test here would be measuring the branch that already
worked — passing, and about nothing. This is the same hazard `facewall`'s
first control covers, and it is worth the four lines for the same reason.

### `fn a_typo_in_a_run_written_one_glyph_at_a_time_can_be_corrected`

Driven through the real [`super::plan`] rather than by hand-building an
`EditRequest`, because the claim is about **what the shell decides**. A test
that assembled the request itself would pass on a build where `plan` had gone
back to sending the pin, which is precisely the build this exists to catch.

### `fn left_edge`

It names a run, and that is the whole point. The minimum `llx` over
*every* run on the page answers a different question — *"is anything on this
page still at the far left?"* — and on a sheet with a title block something
always is, so an edit could fling the corrected line across the page while
that number did not move at all. The fixtures here hold one and two runs, so
both readings agree on them and the defect would have shipped invisibly.

Panics when the run has no `bbox`, which is the honest outcome: a run whose
box could not be derived cannot answer the question, and `f64::INFINITY`
folded in silently would make the comparison pass.

### `fn a_typo_that_appears_twice_on_the_page_edits_the_one_that_was_clicked`

# What this test asserted until today, and why it was right then

It asserted a **refusal**. Three things, each with its own way of going
missing: that the count saw both occurrences, that the pin was kept — which
made the request unmatchable *on purpose* — and that the refusal classified
as `AmbiguousOnThePage` rather than as [`EditRefusal::SplitAcrossPieces`],
both being true of this page and only the first being what stopped it.

That was the best available answer while `EditRequest` carried no way to
say *which* occurrence. The shell had exactly two options — address this
run **or** span across operators — and it chose to refuse rather than to
guess on a signed drawing.

# What changed

`Pass 272.0` gave it a third:
[`EditRequest::spanning_from`](pdfcer_core::text_edit::EditRequest::spanning_from)
starts the span search **at the pinned operator**. `find` says what, the
pin says which one, and every guard the span search already had is
unchanged. Shipped the same day this shell filed for it.

⇒ So the page that could not be edited is now edited **correctly**, and
this test proves it by the only means that discriminates: it checks that
the *first* occurrence changed and the *second* did not. A build that
scanned from operator 0 would also produce a page containing the fix and
would pass any assertion phrased as "the corrected text is present".

That trap is not hypothetical — the engine's own reply records its third
sabotage passing twice, once because the fixture lacked a third
occurrence and once because the assertion asked *"does this operator appear
somewhere"* rather than naming the line.

### `fn the_engine_would_have_edited_the_wrong_one`

Without this, a reader could believe `per-glyph-twice.pdf` refuses because
the engine refuses it, which would make the test above vacuous and the count
decoration. It does not: **unpinned, the engine applies the edit happily**,
to whichever occurrence it reaches first.

⚠ That is the build this module exists to prevent shipping, and this test is
the closest thing to it that can be safely written down: it demonstrates the
wrong behaviour on a throwaway session, so that nobody has to wonder what
would happen if the pin were dropped unconditionally.
