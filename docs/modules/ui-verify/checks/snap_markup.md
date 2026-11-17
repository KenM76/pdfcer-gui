# `ui-verify/checks/snap_markup`

`measure_tool_snaps_to_a_markup`, on `fixtures/annots-align.pdf`, in Review
with Measure ▸ Linear armed. Nothing is clicked.

PASS needs, in order:

1. Hovering blank paper at (350, 120) traces `measure-hover … markup=0`.
2. Hovering (41, 239), one point inside the square markup's top-left corner
   (`/Rect [40 200 100 240]`), traces `measure-hover … snap=1 markup=N` with
   `N` not zero: the candidate's `source_annotation`.

## Falsified

- A build that does not offer markups as snap targets traces no `markup=`
  key → FAIL at step 1.

## What it does not prove

- Which point of the markup was taken (corner, midpoint, centre, on-edge);
  only that the snap came from a markup.
- That a click commits the snapped point; `measure_linear` covers the commit
  path, which reads the same resolved value.
- Text boxes, lines, polygons and ink; the fixture's markup is a square.
- That a markup being reshaped does not snap to its own old outline.
- That a markup on a hidden layer offers nothing.
