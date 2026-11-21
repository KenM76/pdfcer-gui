# `ui-verify/checks/stroke_style`

Two checks on `fixtures/half-scale-line.pdf`: a 400×300 page with a blue line
drawn `4 w` under `0.5 0 0 0.5 0 0 cm` (2 pt on the page, y = 150) and a 2×2
grey inline picture at (50,40), 60×40. Edit mode, Properties tab forward. The
section sits under the fold, so each row is wheeled into view first.

## `a_line_width_is_typed_in_points`

Clicks the line. PASS needs, in order:

1. `stroke-style-shown paths=1 … width_pt=2.000`;
2. 5 typed into Width → `stroke-style-calls calls=1 units=page width_pt=5.000`
   (one engine call in points), re-read `width_pt=5.000` through
   `PathObject::line_width_pt`;
3. Long dash picked → re-read `dash=long-dash`;
4. 40 typed into Line opacity → re-read `line_alpha=0.400` with the width
   still 5 pt.

Falsified:
- style sent as `StrokeUnits::UserSpace` → FAIL at step 2, `units=user`;
  with the trace forced to say `page`, FAIL at the re-read, `width_pt=2.500`.

## `a_pictures_opacity_can_be_set`

Clicks inside the picture. PASS: `paths=0 pictures=1`, then 50 typed into
Picture opacity re-reads `picture_alpha=0.500`.

## What they do not prove

- The rendered pixels. The re-read is the engine's object model after the
  edit, not a screenshot.
- A selection spanning two scales. The division by each path's scale is the
  engine's, under `StrokeUnits::Page`.
- Undo.
