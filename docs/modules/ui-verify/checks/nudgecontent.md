# `ui-verify/checks/nudgecontent`

`arrow_keys_move_a_selected_object_on_the_page`, on
`fixtures/overlapping-boxes.pdf` (see `restack.md` for the page). The red box
is clicked at (80,80) in Edit with Properties open; its origin is read from
the `geometry-draft` line Properties traces every frame.

PASS needs, in order:

1. Control: the click selects red (`x=60.00 y=60.00`).
2. ArrowRight: a `canvas-move` and no `canvas-move-declined` after the press;
   Properties reads `x=61.00 y=60.00`.
3. ArrowUp: `x=61.00 y=61.00`. Screen-up is PDF-up; a sign error reads 59.
4. Ctrl+ArrowLeft: `x=60.75 y=61.00`, the quarter-point step.

`an_arrow_key_moves_a_part_of_a_placed_drawing`, on
`fixtures/half-scale-form.pdf`: the line inside the drawing is entered at
(120,95) as `form_style` does, ArrowRight is pressed, and PASS needs a
`move-leaves-in-form` with `n=1` after the press and neither
`canvas-move-declined` nor `canvas-decline`. The distance in the drawing's
units is the drag's conversion, asserted by `form_leaf_move`.

## Falsified

- `Need::arrow_on_page_content` left out of `wanted()` → FAIL at step 2
  (`canvas-move-declined`, no decomposition on the frame).
- The vertical step handed to the drag negated → FAIL at step 3 (y=59.00).
- The leaf check driven (`--exe`) on a build without the content nudge →
  FAIL: no `move-leaves-in-form` after the press.

## What it does not prove

- Rendered pixels; the oracle is Properties' reading of the engine's bounds.
- A part of a path or an anchor: they share the drag's commit
  (`moving::drag`), which their own drag checks drive.
- Undo: one entry per press is the drag's, not asserted here.
