# `ui-verify/checks/redact_plural`

`marking_two_chunks_makes_one_mark_and_two_regions` — **O217's third ask,
driven: a redaction gesture over a set of lines is ONE mark holding one
region per line.**

# The request

`OPERATOR_REQUESTS.md` **O217** asks for redaction that addresses a chunk of
a text block. Its neighbour `redact_chunk` drives the unit, the gesture and
the route against a selection of exactly one line.
This row drives the ask that only appears once more than one line is held:
**what a single gesture over a set is supposed to produce.**

# The oracle is TWO fields that pull in OPPOSITE directions

| field | required | the wrong build it kills |
|---|---|---|
| `redact-panel marks=` | before → **before + 1** | one annotation per line: two rows in the review list and two presses of undo for one gesture |
| `redact-mark-selection-requested quads=` | **2** | the two regions unioned into one, which destroys the unselected line between them |

⇒ **Neither field alone is an oracle, and that is the whole design of this
row.** A census of +1 is satisfied by a build that unioned; `quads=2` is
satisfied by a build that wrote two separate annotations of one quad each.
Only together do they pin what the operator actually asked for: *one
gesture, one mark, one region per line, one undo.*

Each is also checked against a **control measured in the same launch** — a
singular mark taken first, on one line, through the same menu row. Without
it, `quads=2` could be a build that always writes two, and `marks` +1 could
be a build whose panel census does not move at all.

# Why chunks 0 and 2, never 0 and 1

Adjacent chunks make a union and a pair of regions produce nearly the same
`bbox`, so the geometry cannot tell them apart and the check would be
deciding on `quads` alone. Two apart leaves chunk **1 unselected between
them** — the exact shape of O217's warning — and puts the two aims 32 pt
apart on a document whose baselines are 16 pt apart, so an aim off by a few
points still lands on the line intended. `chunk_multi_move` picks the same
pair for the same reason.

# ⚠ What this check CANNOT see

The trace publishes the **union** of the regions, not each one. So a PASS
here says two regions were built and what they span together; it does not
say that each is the box of the line it belongs to, and therefore does not
by itself prove chunk 1 survives. The end-to-end proof is the apply report,
which lists the text a removal will destroy region by region —
`the_apply_report_lists_the_text_it_will_destroy`, on a copy of the fixture.

The height comparison below narrows the gap without closing it: two regions
spanning three lines of a six-line block have a union at least twice the
singular's and well under the block's, which a single unioned rectangle over
the whole block would fail. It bounds the answer; it does not name it.

# Fixture — pinned, and `--pdf` is ignored

`fixtures/paragraph.pdf` through [`crate::fixture::text_chunk_point`]: one
text object of six lines on baselines 16 pt apart.

⚠ A missing fixture is a **FAIL**, not a SKIP: it is committed here, so its
absence is a broken checkout.

# ⚠ HOW TO FALSIFY THIS CHECK — do this before believing a PASS

The two assertions need two different plants, and running only one leaves
half the oracle unfalsified.

1. **Copy the file aside first.** **Never revert it with git** — this
   project runs parallel tracks and a chained revert discards another
   track's uncommitted work; restore from the byte copy.
2. **Plant the union**: in `app::actions::redactsel::mark_selection`, fold
   the outlines into a single enclosing rectangle before building quads.
   `quads` reads 1, `marks` still reads +1, and step G goes red — which is
   the field the count alone cannot see.
3. **Plant the split**: make the same function issue one `add_redaction`
   per outline instead of one carrying every quad. `quads` still reads 2 —
   the regions were right and the GROUPING was not — and `marks` jumps by
   2, so step G goes red on the census while the `quads` assertion stays
   satisfied. That is the half the count alone cannot see, and a plant that
   leaves the other assertion green is the proof it is a second oracle
   rather than a restatement of the first.
4. **Prove the plant is in the artifact.** `cargo build --release -p
   pdfcer-gui`, then confirm the exe is newer than the source: a stale
   binary is the commonest cause of a falsification that "did not
   reproduce", and its tell is an **absent** trace line rather than a wrong
   one.
5. **Require the `[FAIL]` line**, not the exit code — a SKIP exits the way a
   PASS does.
6. **Restore from the byte copy**, rebuild, confirm the PASS returns.
