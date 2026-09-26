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
