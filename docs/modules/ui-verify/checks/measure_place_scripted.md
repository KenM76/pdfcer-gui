# `ui-verify/checks/measure_place_scripted`

`measure_place_without_the_mouse` — a finished radius measurement follows the
pointer and is committed only by the click that places it, with its value
text at that click (O261). The window is off the desktop and driven only
through `ScriptedPointer`, so the check runs under `--no-input`.

## Steps

1. Open `fixtures/blank-overhang.pdf`; click `ribbon.mode.review`,
   `ribbon.tab.measure`, then `ribbon.item.measure.radius_diameter`, opening
   collapsed `ribbon.group.measure.*` buttons until the item shows.
2. On the blank right half, a circle of radius 0.10 page widths centred at
   (0.70, 0.35) of the page: single clicks on the rim at 90°, 210° and 330°,
   then a double-click at 150°. The picks are quick, so egui may count the
   double as a triple; both must end the fit.
3. A `measure-finish` line must follow, with no new `add-dimension`.
4. Hover at (1.6 r, 0.9 r) from the centre, take the in-app screenshot
   `measure-place-scripted-preview.png`, then click there.
5. `measure-place` must carry `text_x`/`text_y` — the committed kind's
   `label_anchor()` — within 1.5 pt of the aimed point, and `add-dimension`
   must have increased.
6. `measure-place-scripted-placed.png` is taken after the commit, to set
   beside the preview: an R8b screenshot comparison, read by eye.

## Falsification

Making `placed_at` return its input unchanged fails step 5 with the text
reported at its unplaced anchor.

## What it does not cover

Linear, perimeter and two-line placing: the OS-driven checks
`dimension_label_drag`, `dimension_corner_count`, `measure_perimeter`,
`dimension_display_menu` and `dimension_circular_label_drag` click to place
and read the commit. Whether the preview's pixels equal the commit's is not
asserted; the two screenshots are the evidence.
