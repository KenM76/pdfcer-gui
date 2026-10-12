# `ui-verify/checks/typedturn`

`a_typed_turn_rotates_an_object_about_its_centre`, on
`fixtures/overlapping-boxes.pdf` (see `restack.md` for the page). The red box,
60..220 by 60..180, is clicked at (80,80) in Edit with Properties open.

PASS needs, in order:

1. Control: Properties reads red as `x=60.00 y=60.00 w=160.00 h=120.00`.
2. The `properties.geometry.turn` region drew after the click. Without this
   step a build lacking the field would make the typing a harness error, which
   the runner files as SKIP.
3. 90 typed into Rotate by and Apply pressed: a `transform-objects` line.
4. The first `geometry-draft` after that line reads
   `x=80.00 y=40.00 w=120.00 h=160.00` and `turn=0.00`. That is the same
   centre (140,120) with the extents swapped, so a turn about a corner fails
   on x and y. The field back at 0 shows the draft reseeded.

## Falsified

- On the published build, which has no Rotate by field: FAIL at step 2,
  "Properties must offer Rotate by for a page object; it read `no
  properties.geometry.turn region`", not SKIP.
- With the pivot planted at the box's lower-left corner (`draft.x, draft.y`):
  FAIL at step 4, reading `x=-60.00 y=60.00 w=120.00 h=160.00`.

## What it does not prove

- The direction. A quarter turn of a rectangle about its centre gives the same
  box either way; `a_turn_pivots_on_the_centre_anticlockwise` in
  `geometry.rs`'s tests holds the sign.
- A part of a placed drawing. It goes through `TransformLeavesInForm`, which
  `a_part_of_a_placed_drawing_can_be_resized_and_rotated` drives from the grip.
- Rendered pixels.
