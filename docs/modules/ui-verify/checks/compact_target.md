# `ui-verify/checks/compact_target`

`a_compacted_copy_can_be_made_to_fit_a_size` — **the compact window's size
target reaches the engine's size search, the copy it keeps fits, the window
says so, a starting point can be chosen, and Save writes that copy.**

## What it drives

Opens `fixtures/noisy-photo.pdf` with
`PDFCER_DIAG_INVOKE=mode.edit,file.save_compacted` and `PDFCER_DIAG_SAVE_PATH`
in the run's output folder. Waits for the unasked baseline
`compact-recompressed` line, clicks *Fit under* (`compact.target`), waits for
the search, opens *Start from* (`compact.preset`), picks Screen
(`compact.preset.screen`), waits again, then clicks Save (`compact.commit`).

## What it asserts, in order

| # | assertion | what its failure means |
|---|---|---|
| 1 | the baseline says `target=none` | a search ran unasked, or the trace lost the field |
| 2 | the switch is declared visible | the option is missing or clipped |
| 3 | the next line says `target=met`, `bytes ≤ max` | the search did not run, failed, or kept a copy that does not fit |
| 4 | `step ≥ 1` and the baseline is above `max` | the opening target did not exercise the search |
| 5 | `compact.target.said` is drawn | the lowered quality goes unsaid |
| 6 | picking Screen gives `resized ≥ 1` with the target still met | the starting point never reached the engine |
| 7 | the written file's length equals that line's `bytes` and is within `max` | Save wrote a different copy |

## Starting points

The window opens at `dialogs::compact_target::Start`: *Save a compacted copy*
at Lossless with no target, *Reduce file size* at Standard, *Fit to a size* at
Lossless with the target on. This check opens it plain; the other two are
driven by `size_reduction_is_on_the_ribbon_and_the_tab_menu`.

## Why the fixture is noise

Noise does not compress, so the plain copy stays above half its own size and
only a lossy step of the search fits it. On a compressible picture the plain
copy could already fit and arm 4 would report the search unexercised.

## Falsification

Arm 3 is red against a build whose `recompress` passes no budget to
`run` (the baseline line comes back with `target=none`). Arm 6 is red against
a build whose `settings_for` always starts from Lossless (`resized=0` after
Screen is picked).

The starting-point list sits above the result line, so the check scrolls the
body up to reach it after scrolling down to the result.
