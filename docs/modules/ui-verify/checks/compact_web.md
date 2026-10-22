# `ui-verify/checks/compact_web`

`a_compacted_copy_is_laid_out_for_fast_web_view` — **the compact window's
Fast Web View switch reaches the engine, and Save writes the linearized copy
it measured.**

## What it drives

Opens `fixtures/ocr-layer.pdf` with
`PDFCER_DIAG_INVOKE=mode.edit,file.save_compacted` and `PDFCER_DIAG_SAVE_PATH`
pointing into the run's output folder, through the scripted pointer, so it runs
under `--no-input`. It waits for the first `compact-recompressed` line, scrolls
*Fast Web View* (`compact.linearize`) into view and presses it, waits for a new
line, then presses Save (`compact.commit`).

## What it asserts, in order

| # | assertion | what its failure means |
|---|---|---|
| 1 | the first line has `linearized=false` | the switch is on by default, or the trace lost the field |
| 2 | `compact.linearize` is drawn | the switch is missing from the window |
| 3 | switched on: `linearized=true` and `bytes>0` | the switch never reached `RebuildOptions::linearize`, or a larger linearized copy was discarded |
| 4 | the written file carries `/Linearized` and its length equals step 3's `bytes` | Save wrote a plain copy, or a different copy from the one measured |

That the linearized file parses back, and that the window says the copy is laid
out for Fast Web View, are asserted by `dialogs::recompress`'s unit test
`fast_web_view_is_written_only_when_asked`. The correctness of the layout
itself (hint stream, first-page section) is the engine's, tested there.
