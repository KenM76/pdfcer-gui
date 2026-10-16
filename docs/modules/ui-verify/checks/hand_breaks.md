# `ui-verify/checks/hand_breaks`

`hand_breaks_are_read_from_marks`, G163, on
`fixtures/four-pages-unrotated.pdf` (612×792), page 1 below the band, where
the paper is blank; and `inferred_breaks_are_disclosed`, on
`fixtures/paragraph.pdf`, which no pdfcer write has marked.

## Drive

1. Edit mode, *Edit text* armed (`PDFCER_DIAG_INVOKE`).
2. Click (100, 400): blank paper, so the click becomes an add. Type
   `A much longer first line`, Enter, `x`, Ctrl+Enter.
3. Click (110, 403), on the first line.
4. Ctrl+A, type the first line, Enter, `y`, Ctrl+Enter. The last word
   differs from step 2's so the paragraph writes; an identical retype is no
   edit and writes nothing.
5. Click (110, 403) again, then Escape to close that draft unchanged.

## Oracle

- Step 2 traces `add-text`; step 4 traces `edit-block-text-applied`. A
  missing one is an error, not a verdict.
- Steps 3 and 5: `source=marked breaks=1 len=26` and no note. Add text
  writes the same line-end marks the paragraph rewrite does. `x` fits after no line
  but the longest, so layout alone reads this break as a wrap; only the mark
  `edit_block_text` writes keeps it.

`inferred_breaks_are_disclosed`: a click at (120, 668), line 2 of
`paragraph.pdf`, must trace `text-edit-widened … source=inferred`, a
`text-edit-note note=breaks-inferred`, and draw the `status-group:draft-note`
region. The engine judged the breaks from layout and the operator is told.

## Falsified

Each mutation run alone against the passing build:

- The model built without `with_line_marks` → FAIL at step 5:
  `source=inferred breaks=0`.
- The `BreaksInferred` raise removed from `place` → FAIL in
  `inferred_breaks_are_disclosed`: no note.

## What it does not prove

A blank line: the engine marks it, but `line_ends` says only `Break`, not
how many (G183), so a re-open keeps one. A paragraph from a foreign
producer carries no marks and is always inferred.
