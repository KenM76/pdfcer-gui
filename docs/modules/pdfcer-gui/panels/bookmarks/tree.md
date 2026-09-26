# `panels::bookmarks::tree` — the two questions this panel asks of an
outline, in the one place they can be tested


## Generic over the tree, because `OutlineItem` is `#[non_exhaustive]`

This crate **cannot construct a `pdfcer_core::outline::OutlineItem`**. A
recursion written directly over one is therefore a recursion no unit test in
this crate can reach — and the recursion is the only part of this module
with anything to get wrong.

Both walks here are split in two: a one-line wrapper that names the real
type, and a generic worker that takes *the two things a tree is* — how to
read a node's identity, and how to read its children. The worker can be
exercised against a tree built here in a test.

That is the fourth remedy in `D:/dev/rag/rust/`'s `#[non_exhaustive]`
finding — restructure so the logic does not touch the unconstructible type —
and it is the third time in this codebase that the constraint pushed toward
the better shape rather than merely around it (`dialogs::insert_image`'s
arithmetic and `add`'s original `find_in` were the first two).

## Why depth-first, and why the order is load-bearing

A breadth-first walk would return a shallower item carrying a duplicate id
before a deeper one. `ObjId`s are unique in a well-formed document — and an
outline that made them not so is exactly the malformed case
`read_outline`'s cycle-breaking exists for, which means it is a case that
reaches this code rather than one that cannot.

## What [`descendants`] counts, and what it deliberately does not

It counts **the nodes below a node in the tree the panel drew**. It does
*not* read `/Count`, and the distinction is the whole §12.3.3 trap the
engine warned this shell about:

| | root `/Outlines` | an item |
|---|---|---|
| `/Count` counts | all visible items, including the top level | visible **descendants**, excluding itself |
| sign | cannot be negative | **positive = open, negative = closed** |

A **closed** item contributes exactly **1** to its ancestors however large
its subtree, so `/Count` on a collapsed heading is not the number of things
a delete would take. `declared_count` is carried on `OutlineItem`
*"verbatim … Do not use this to size anything"*, in core's own words, and
this module obeys that. Walking the children the reader already resolved
gives the number the operator needs, which is *how many bookmarks go if I
press this*.

It is still the **shell's** number rather than the engine's, and the two are
allowed to differ: `read_outline` gives up part-way on a cycle, on excessive
depth, or on exhausting its item budget. See
`crate::text::panels::bookmark_deleted` for why the delete therefore reports
the engine's count afterwards as well as this one beforehand.

## Item notes

### `struct Node`

`OutlineItem` is `#[non_exhaustive]`, so the real one cannot be
constructed here — which is why the walks above take accessors rather
than the type. This is the tree they are exercised against.

### `fn an_item_is_found_at_any_depth`

Depth is the point. The hazard this search replaces is an **index**, and
an index is wrong precisely for the nested case — which is the one the
engine hit in its own CLI: *"I got this wrong myself while driving the
command and nested something two levels deeper than intended, and the
output looked entirely plausible."*

### `fn a_collapsed_item_is_visible_to_the_disclosure`

`open` is the shell's read of the **sign** on `/Count` — §12.3.3 defines
no `/Open` key, so the sign is the only carrier — and it is the one
field that decides whether an operator will be able to see what they
just added.

### `fn the_subtree_count_is_every_level_and_excludes_the_node`

This is the number the delete disclosure quotes before the press, so
both of its plausible errors are worth pinning:

* counting the node itself would over-report by one and read as *"and
  the 3 bookmarks under it"* for a parent with two children;
* counting only the immediate children would under-report, and would
  under-report **most** on exactly the deep heading where the operator
  can least see what they are about to lose.

The fixture is deliberately three levels deep and lopsided so the two
wrong answers (3 and 5) differ from the right one (6) and from each
other. That is the discipline the engine's `Pass 156.0` note asks for:
*"when you assert that A and B differ, check your fixture can tell them
apart"* — its own delete test passed against every sabotage because it
only asserted the list got shorter.

### `fn a_collapsed_nodes_subtree_is_counted_in_full`

The case the §12.3.3 trap would get wrong. `/Count` on a closed item is
negative and its magnitude is not a subtree size — core's own doc says
*"Do not use this to size anything"* — and a closed item contributes
exactly **1** to its ancestors however large it is. So a disclosure
built from `/Count` would tell an operator that removing a collapsed
chapter takes one bookmark when it takes twenty.

The fixture makes the two answers different: the collapsed node holds
two levels, so a `/Count`-shaped answer (1) and the true one (3) cannot
be confused.
