# `pdfcer-gui/app/actions/onestep`

`Action::OneUndoStep(steps)` is for one operator gesture that needs several
actions — Properties' Apply with a typed move, resize and turn. Each step runs
through the ordinary router, so each is refused or applied exactly as it would
be alone; afterwards everything they pushed is folded into one undo entry,
labelled with the first pushed command's kind.

## Item notes

### `fn fold_since`

The count of pushed commands is the undo-depth delta. At `MAX_UNDO_DEPTH`
the stack drops its oldest entry per push and the depth stops moving, so the
edit-epoch delta counts instead: every applied edit bumps it once. A refused
step pushes nothing and is not counted; what did apply stays applied and is
still folded. Fewer than two pushed needs no fold.

`coalesce_last` needs the session uniquely owned, so the render worker is
cancelled first. When the fold cannot happen every change is still applied
and only the grouping failed: the off-canvas note says how many Undo presses
it takes, and nothing is retried. Traced
`one-undo-step pushed= folded= undo_depth=`; driven by
`a_typed_move_and_turn_are_one_undo`.
