# `canvas::handledrag` — **dragging a Bézier handle**, the last of Phase 1

## What this closes, and the row that was wrong about it

`pdfcer`'s `gui` column ticked *"edit a Bézier handle"* `[x]`. Their sweep of
2026-08-19 corrected it to `⬜ nothing`: one of six rows that were true of
the **old** in-repo shell and became false, untouched, when the column's
referent moved to this build.

★ **Nothing was blocking it.** `EditSession::move_handle` has existed since
Pass 30.1, with a `Handle` enum, a planner, a `v`/`y` re-spelling path and a
disclosure contract — the whole capability, documented, waiting. What was
missing was a way to *see* a handle and a way to *grab* one, and both are
this shell's.

## ★★ Why this is a distinct verb from moving a node

Because the two change different things and the engine says so in the type.
`move_node` moves a point the curve passes **through**; `move_handle` moves
a point that governs the curve's **shape** and that the curve never touches.
A single "move a point" verb would have to infer which the operator meant
from what they grabbed — exactly the inference `pdfcer_core::vector::Handle`
exists to remove.

## ★ The gesture priority, and the rule behind it

A handle sits **inside** the selection's bounding box, so `handles::grip_at`
answers `Grip::Move` for every press on one. Left alone, that makes handles
undraggable — the identical collision that made the corner *anchors*
undraggable until the eight scale grips were confined to the Object rung.

Both are the same rule: **the most specific thing under the pointer wins**,
and specificity is depth down the selection ladder. So the press is tested
against handles first, then anchors, then the box.

## ★★ The disclosure this owes, and it is invisible by construction

`move_handle` returns a list of sentences that is **empty unless a `v`/`y`
segment had to be re-spelled as `c`**. ISO 32000-1 §8.5.2.1 Table 59 gives a
cubic three spellings and two omit a control point by making it equal to a
point the segment already carries; a handle that must hold its own value
cannot be expressed in those, so the drag rewrites the operator.

**The curve draws identically.** Nothing on the page changes. What changes
is that the original bytes are gone and dragging back does not restore them.
That is precisely rule 4's surviving half — *an inference the operator
cannot see still owes an off-canvas report* — and it is why
`VectorAction::MoveHandle.into()`'s apply arm forwards those sentences to the disclosure
channel rather than discarding them as "no error".

## What is deliberately NOT here

- **Turning a line into a curve.** `move_handle` refuses with
  `NoHandleHere` when the neighbouring segment is straight, and the engine's
  own comment says why: *"turning a line into a curve is a different
  operation and is not inferred from a drag"*. This shell agrees by not
  drawing a handle there, so the refusal is unreachable from the pointer.
- **Symmetric / smooth handle constraints.** Dragging one handle and having
  its opposite mirror is a *modelling* convention (Inkscape's node types),
  not a PDF one — the format has no notion of a smooth node, so the shell
  would have to invent and store one. It would also make one gesture two
  engine calls and two undo entries.
- **A closed subpath's first-anchor incoming handle.** The closing segment
  of an `h`-terminated subpath has no operands in the content stream, so
  there is nothing for `move_handle` to rewrite. `ObjectModelProvider::
  node_handles` does not return it, so no control is offered for it.
