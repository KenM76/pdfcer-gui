# `markup_resize_preview` — dragging a comment's corner shows where it is going


> *"the Markup Items don't have a live preview — the bounding box stays the
> same size when I drag the handles — the items that I can resize do
> resize."*

## What shipped, and why nothing here could see it

`canvas::overlay` has two sibling ghost painters, split apart deliberately
(a move is one displacement applied to everything; a resize is a **map**
whose answer depends on where each corner started).

`draw_move_ghost` opens with an annotation arm and returns.
`draw_resize_ghost` did not: it iterated `SelectionState::outlines()`, which
holds **page-content** entries and is **empty** whenever what is selected is
a comment. So the loop drew nothing — and one level up, the `grip_box`
guard beside it answered `None` for exactly the same reason, so the ghost
was not even reached.

⇒ **The resize itself worked the whole time.** What was missing was the
picture of where it was going, which from the operator's chair is
indistinguishable from a resize that does not work. He read it, reasonably,
as the second.

## Why this check exists rather than a unit test alone

There *is* a unit test now — `canvas::overlay::tests::
an_annotations_ghost_box_is_its_own_rect_and_grip_box_is_left_alone` — and
it fails against the shipped code. It is not enough on its own, and the
reason is this project's founding rule in its most literal form:

**the defect is a picture that was not drawn.** A unit test can assert that
a function returns a rectangle; only a driven run can assert that a
rectangle *reached a frame* while a real pointer was held down. Every other
instrument agreed the feature worked — the commit landed, the trace was
clean, the tests were green, and the operator watched nothing happen.

## What it asserts, and the one that discriminates

1. a rectangle is authored and selected — the steps before the subject;
2. a drag from its **corner** raises a resize, so the grip was hit rather
   than the body (a body hit is a MOVE, and would pass a sloppier check);
3. **`canvas-resize-ghost` is published at all**, which is the half that was
   missing; and
4. **at least two published ghosts differ in size**, which is the half
   that cannot be satisfied by a ghost that is drawn once at the selection's
   own dimensions and never updated. That is precisely *"the bounding box
   stays the same size"*, and a check asserting only (3) would pass on it.

## Item notes

### `const SHAPE`

Deliberately the same rectangle `markup_move` uses. Two checks aiming at
one shape means a fixture that breaks one breaks both visibly, rather than
one of them quietly measuring an empty patch of paper.

### `const DRAG_TO`

Both axes, for `markup_move`'s reason applied to a different value: a
resize that scaled only x would satisfy a check that dragged only in x, and
`sy` is the factor with the sign convention to get wrong.

### `fn extent`

Width and height rather than the corners, because the assertion is about
**size** and carrying the position would invite a check that accidentally
asserts the ghost is somewhere in particular — which it is not required to
be, since the pivot is the opposite corner and moves with the grip.
