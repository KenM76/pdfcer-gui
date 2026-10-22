# `ui-verify/checks/compact_fonts`

`a_compacted_copy_can_cut_fonts_down` — **the compact window's font switch
reaches the engine, says what it cut, and is what Save writes.**

## What it drives

Opens `fixtures/resubset-pair.pdf` (one synthetic TrueType face embedded twice,
as a Word document holds it) with
`PDFCER_DIAG_INVOKE=mode.edit,file.save_compacted` and `PDFCER_DIAG_SAVE_PATH`
pointing into the run's output folder. It drives through the scripted pointer,
so it runs under `--no-input`. It waits for the first `compact-recompressed`
line, presses *Cut embedded fonts down to the characters used*
(`compact.fonts`), waits for the next line, then presses Save
(`compact.commit`).

## What it asserts, in order

| # | assertion | what its failure means |
|---|---|---|
| 1 | the baseline line has `fonts_cut=0` | fonts were cut without being asked for |
| 2 | a new line follows the switch with `fonts_cut=1` and fewer `bytes` | the choice never reached the engine's plan, or saved nothing |
| 3 | `compact.fonts.said` is drawn | what was cut goes unsaid |
| 4 | the written file's length equals that line's `bytes` | Save wrote a different copy from the one measured |

`fonts_cut=1` rather than 2: the engine merges the second copy into the first
program, so one program is cut. The sentence for it ("3 of 8 characters kept,
with 1 other copy(ies) merged into it") is asserted by
`dialogs::recompress`'s unit test, which reads the same plan.
