# `ui-verify/checks/typedseveral`

`a_typed_x_moves_several_objects_together`, on `fixtures/overlapping-boxes.pdf`
(see `restack.md` for the page). Red is clicked at (80, 80) and green
Shift-clicked at (40, 250), in Edit with Properties open.

PASS needs, in order:

1. Control: a `geometry-several` line with `n=2` and the union
   `bx=20.00 by=60.00 bw=200.00 bh=210.00`.
2. X typed as 120, Apply pressed: the union reads
   `bx=120.00 by=60.00 bw=200.00 bh=210.00`. Only green moving would read
   `bx=60.00`; only red moving keeps `bx=20.00`; both moving by different
   distances changes `bw`.
3. One Ctrl+Z: the first `geometry-several` after the `undo` line reads the
   control union again.

## Falsified

- `align::shift` moving only the first box: FAIL at step 2, `bx=20.00
  bw=300.00`.
- The route never taken (`boxes.len() > 99`): FAIL at step 1, no
  `geometry-several` line.

## What it does not prove

- Y, and a mix with annotations, fields or parts: they go through the same
  `moves_action` that Align's checks drive.
- The full-stack branch (`MAX_UNDO_DEPTH`).
