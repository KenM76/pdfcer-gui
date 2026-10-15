# `ui-verify/checks/annot_too_many`

`an_align_too_large_for_one_undo_is_refused` drives `fixtures/annots-many.pdf`
under `--no-input` with `mode.edit`. It has no SKIP arm. The fixture holds 300
Square markups in a 20-column, 15-row grid, and nothing else; its provenance
script carries the geometry.

The gesture must commit nothing because one undo step folds at most
`MAX_UNDO_DEPTH` (256) commands. A move of more than that would leave the
earliest commands outside the step, so one Undo would take back only part of
the alignment. The guard is `foldeach::within_undo_depth`, applied before any
command is pushed.

1. Require `status-group:decline` not yet declared.
2. Drag a window marquee from (5, 495) to (595, 5); require
   `annot-band selected=300`.
3. Press Ctrl+Alt+4 (Align left). Require `move-mixed-applied annots=0/285`
   (column 0 already sits at the left edge) and a `move-mixed-refused` line.
4. Require `undo-depth-declined changes=285 limit=256`. That line is the
   named decline being recorded; without it, the funnel falls back to the
   generic "That change was refused".
5. Require `status-group:decline` declared.

A build without the guard moves all 285 and traces `annots=285/285`, which
fails step 3.
