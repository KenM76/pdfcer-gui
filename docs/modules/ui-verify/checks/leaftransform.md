# `ui-verify/checks/leaftransform`

`a_part_of_a_placed_drawing_can_be_resized_and_rotated`, on
`fixtures/form-parts.pdf`. Drives with the scripted pointer, so it runs under
`--no-input`.

PASS needs, in order:

1. A click and a double-click on the placed drawing at (240, 120) selects leaf 1, the
   polyline (`form_node_move::enter_leaf_at`).
2. Dragging the selection outline's max corner by +40, +40 writes
   `transform-leaves-in-form-applied page=0 leaves=1` with `a` and `d` above
   1.05 and `|b|` below 0.001, and the re-declared outline is more than 20
   wider and taller.
3. Dragging the rotate handle a quarter turn about the outline's centre writes
   the same line with `|a|` below 0.1 and `|b|` above 0.9.

## Falsified

- The identity matrix handed to the engine: FAIL at step 2, the outline did
  not grow.
- The rotate handle routing the leaves as page objects: FAIL at step 3, no
  applied line; the engine refused "object index 1 is out of range".

## What it does not prove

- A selection mixing page objects and leaves (the leaves are dropped).
- A live preview during the drag; the engine has no preview twin, so the
  outline moves and the drawing changes only on release.
- The Properties panel's typed width and height for a leaf.
- Rendered pixels; the oracle is the outline region re-declared from the
  re-read model.
