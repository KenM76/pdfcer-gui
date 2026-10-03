# `ui-verify/checks/snapshot_grips`

`a_snapshot_box_moves_resizes_and_clears` — a drag from inside a laid snapshot
box moves it whole, a drag from its bottom-right grip moves only that corner,
and putting the tool down clears the box (O272). It shares the launch, the rig
and the box-line reader of [`snapshot_box`](snapshot_box.md), and runs under
`--no-input`.

## Steps

1. Arm View > Snapshot and lay the base check's box, (0.42, 0.44)-(0.58, 0.56)
   of the page box.
2. Drag from the box's centre by (+0.10, -0.08) of the page. The last
   `snapshot-box` line must say `part=move` with all four edges shifted by that
   amount, within 2.5 pt.
3. Drag the bottom-right corner — `(urx, lly)` on the upright fixture — by the
   same amount. The line must say `part=resize` with only `urx` and `lly`
   changed.
4. Press the Snapshot button again. The trace must carry
   `snapshot-cleared reason=tool`, and `canvas.snapshot` must be retired.

## Falsification

Making `part_at` answer `None` fails step 2: the drag draws a new box
(`part=new`). Removing the hook in `app::frame` that calls
`canvas::snapshot::settle` fails step 4.
