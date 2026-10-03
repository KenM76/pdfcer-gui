# `ui-verify/checks/snapshot_copy`

`a_snapshot_copy_is_cropped_at_the_set_dpi` — with a snapshot box laid over
blank paper beside a drawing, both routes to Copy put the box on the clipboard:
vectors with none of the drawing in them, and a picture the box's size at the
resolution the app reports (O272). It shares the launch, the rig and the
box-line reader of [`snapshot_box`](snapshot_box.md) and runs under
`--no-input`: the Copy chord is the scripted pointer's `copy` event, the
platform command egui receives for Ctrl+C, and the clicks are scripted too.

The clipboard is snapshotted before the run and restored after it with
`ClipGuard`, unless another program wrote it in between.

## Steps

1. Arm View ▸ Snapshot and lay a box from (0.70, 0.10) to (0.90, 0.30) of the
   page on `blank-overhang.pdf`, whose drawing lies inside x 6–50 %,
   y 50–92.5 %; read the box's size in points from its `snapshot-box` line.
2. Clear the clipboard, send Copy, and judge.
3. Clear the clipboard, right-click inside the box, require the
   `canvas.snapshot` menu, click its `edit.copy` row, and judge.

Judging requires, for that route's `clipboard-snapshot-copy` line:

- no `clipboard-snapshot-copy-refused` line;
- `vectors=cut` and `formats=image/svg+xml,CF_ENHMETAFILE,PNG,CF_DIBV5`;
- the clipboard's PNG header is `ceil(side × dpi / 72)` pixels on each side,
  within one pixel (the trace rounds corners to 0.1 pt), at the `dpi=` the line
  reports, and its `w=` agrees;
- the clipboard's SVG root is the box's width in points, within 0.5 pt;
- the SVG paints nothing: `painting_elements` counts path, use, image, text and
  the shape elements outside `defs`, `clipPath`, `mask`, `pattern`, `symbol`
  and `marker`, and the count is 0.

A whole-page copy fails the PNG and SVG sizes; a selection copy, or a Copy that
fell through to another rung, writes no `clipboard-snapshot-copy` line; vectors
made from the page with only its crop box narrowed carry the drawing beside the
box and fail the painting count.

## Falsification

Making the vectors from `cropped`'s page instead of the engine's cut fails the
painting count.
