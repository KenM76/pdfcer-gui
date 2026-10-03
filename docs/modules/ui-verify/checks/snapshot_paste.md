# `ui-verify/checks/snapshot_paste`

`a_snapshot_pastes_back_as_a_drawing` — in Review, a snapshot box laid round a
drawing and copied puts a one-page PDF on the clipboard under
`application/pdf`, and Ctrl+V places that PDF back as a stamp the box's size;
in Read the same paste is refused (O272). It shares the launch, the rig and the
box-line reader of [`snapshot_box`](snapshot_box.md) and the clipboard helpers
of [`snapshot_copy`](snapshot_copy.md), and runs under `--no-input`: the Copy
and Paste chords are the scripted pointer's `copy` and `paste` events, and the
mode switches are scripted `Ctrl+2` and `Ctrl+1`.

The clipboard is snapshotted before the run and restored after it with
`ClipGuard`, unless another program wrote it in between.

## Steps

1. Switch to Review, arm View ▸ Snapshot and lay a box from (0.04, 0.48) to
   (0.52, 0.95) of the page on `blank-overhang.pdf`, round the whole of its
   drawing (x 6–50 %, y 50–92.5 %), so nothing crosses the box's edge and the
   vectors are cut rather than withheld. Read the box's size from its
   `snapshot-box` line.
2. Clear the clipboard and send Copy. Require the `clipboard-snapshot-copy` line
   to say `vectors=cut` with `application/pdf` among its `formats=`, and the
   clipboard's `application/pdf` entry to begin `%PDF-`.
3. Hover at (0.75, 0.30) of the page and send Paste. Require the next
   `clip-pasted` line to be `kind=pdf as=stamp`, its rectangle the box's width
   and height within 0.5 pt, and a `custom-stamp-placed` line after it.
4. Switch to Read and paste again at the same point. Require no new
   `clip-pasted` line and a `command-declined id=edit.paste` line.

The clipboard also holds the picture, at the box's size in points (its PNG
states the dpi it was made at), so a paste that took the picture instead would
pass the size test; `kind=pdf` is what only the drawing route writes.

## Falsification

Making `clippaste::read` skip `application/pdf` pastes the PNG as a picture
stamp: `kind=image`, and step 3 fails.
