# `ui-verify/checks/model_select_readout`

`the_3d_viewer_converts_projects_and_reframes_its_measures` — the 3D viewer's
*Face, edge, corner* measure says its figures in a chosen unit, flattens a
distance and a face's area onto a chosen plane, and says corners in a
coordinate system set from two picked edges.

# What it drives

`fixtures/glow-room.prc` (`fixtures/glow-room.PROVENANCE.py`), the fixture of
`model_select_measure`: a 4 by 4 floor at height 0, a unit box and `tube-b`
(x -1.0 to -0.8), whose nearest sides face the box's 0.3 apart along X. The
engine's PRC writer states glTF's metres, so the file states its unit. The
viewer's frame is z-up. Opened with *View…* under `--no-input` with a
`ScriptedPointer`. The trace's figures are in model units and the model's
system whatever the readout shows, except `local=`.

1. Drag across `model3d.image`, press `model3d.measure.on` and
   `model3d.measure.select`, and survey the 7 by 7 grid as
   `model_select_measure` does.
2. Press `model3d.select.unit.2` (millimetres): `model-select-unit` must read
   `unit=mm factor=1000`. Press `model3d.select.second.4` (inches): it must
   read `second=in` and show one metre as `1000.000_mm_(39.370_in)` (the
   trace writes spaces `_`).
3. Press `model3d.select.projection.2` (XY) and pick a floor spot: the
   `model-select-projected` line must read `plane=xy` and a `face1`
   toward/away pair summing to 16 with one side 0. Press
   `model3d.select.projection.3` (YZ): `face1` must sum to 0.
4. Still on YZ, pick the box and `tube-b`: `minimum` 0 and `out` ±0.3. Press
   XY: `minimum` 0.3 and `out` 0.
5. Press projection 0 and `model3d.select.entity.1` (Edge). Pick the floor
   spot nearest a side running along Y, then the one nearest a side running
   along X. Press `model3d.select.frame.set`: `model-select-frame` must read
   `set=1`, an origin at (±2, ±2, 0) and a Z of (0, 0, ±1).
6. Press `model3d.select.entity.2` (Corner) and pick the first spot again:
   its `local=` must have X and Y each 0 or 4 and Z 0.
7. Press `model3d.select.frame.model`: `set=0`.

Every oracle is the fixture's geometry or the unit's definition, not a figure
the engine produced.

# Falsified

Swapping the XY and YZ normals in `Readout::normal` fails step 3: the floor
projects nothing on XY.
