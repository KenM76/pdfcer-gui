# `ui-verify/checks/snapshot_picture`

`a_snapshot_set_to_picture_copies_no_vectors` — when **Snapshot copies as** is
set to *Picture only*, Copy over a laid snapshot box places a picture and
nothing else. It shares the rig of [`snapshot_box`](snapshot_box.md) and the
box corners and clipboard clearing of [`snapshot_copy`](snapshot_copy.md). It
runs under `--no-input`, and `ClipGuard` restores the clipboard afterwards.

## Steps

1. Write `snapshot_copy = picture` as the driven binary's preferences file,
   `userdata/` beside the exe.
2. Arm View ▸ Snapshot, lay the box over blank paper, clear the clipboard and
   send Copy.
3. Rewrite the preferences file empty, whatever the outcome, so later checks
   start from the defaults.

## Judging

The last `clipboard-snapshot-copy` line has `vectors=picture-only`. Its
`formats=` includes `PNG` and none of `image/svg+xml`, `CF_ENHMETAFILE` or
`application/pdf`.

## Falsification

A build that ignores the setting traces `vectors=cut` and places all five
formats.
