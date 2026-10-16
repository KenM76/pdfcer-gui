# `ui-verify/checks/typedonestep`

`a_typed_move_and_turn_are_one_undo`, on `fixtures/overlapping-boxes.pdf`
(see `restack.md` for the page). The red box is clicked in Edit with
Properties open.

PASS needs, in order:

1. Control: Properties reads red as `x=60.00 y=60.00 w=160.00 h=120.00`.
2. X typed as 100 and Rotate by as 90, Apply pressed: a `one-undo-step` line
   with `pushed=2 folded=true` — the move and the turn both reached the
   engine and were folded.
3. One Ctrl+Z: the first `geometry-draft` after the `undo` line reads the
   control box again. Undoing only the turn leaves x at 100.

## Falsified

- Folding one command instead of `pushed` (`coalesce_last(1, kind)`): FAIL at
  step 3, red read at x=100 after one Ctrl+Z.
- Apply never wrapping its steps (`steps.len() > 99`): FAIL at step 2, no
  `one-undo-step` line.

## What it does not prove

- A typed resize in the same Apply. That step goes through
  `resizing::action`; the resize checks that drive it need OS input and SKIP
  under `--no-input`.
- Several selected objects: that is `typedseveral.md`.
- The full-stack branch (`MAX_UNDO_DEPTH`), which counts by edit epoch.
