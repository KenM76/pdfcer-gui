# `panels::bookmarks::tree` — the two questions this panel asks of an
outline, in the one place they can be tested


## ★★ Generic over the tree, because `OutlineItem` is `#[non_exhaustive]`

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

## ★ What [`descendants`] counts, and what it deliberately does not

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
