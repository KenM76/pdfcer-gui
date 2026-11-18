# `ui-verify/checks/model_light_meter`

`the_3d_viewer_reads_the_light_on_a_surface` — the 3D viewer's Measure tab
reads how much light arrives at a point picked on a surface, the reading
falls when the model's lights are switched off, and a circle round the point
is averaged over many points.

# What it drives

`fixtures/glow-room.prc` (`fixtures/glow-room.PROVENANCE.py`), opened with
*View…* under `--no-input` with a `ScriptedPointer`. Its lamps come from a
PRC file and state no photometric output, so its readings are relative, not
lux; the lux path is the engine's to test.

1. Press `model3d.measure.on`, then `model3d.measure.light`, on the options
   menu; close the menu.
2. Click spots across `model3d.image`, the centre first, waiting after each
   `model-light-asked` for its `model-light-reading` line, until one reads
   `direct` above zero. A `model-light-refused` line, or neither, fails. The
   reading must have `total` above zero and equal to
   `direct + emitted + bounced + ambient`, `radius` 0, `on=1` and `lit=1`,
   and `emitted` above zero: the glowing panel's light is counted by default.
3. Press `model3d.light.glow` and close the menu. With no click, the point is
   read again: `emitted` of zero, `total` below the first and one more note
   (the glow left out). Press it again: the next reading's `emitted` is the
   first's.
4. Click `model3d.lights.all`, which switches every listed light off, and
   click the same place on the picture. The note under the picture changes
   with the lights, so the picture's size and the point under the click do
   too. The glowing part in a folder of its own stays on, so the reading
   must have `direct` and `ambient` of zero and `total` below the first.
5. Click `model3d.lights.all` again, press `model3d.light.circle`, close the
   menu and click the same place. The reading must have `radius` above zero,
   `on` of two or more and `total` above zero.

# Falsified

The build before the meter fails at step 1: it draws no
`model3d.measure.light`. A build whose glow switch sets the opposite of what
it shows fails step 3 on the reading with the glow left out.
