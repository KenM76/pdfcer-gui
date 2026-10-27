# `ctrl_alt_click_deletes_a_markup_point`, `ctrl_alt_click_deletes_a_page_point`

**Detects:** with the Points tool armed, a Ctrl+Alt+click on a shown point
that does not delete it.

## Drive — markup

1. Open a copy of `fixtures/square-no-appearance.pdf` in Review.
2. Arm Polygon, click three corners and double-click the fourth; require
   `add-markup`.
3. Press `A` (Points tool) and click the middle of the bottom edge to select
   the polygon; require the `canvas.markup-node.1` region.
4. Ctrl+Alt+click that region's centre.

**Oracle:** after the mark, `point-delete via=markup outcome=raised` and then
`remove-annotation-vertex`.

## Drive — page content

1. Open a copy of `fixtures/polyline-nodes.pdf` in Edit.
2. Press `A` and click the first segment to show the polyline's points.
3. Ctrl+Alt+click the second anchor at (200, 320).

**Oracle:** after the mark, `point-delete via=anchor outcome=raised` and then
`delete-node-applied`.

## Falsification

Both fail on a build without `canvas::pointdelete`: the markup drive shows no
`canvas.markup-node.1` (the Points tool in Review never produced a click), and
the page drive traces no `point-delete`.
