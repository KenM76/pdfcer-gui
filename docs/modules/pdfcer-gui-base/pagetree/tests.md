# `pagetree::tests` — and the one shape that would make all of them vacuous

**A test on a FLAT page tree defeats this entire module.**

On a flat tree the immediate parent *is* the root, so `/Count` and the leaf
tally can only disagree at one node — and an implementation that compared
the root's `/Count` against `pages().len()` and looked no further would pass
every such test while being blind to the defect this module exists for. The
defect is *an ancestor above the parent going stale*, and that state cannot
be constructed on a one-level tree at all.

⇒ So the positive controls here are **nested**, and the load-bearing one is
not hand-built: [`the_real_corrupt_file_is_caught`] runs against a document
produced by **the engine's own CLI performing the operator's own operation**
— `pdfcer.exe delete-pages --pages 2` on `fixtures/nested-page-tree.pdf` —
so what is under test is the real defect rather than a hand-made imitation
of it. A hand-built graph can only assert that the walk does what its author
thought; a file the writer produced asserts that the walk catches what the
writer does.

## What each control is for

| test | proves |
|---|---|
| [`a_healthy_nested_tree_is_consistent`] | the guard does not fire on good files — without it the guard could be `false` |
| [`a_stale_root_above_a_correct_parent_is_caught`] | the walk goes **above the immediate parent**; this is the assertion a flat fixture cannot make |
| [`a_stale_middle_node_is_caught_as_well_as_the_root`] | it reports **every** bad node, not just the root |
| [`the_real_corrupt_file_is_caught`] | end to end, against bytes the engine wrote |
| [`the_real_clean_file_is_consistent`] | the same file **before** the delete, so the test above cannot pass by always refusing |
| [`a_flat_tree_is_still_checked`] | the flat case is not skipped — it is merely unable to exhibit the defect |
| [`an_absent_count_is_not_a_disagreement`] | §6 — malformed is counted, not refused |
| [`a_document_that_cannot_be_walked_is_not_refused`] | §6 — a skip narrows evidence rather than fabricating it |
| [`a_kids_cycle_terminates`] | the walk cannot loop on legal-but-hostile syntax |
| [`a_node_whose_whole_subtree_was_removed_is_caught`] | the case `PageSlot::ancestors` structurally cannot see (§5) |

## Item notes

### `struct Objects`

Hand-built rather than loaded from a file for the three tests whose subject
is one *shape* (an absent `/Count`, a cycle, an emptied subtree). Those
shapes are awkward or impossible to obtain from a real writer, and building
them by hand is the only way to assert the branch at all. Every test whose
subject is the **defect** uses the real file instead — see the header.

### `fn a_healthy_nested_tree_is_consistent`

The control that makes every other test here mean something: without it,
`is_consistent` returning `false` unconditionally would satisfy all the
positive controls, and the guard would refuse every save in the program.

### `fn a_stale_root_above_a_correct_parent_is_caught`

The single assertion this module exists for, and the one a flat fixture is
structurally incapable of making. `A1` declares 1 and holds 1 — correct, as
the engine leaves it. `A` and the root still declare the pre-delete numbers.
An implementation that checked only the node holding the changed page would
pass a clean bill on this graph.

### `fn a_stale_middle_node_is_caught_as_well_as_the_root`

The middle node `A` and the root are both stale here — the shape the engine
actually produces on a three-level tree — and the walk must name both.
Reporting only the root would still refuse the save, so this is not about
the verdict; it is about the trace being able to say *how far up the rot
goes*, which is what distinguishes "the walk stopped one short" from "there
is no walk".

### `fn a_flat_tree_is_still_checked`

The guard has no special case for a one-level document and must not grow
one: a flat tree simply cannot exhibit the defect, which is a property of
the document rather than a reason to look away. If a writer ever does leave
a flat root stale, this catches it.

### `fn an_absent_count_is_not_a_disagreement`

§7.7.3.2 requires the key, so the node is malformed; but no pdfcer verb
produces one, it arrives on files pdfcer merely opened, and refusing it
would block a save on damage pdfcer did not do. `redact::proof`'s header
argues the same case at length for the same reason: a false refusal after
the operator has done the work is worse than the thing it guards against.

### `fn a_document_that_cannot_be_walked_is_not_refused`

The two halves matter separately. Not refusing is the posture; `walked ==
false` is what stops a skipped audit and a clean audit being the same
value, which is this project's most-repeated failure shape.

### `fn a_kids_cycle_terminates`

`1 0 obj << /Kids [1 0 R] >>` is legal syntax. Without the visited set this
recurses until the stack ends, and `pdfcer-core`'s panic-free policy on
untrusted input forbids that outcome as firmly as it forbids an `unwrap`.
The count is asserted as well as the termination, because a walk that
silently truncated would report a leaf tally that is a floor rather than a
total and would then refuse a save over its own truncation.

### `fn a_node_whose_whole_subtree_was_removed_is_caught`

`A1` has an empty `/Kids` and still declares 2. It appears in no
`PageSlot`, because a `PageSlot` exists only per surviving leaf, so an
implementation built on `page_slots` would report this document clean. It is
exactly the state a page deletion that emptied a subtree produces.

### `fn the_real_clean_file_is_consistent`

The negative control for [`the_real_corrupt_file_is_caught`], and it is not
optional: a guard that refused every document would pass that test. It also
asserts the fixture's shape, so a regenerated or flattened fixture fails
here rather than silently making the positive control vacuous.

### `fn the_real_corrupt_file_is_caught`

The bytes under test are produced here, at test time, by
`pdfcer_core::EditSession::delete_pages` — **the same code path the CLI ran
when the defect was measured**, and the same one the GUI's delete-pages arm
reaches. So this test asserts that the guard catches what the writer
actually does, not what this module's author believed it does.

It is written to **skip loudly rather than fail** if the engine is ever
fixed. The day `pdfcer-core` walks the ancestor chain, this document comes
out consistent and the assertion below would go red on a *repaired* engine —
turning the fix into a broken build. The skip prints the fact instead, and
[`a_stale_root_above_a_correct_parent_is_caught`] keeps the guard itself
under assertion with a graph that does not depend on any engine behaviour.
That is the same split `check-stale-blockers` exists to enforce: a claim
about what the engine cannot do is a dated citation, and where it can be an
assertion it should be one — but not an assertion that inverts on good news.

### `fn depth_distinguishes_a_flat_tree_from_a_nested_one`

`Audit::depth` is a diagnostic and it is asserted because a *consumer*
depends on it: `app::save::tests` refuses to run its engine-half assertion
on anything shallower than three levels, and the driven check reads
`levels=` off the trace and does the same. Both were added after a
falsification in which a flat fixture made a test print *"the engine has
been fixed"* about a build carrying the defect in full. A `depth` that was
silently always 0 would switch both guards off.
