# `ui-verify/checks/snapshot_ants`

`a_snapshot_box_outline_marches` — a laid snapshot box's outline moves
between frames and the page inside it does not (O272). It shares the launch,
the rig and the box-line reader of [`snapshot_box`](snapshot_box.md), and runs
under `--no-input`: the shots are egui's own screenshots of the window, never
a desktop capture.

## Steps

1. Arm View > Snapshot and lay the base check's box on the blank fixture.
2. Take the pointer off the window, so no hover changes the frame.
3. Take three screenshots 170 ms apart. The dash pattern repeats every
   half second, so 170 ms is never a whole period and consecutive shots
   catch the dashes at different offsets.
4. Compare each pair over the box's `canvas.snapshot` region converted to
   capture pixels:
   - the **edge**, within 2 px of the border, must differ in at least 20
     pixels for some pair;
   - the **interior**, more than 16 px inside (clear of the grips), must not
     differ in any pixel for any pair.

The counts for every pair are written to the report.

## Falsification

Making `ants_offset` answer `0.0` fails the edge test: the dashes stand still.
