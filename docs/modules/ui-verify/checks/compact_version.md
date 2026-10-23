# `ui-verify/checks/compact_version`

`a_compacted_copy_is_made_compatible_with_an_older_version` — **the compact
window's *Make compatible with* list reaches the engine, and Save writes the
lowered copy it measured, with the older header.**

## What it drives

Opens `fixtures/ocr-layer.pdf` (declared 1.7, nothing in it past 1.4) with
`PDFCER_DIAG_INVOKE=mode.edit,file.save_compacted` and `PDFCER_DIAG_SAVE_PATH`
pointing into the run's output folder, through the scripted pointer, so it runs
under `--no-input`. It waits for the first `compact-recompressed` line, scrolls
the list (`compact.version`) into view, opens it, presses *PDF 1.4*
(`compact.version.1`), waits for a `compact-lowered` line, then presses Save
(`compact.commit`).

## What it asserts, in order

| # | assertion | what its failure means |
|---|---|---|
| 1 | no `compact-lowered` before a choice | the list does not start at *Keep*: the copy is lowered unasked |
| 2 | `compact.version` is drawn, and opened offers `compact.version.1` | the list is missing, or does not offer 1.4 |
| 3 | `to=1.4 outcome=lowered bytes>0` | the choice never reached `optimize::lower_version`, or the engine refused a file it should lower |
| 4 | the written file starts `%PDF-1.4` and its length equals step 3's `bytes` | Save wrote the unlowered copy, or a different copy from the one measured |

A refusal (a target below what the content needs, naming what needs it) and
*not lower* are asserted by `dialogs::compact_version`'s unit tests on a
self-built file with `/UserUnit`.
