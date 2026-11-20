# `ui-verify/checks/model_select_measure`

`the_3d_viewer_measures_faces_edges_and_corners` — the 3D viewer's *Face,
edge, corner* measure gives a picked face's area, perimeter and shape, an
edge's length and shape, a corner's position, and the smallest and largest
distances between two picks.

# What it drives

`fixtures/glow-room.prc` (`fixtures/glow-room.PROVENANCE.py`): a 4 by 4 floor
at height 0, a unit box, a bulb and two tubes, every face flat. The file
separates no faces, so the engine grows a face pick into the flat patch
round the click and says so (`source=grown`). The viewer's frame is z-up: the file's y is the viewer's height
`z`, and the file's z the viewer's `-y`. Opened with *View…* under `--no-input` with a `ScriptedPointer`.

1. Drag across `model3d.image` to bring side faces into view. Press
   `model3d.measure.on`, then `model3d.measure.select`; close the menu.
2. For each spot of a 7 by 7 grid over `model3d.image`, press
   `model3d.measure.again` and click the spot. A spot that lands traces a
   `model-select-figures` (or `model-select-refused`) line with the point it
   landed at (`at`).
3. A spot at height `z` 0 is on the floor: its line must read `shape=plane`,
   `area` 16 and `perimeter` 16, within 0.0001. A spot on the box must read
   `source=grown` and `area` 1: one unit side, not the box's 6.
4. Press *Start again*, click a spot on the box and one on `tube-b` (x -1.0
   to -0.8; the grid is too coarse to land on the thinner pair). Both must be
   whole parts. The `model-select-distance` line must read `minimum` 0.3 and
   `maximum` 2.692582, the farthest corners, `sqrt(1.5² + 2² + 1²)`.
5. Press `model3d.select.entity.1` (Edge), *Start again*, click the floor
   spot nearest the floor's outline: `entity=edge shape=line length=4`.
6. Press `model3d.select.entity.2` (Corner), *Start again*, click the floor
   spot: `corner` is (±2, ±2, 0).

Every oracle is the fixture's geometry, not a figure the engine produced.

# Falsified

The build before the measure fails at step 1: it draws no
`model3d.measure.select`.
