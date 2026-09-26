# `ui-verify/checks/chunk_click`

`clicking_a_chunk_selects_that_chunk` — **O215 asks 1 and 6, driven: the
left button alone reaches one line of a text block, and it reaches the same
one every time.**

# The request

`OPERATOR_REQUESTS.md` **O215**, in his words:

> *"…then let us use our usual mouse selection methods to move the chunks."*

with ask 1 — *selecting one chunk is repeatable* — and ask 6 — *all on the
left button, the right-click route stays*. Ask 3 (the boxes) is
[`crate::checks::text_chunks`]; this is the gesture the boxes made aimable.

# What "repeatable" means here, and why it needs three clicks

His report is that the same gesture *"sometimes takes the whole block"*. So
the claim under test is not *a chunk can be selected* — a unit test can say
that — but **the same point selects the same chunk on a later visit**. That
is a statement about two clicks separated by a third, and it cannot be
measured with fewer:

| clicks | what it would measure |
|---|---|
| one | that something was selected |
| the same point twice | nothing: the state did not change, and the trace collapses (below) |
| A, B, A | that A is reachable, that B is a *different* chunk, and that A comes back |

`canvas-selection` is written through `diag::trace_changed` under its own
slot, so **an identical repeat writes nothing**. A check that clicked one
chunk twice would read silence and could not tell *it selected the same
chunk again* from *the second click did nothing at all* — the two verdicts
this whole row exists to separate. Alternating is what makes the channel
answer.

# The chunk index is compared, never assumed

The check never asserts *the click on the top line selects chunk 0*. Which
index the provider gives a line is the provider's business, and pinning it
here would make a legitimate change of line granularity look like a
selection defect. What is asserted is the shape the operator experiences:
two aim points give two **different** indices, and returning to the first
gives back the **first** index.

# Ask 6 — the left button, and only the left button

Every gesture below is [`Driver::click_at`], a plain left click with no
modifier. A build where the chunk is reachable only through the context
menu passes `chunk_boxes_show_what_a_text_block_is_made_of` and fails here,
which is the distinction ask 6 makes.

# What the first click must NOT do

It must select the **block**, not a chunk. A build that descended on first
contact would make dragging a whole text block unreachable while the boxes
are on — a capability regression (**R6**) traded for the one being added, so
it is asserted rather than left to be discovered on a CAD sheet.

# Fixture — pinned, and `--pdf` is ignored

`fixtures/paragraph.pdf` through [`crate::fixture::text_chunk_point`]: one
text object of six lines on baselines 16 pt apart. Chunks **0 and 2** are
aimed at, 32 pt apart, so an aim off by a few points still lands on the
intended line rather than its neighbour.

⚠ A missing fixture is a **FAIL**, not a SKIP: it is committed here, so its
absence is a broken checkout.

# ⚠ What this check can see, and where its reach ends

It reads the selection the application reports, not the pixels. A build
whose chunk outline is drawn in the page colour selects correctly and shows
nothing; that has one oracle, a rendered screenshot, and it is
`text_chunks`'s subject rather than this one's.

# ⚠ HOW TO FALSIFY THIS CHECK — do this before believing a PASS

1. **Copy the file aside first**: `crates/pdfcer-gui/src/canvas/selection/mod.rs`
   to the scratch directory. **Never revert it with git** — this project
   runs parallel tracks and a chained revert discards another track's
   uncommitted work; restore from the byte copy.
2. **Plant the defect the operator reported**: in `click_at_object_rung`,
   change the `hit.chunk` term of the narrowing condition to `false`. Every
   unit test but two stays green, the boxes still draw, and step B goes red
   — the second click leaves the selection at the Object rung, which is
   *"it took the whole block"*.
3. **A second plant, for steps C and D**: in `canvas::presspick::take`,
   delete the `select_part` call and answer `false`. The first descent still
   works through the click path; moving between chunks stops, so a run reads
   `part=` frozen at its first value.
4. **A third, for the first-click rule**: make `canvas::clicking` set
   `chunk: true` unconditionally. Step A goes red on `level=Part` — the
   block became undraggable the moment the boxes were switched on.
5. **Prove the plant is in the artifact.** `cargo build --release -p
   pdfcer-gui`, then confirm the exe is newer than the source: a stale
   binary is the commonest cause of a falsification that "did not
   reproduce", and its tell is an **absent** trace line rather than a wrong
   one.
6. **Require the `[FAIL]` line**, not the exit code — a SKIP exits the way a
   PASS does.
7. **Restore from the byte copy**, rebuild, confirm the PASS returns.
