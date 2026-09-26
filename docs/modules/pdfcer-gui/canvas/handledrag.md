# `canvas::handledrag` — **dragging a Bézier handle**, the last of Phase 1

## What this closes, and the row that was wrong about it

`pdfcer`'s `gui` column ticked *"edit a Bézier handle"* `[x]`. Their sweep of
2026-08-19 corrected it to `⬜ nothing`: one of six rows that were true of
the **old** in-repo shell and became false, untouched, when the column's
referent moved to this build.

**Nothing was blocking it.** `EditSession::move_handle` has existed since
Pass 30.1, with a `Handle` enum, a planner, a `v`/`y` re-spelling path and a
disclosure contract — the whole capability, documented, waiting. What was
missing was a way to *see* a handle and a way to *grab* one, and both are
this shell's.

## Why this is a distinct verb from moving a node

Because the two change different things and the engine says so in the type.
`move_node` moves a point the curve passes **through**; `move_handle` moves
a point that governs the curve's **shape** and that the curve never touches.
A single "move a point" verb would have to infer which the operator meant
from what they grabbed — exactly the inference `pdfcer_core::vector::Handle`
exists to remove.

## The gesture priority, and the rule behind it

A handle sits **inside** the selection's bounding box, so `handles::grip_at`
answers `Grip::Move` for every press on one. Left alone, that makes handles
undraggable — the identical collision that made the corner *anchors*
undraggable until the eight scale grips were confined to the Object rung.

Both are the same rule: **the most specific thing under the pointer wins**,
and specificity is depth down the selection ladder. So the press is tested
against handles first, then anchors, then the box.

## The disclosure this owes, and it is invisible by construction

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

## Item notes

### `fn the_nearest_of_two_close_handles_wins`

An anchor's two handles can be within a few pixels of each other on a
shallow curve. "Whichever came first in the list" would make which one
the operator got depend on the decomposition order — a coin toss they
cannot see and cannot learn, and one that would show up as "sometimes it
drags the wrong side".

### `const GRAB_PX`

Eight — larger than the six-pixel mark it is grabbing, matching the two
points of slack `handles::GRIP_GRAB_SLACK_PX` gives a resize grip and for
the same reason: a target that requires hitting its exact pixels is a target
an operator misses, and the miss here is worse than for a grip because it
falls through to a *move of the whole object*.

### `fn visible`

Returned as a flat list rather than grouped per anchor, because both
consumers — the hit test and the painter — want to walk all of them, and the
anchor each belongs to travels on the tuple.

Empty at the Object and Part rungs: a handle is a property of a *selected
anchor*, and there is no selected anchor above the Node rung.

### `fn at`

# Why the press is in SCREEN space and the handles are in canvas space

Because the grab radius is a **screen** distance — eight pixels is eight
pixels at any zoom, which is what makes the target feel the same size on an
A1 sheet at 0.38× and on a letter page at 1×. Comparing in canvas space
would make the radius shrink with the zoom, so a handle on a drawing would
be ungrabbable at exactly the zoom an operator uses to see the whole sheet.

### `struct Frame`

The same shape and the same reason as `canvas::resizing::Frame`: everything
below is a pure function of these, so the geometry is testable without a
document, a provider or an `egui::Context`.

### `fn drag`

Returns the preview to draw while the drag is in flight: the handle's
canvas-space position and the anchor it is tethered to, so the overlay can
draw the tether moving with the pointer.

# Why the preview is the pointer position and not a ghost of the curve

Because drawing the curve the drag *would* produce means evaluating the
Bézier this shell does not own — and a preview curve that differed from what
the engine writes, by any amount, would be two rendering paths for one
shape. `BENCHMARK.md`'s standing rule about previews applies: **a preview
shows the cursor, the render shows the document.**

### `fn anchor`

Only ever asked for by a *constrained* handle drag —
[`crate::canvas::constrain::toward`] needs a point to measure the
displacement from, and for a control point that point is its anchor rather
than the press. A handle's whole meaning is its direction and distance from
the on-curve point it serves, so locking it to the *press* row would lock a
quantity nobody thinks in.

It is a separate call, made only when Shift is down, because
[`ObjectModelProvider::subpath_node_points`] allocates over every anchor of
the subpath. Folding it into the unconstrained path would put that
allocation on every frame of every handle drag for a value nothing reads —
the same cost `canvas::moving::drag` is at pains to avoid, where one
measured CAD export has 6,681 anchors.

Subpath-scoped rather than object-scoped for the same reason: the anchor is
known to be on the entered subpath, and asking the object costs every other
subpath's nodes as well.

### `fn tether`

A free function so the overlay can draw it without knowing how a handle is
found, and so this file owns the one statement of *what a handle looks
like*: a line from the on-curve point to the control point, with a mark at
the far end. That is the universal vector-editor idiom — Illustrator,
Inkscape, Figma and the old shell all draw it — and the reason it is
universal is that a control point with no tether is an unexplained dot.
