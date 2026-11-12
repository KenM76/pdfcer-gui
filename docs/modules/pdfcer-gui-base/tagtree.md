# `tagtree` — the open document's structure tree, read once per revision

**Purpose.** The Tags panel and the canvas outline both read the structure
tree; this module reads it once per `edit_epoch` and holds what they share.

**Contract.**

- `OpenDoc::tag_tree()` returns a `TagRead`: `Untagged` when the catalog has
  no `/StructTreeRoot` (`has_structure_tree`), `Failed(why)` when the engine's
  read errs, else `Read(StructureTree)`. The read is cached on `OpenDoc::tags`
  and redone when `edit_epoch` changes.
- `TagPick` is the picked element, the epoch it was read against, and its
  boxes `(page index, Rect)` in default user space. `boxes_on(page, epoch)`
  yields nothing once the epoch has moved, so a stale outline is never drawn
  over edited content.
- `no_box_reason(tree, i)` says why an element has no box: no content kids,
  a page that did not resolve, an MCID the page does not hold, or content that
  is not text (the engine's `element_bbox` measures text runs).

**Why the boxes come from `element_bbox`.** It is the engine's own answer and
it unions descendants, so a container outlines its whole content. Painted
bounds (`structure_bounds`) replace it at the pin move that brings them; the
GUI computes no bounds of its own.
