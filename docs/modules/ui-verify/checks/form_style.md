# `ui-verify/checks/form_style`

Two checks on `fixtures/half-scale-form.pdf`: a 400×300 page placing one form
at `0.5 0 0 0.5 40 40 cm`. The form holds a blue line `4 w` (leaf 0: page y 95,
x 50–190, 2 pt on the page) and a red filled rectangle (leaf 1: page
(50,50)–(90,80)). The form owns its `/Resources`, so opacity can bind. Each
check enters the drawing and selects one leaf through
`form_node_move::enter_leaf_at`.

## `a_part_of_a_placed_drawing_takes_a_width_in_points`

Selects the line. PASS needs:

1. `stroke-style-shown leaves=true … width_pt=2.000`;
2. 5 typed into Width → `stroke-style-calls … user_widths=10.000` and a
   re-read `width_pt=5.000`;
3. 40 typed into Line opacity → re-read `line_alpha=0.400`.

## `a_part_of_a_placed_drawing_can_be_recoloured`

Selects the rectangle. PASS needs `paint-shown leaves=true … fill=255,0,0`,
then: the fill swatch pressed, a point in the picker's colour square clicked,
Escape to close it (the close is the commit), a `set-object-paint` line, and a
later `paint-shown leaves=true` whose fill is not `255,0,0`. A selection
cleared by the Escape leaves no later `paint-shown`, so the check also covers
the popup's claim on Escape.

## Falsified

- Leaves routed to the page verbs → both FAIL (width: no re-read; colour: no
  recolour reached the engine).
- Canvas Escape ladder without the popup rung → the colour check FAILs: the
  Escape that closed the picker also cleared the selection.

## What they do not prove

- The rendered pixels; the re-read is the engine's leaf model.
- A drawing placed more than once (the reach disclosure).
- Dash on a leaf, Undo.
