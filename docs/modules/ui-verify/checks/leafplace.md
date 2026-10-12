# `ui-verify/checks/leafplace`

`typed_x_moves_a_part_of_a_placed_drawing`, on `fixtures/half-scale-form.pdf`.
The rectangle inside the half-scale drawing is entered at (70,65) as
`form_style` does; Properties traces its X, Y, W and H as `geometry-draft`.

PASS needs, in order:

1. A `geometry-draft` line once the part is selected: the fields drew for a
   part, which they did not before `geometry::single_target` took leaves.
2. X typed as the drawn X + 10 into `properties.geometry.x` and Apply pressed:
   a `move-leaves-in-form` with `n=1`.
3. The first `geometry-draft` after that line reads the typed X. Anchored on
   the move's own line, so a draft that never reseeded cannot answer with the
   number that was typed.

`parts_of_a_placed_drawing_align_to_each_other`, on the same fixture. The
rectangle is entered at (70,65) and the line inside the same drawing is
shift-clicked at (120,95); Ctrl+Alt+6 raises `edit.align_right`. Align
right, because both parts start at x=50 on this sheet: Align left is
correctly `nothing-moves`.

PASS needs, in order:

1. A `canvas-selection` with `sel=2` and `first=leaf:` — two parts, no page
   object.
2. No `align-declined` after the key.
3. A `move-each-in-form-applied` with `moved` equal to `of` and above zero:
   the arrangement went through `move_objects_each_in_form`, not the page
   verb that would refuse leaf indices.

`page_objects_align_to_each_other` is the page-object twin, on
`fixtures/overlapping-boxes.pdf`, because `align::measure` now chooses the
route: red is clicked at (80,80), blue shift-clicked at (300,200), and
Ctrl+Alt+6 pressed. PASS needs `sel=2` with no `first=leaf:`, no
`align-declined`, a `move-each-applied`, and red, re-selected alone at
(300,70), reading `x=160.00 y=60.00` in Properties.

## Falsified

- Both checks on a build without these routes (the previous release's exe,
  `--allow-stale`): typed X FAILs at step 1, no `geometry-draft` for a part;
  Align FAILs at step 2, `align-declined reason=nothing-selected`.
- Typed X with the part's move planted at half the delta: the move commits
  and Properties reads X=55.00 against the typed 60.00 — FAIL at step 3.
- The page-object check with `measure` planted to call every selection
  parts: `align-declined reason=nothing-selected` — FAIL.

## What it does not prove

- Y, W and H. Y is the same arithmetic on the other axis; W and H go through
  `resizing::action`, which `a_part_of_a_placed_drawing_can_be_resized_and_rotated`
  drives from the grips.
- Rendered pixels.
- Where the aligned part lands: `moved=1 of=1` says the engine committed one
  move, not its size.
- Circular with *Rotate objects* on parts, declined with
  `reason=rotate-in-form`; not driven.
