# `ui-verify/checks/stroke_style`

Two checks on `fixtures/half-scale-line.pdf`: a 400×300 page with a blue line
drawn `4 w` under `0.5 0 0 0.5 0 0 cm` (2 pt on the page, y = 150) and a 2×2
grey inline picture at (50,40), 60×40. Edit mode, Properties tab forward. The
section sits under the fold, so each row is wheeled into view first.

## `a_line_width_is_typed_in_points`

Clicks the line. PASS needs, in order:

1. `stroke-style-shown paths=1 … width_pt=2.000`;
2. 5 typed into Width → `stroke-style-calls … user_widths=10.000`, re-read
   `width_pt=5.000`;
3. Long dash picked → re-read `dash=long-dash`;
4. 40 typed into Line opacity → re-read `line_alpha=0.400` with the width
   still 5 pt.

Falsified:
- width sent unconverted → FAIL at step 2, `user_widths=5.000`;
- dash array sent unconverted (width still converted) → FAIL at step 3,
  `dash=foreign`.

## `a_pictures_opacity_can_be_set`

Clicks inside the picture. PASS: `paths=0 pictures=1`, then 50 typed into
Picture opacity re-reads `picture_alpha=0.500`.

## What they do not prove

- The rendered pixels. The re-read is the engine's object model after the
  edit, not a screenshot.
- A selection spanning two scales. The grouping and the undo folding are unit
  tests in `app::actions::strokestyle`.
- Undo.
