# `ui-verify/checks/redact_chunk`

`redacting_a_clicked_chunk_marks_only_that_chunk` — **O217's first three
asks, driven: the redaction verb addresses the unit the operator selected,
and it is reachable from the object he selected it on.**

# The request

`OPERATOR_REQUESTS.md` **O217** asks that redaction address a *chunk* of a
text block, by the same gestures that select one, from the place the hand
already is. Three claims, and they are separable:

| ask | what would satisfy it | what this check measures |
|---|---|---|
| the unit | a mark bounded by the chunk, not the block | the marked bounds, twice |
| the gesture | the click that selects a chunk is the click that aims the mark | the Part rung is standing when the verb runs |
| the route | reachable without leaving the canvas (**O53**) | the row is in the object context menu and is pressed |

# Why the oracle is TWO marks in ONE launch

`redact-mark-selection-requested page=N quads=N` is written identically for
a chunk-sized mark and a block-sized one — the exact pair this row exists to
separate — so a count is an assertion both builds satisfy. The line carries
`bbox=llx,lly,urx,ury` for that reason, and this check reads it twice:

1. with the whole text object selected (the **Object** rung), and
2. with one line of it selected (the **Part** rung).

The assertion is a comparison between them — the second inside the first,
and less than half as tall. **No number is pinned.** A fixture whose
leading, page size or text position changed would move both bounds
together, and a check built on a literal would go red on a document change
while the capability was intact.

⇒ The first mark is therefore not decoration: it is the **control**. Without
it there is nothing to say a one-line box is small, because "small" is only
meaningful against the block it came out of.

# Why both marks go through the CONTEXT MENU

The ribbon route is `redact_selection`'s subject and is already driven
there. This row's third ask is the one the ribbon cannot discharge: the
operator's hand is on the chunk he just clicked, and a verb that exists only
on a tab he has to travel to is the defect **O53** names. Pressing
`menu.item.canvas.object.edit.redact_selection` proves the row is drawn,
enabled and wired — three things a roster test asserts about the *plan* and
none about the running program.

# Why the block mark is undone before the chunk mark

So the second gesture aims at the same document the first did. A `/Redact`
is an annotation the page now carries, and a check whose second click lands
on a page the first click changed is measuring two documents. `Ctrl+Z` is
also the cheapest possible assertion that marking is undoable at all.

# Fixture — pinned, and `--pdf` is ignored

`fixtures/paragraph.pdf` through [`crate::fixture::text_chunk_point`]: one
text object of six lines on baselines 16 pt apart. Chunk **2** is aimed at —
an interior line, so a block-sized mark cannot be mistaken for a chunk-sized
one by landing at the same edge.

⚠ A missing fixture is a **FAIL**, not a SKIP: it is committed here, so its
absence is a broken checkout.

# ⚠ What this check cannot see

It reads the bounds the shell *requested*, not the quads the engine stored.
`redact_selection` owns the other half — that a mark reaches the document's
own census and that nothing is applied — and the two are deliberately not
merged: a build that marks the right unit and stores nothing should report
both facts, not one.

# ⚠ HOW TO FALSIFY THIS CHECK — do this before believing a PASS

1. **Copy the file aside first**: `crates/pdfcer-gui/src/canvas/selection/mod.rs`
   to the scratch directory. **Never revert it with git** — this project
   runs parallel tracks and a chained revert discards another track's
   uncommitted work; restore from the byte copy.
2. **Plant the defect the row reports**: in `SelectionState::outline_rect`,
   delete the `part_bounds` arm so every entry answers the whole object's
   bounds. The chunk mark becomes block-sized, both bounds become equal, and
   step E goes red on the height ratio.
3. **A second plant, for the route**: delete the
   `Item::command("edit.redact_selection")` row from `shell::menus`'s
   `CANVAS_OBJECT`. Step C goes red — and the roster test in
   `shell::menus::tests` goes red with it, which is the pair working as
   designed.
4. **Prove the plant is in the artifact.** `cargo build --release -p
   pdfcer-gui`, then confirm the exe is newer than the source: a stale
   binary is the commonest cause of a falsification that "did not
   reproduce", and its tell is an **absent** trace line rather than a wrong
   one.
5. **Require the `[FAIL]` line**, not the exit code — a SKIP exits the way a
   PASS does.
6. **Restore from the byte copy**, rebuild, confirm the PASS returns.
