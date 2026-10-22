# `ui-verify/checks/compact_content`

`a_compacted_copy_tidies_drawing_instructions` — **the compact window's
drawing-instruction switch reaches the engine, says what it tidied, and is
what Save writes.**

## What it drives

Opens `fixtures/ocr-layer.pdf` (one page, two content streams carrying fifteen
operators that change nothing) with
`PDFCER_DIAG_INVOKE=mode.edit,file.save_compacted` and `PDFCER_DIAG_SAVE_PATH`
pointing into the run's output folder, through the scripted pointer, so it runs
under `--no-input`. It waits for the first `compact-recompressed` line, presses
*Tidy the drawing instructions on each page* (`compact.content`) off and back
on, waiting for a new line after each press, then presses Save
(`compact.commit`).

## What it asserts, in order

| # | assertion | what its failure means |
|---|---|---|
| 1 | the first line has `content>=1` and a `bytes` | tidying is not on by default, or the copy came out no smaller |
| 2 | switched off: `content=0` and more `bytes` than step 1 | the switch never reached the engine's plan, or tidying saves nothing |
| 3 | switched back on: step 1's `content` and `bytes` exactly | the switch is not a pure function of its state |
| 4 | `compact.content.said` is drawn | what was tidied goes unsaid |
| 5 | the written file's length equals step 3's `bytes` | Save wrote a different copy from the one measured |

The sentence's wording, and that the tidied stream really carries shorter
numbers and no redundant operators, are asserted by `dialogs::recompress`'s
unit test `drawing_instructions_are_tidied_only_when_asked`, which reads the
same plan. `recompressed_after` is shared with `compact_fonts`.
