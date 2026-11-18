# `ui-verify/checks/model_surface_measure`

`the_3d_viewer_measures_between_surfaces` — the 3D viewer's Measure tab
measures between two picked surfaces: two parallel surfaces give the distance
between their planes, two square ones the angle between them, and *Always
measure a distance* gives a distance for the square pair too, with the angle
still said.

# What it drives

`fixtures/glow-room.prc` (`fixtures/glow-room.PROVENANCE.py`): a floor, a box
and four glowing blocks, every face flat. Opened with *View…* under
`--no-input` with a `ScriptedPointer`.

1. Drag across `model3d.image` from its centre 70 points right and down. The
   viewer opens looking square at the floor, so every surface in sight is
   parallel to it; the drag brings side faces into view. Press
   `model3d.measure.on`, then `model3d.measure.surfaces`, on the
   options menu; close the menu.
2. Click a 7 by 7 grid of spots across `model3d.image`. Each spot that lands
   on the model traces a `model-surface-pick` line with the point (`at`) and
   the normal there; the grid's picks pair up and measure as they go, which
   the check ignores.
3. From the picks choose a pair whose normals are parallel and whose planes
   are more than 0.05 apart, and a pair whose normals are square. None of
   either fails.
4. Press `model3d.measure.again`, click the parallel pair's spots. The
   `model-surface-result` line must read `relation=parallel forced=0`, an
   `angle` of at most 0.5, and a `distance` within 0.001 of the gap the two
   pick lines give: the second point's distance from the plane through the
   first point square to its normal. A `model-surface-refused` line, or
   neither, fails.
5. Press `model3d.measure.again`, click the square pair's spots. The result
   must read `relation=angled` and an `angle` within 0.05 of 90.
6. Press `model3d.surface.force` and close the menu. The held pair is
   measured again: the next result must read `relation=parallel forced=1`, a
   `distance`, and the angle of 90 still.

The readout under the picture keeps its rows whatever it says, so the
picture keeps its size and a spot clicked again lands on the same surface.

# Falsified

The build before the surface measure fails at step 1: it draws no
`model3d.measure.surfaces`.
