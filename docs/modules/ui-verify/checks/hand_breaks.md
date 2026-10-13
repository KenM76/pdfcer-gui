# `ui-verify/checks/hand_breaks`

`hand_breaks_are_read_from_marks`, G163, on
`fixtures/four-pages-unrotated.pdf` (612×792), page 1 below the band, where
the paper is blank.

## Drive

1. Edit mode, *Edit text* armed (`PDFCER_DIAG_INVOKE`).
2. Click (100, 400): blank paper, so the click becomes an add. Type
   `A much longer first line`, Enter, `x`, Ctrl+Enter.
3. Click (110, 403), on the first line.
4. Ctrl+A, type the same two lines again, Ctrl+Enter.
5. Click (110, 403) again, then Escape to close that draft unchanged.

## Oracle

- Step 2 traces `add-text`; step 4 traces `edit-block-text-applied`. A
  missing one is an error, not a verdict.
- Step 3: `text-edit-widened … source=inferred`, a `text-edit-note
  note=breaks-inferred`, and the `status-group:draft-note` region drawn.
  Add text writes no break marks, so the engine judged the breaks from
  layout and the operator is told. Whether it read `breaks=0` or `1` is the
  engine's heuristic and is not asserted.
- Step 5: `source=marked breaks=1 len=26` and no note. `x` fits after no line
  but the longest, so layout alone reads this break as a wrap; only the mark
  `edit_block_text` writes keeps it.

## Falsified

Each mutation run alone against the passing build:

- The model built without `with_line_marks` → FAIL at step 5:
  `source=inferred breaks=0`.
- The `BreaksInferred` raise removed from `place` → FAIL at step 3: no note.

## What it does not prove

A blank line: the engine marks it, but `line_ends` says only `Break`, not
how many (G183), so a re-open keeps one. A paragraph from a foreign
producer carries no marks and is always inferred.
