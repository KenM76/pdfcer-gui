# `ui-verify/checks/compact_profile_fonts`

`a_compact_settings_file_cuts_fonts` — **a settings file saying
`resubset-fonts = true` cuts the fonts with the window's own switch off, and
Save settings… writes both font keys back.**

## What it drives

Opens `fixtures/resubset-pair.pdf` (one TrueType face embedded twice) with
`PDFCER_DIAG_INVOKE=mode.edit,file.save_compacted`, the pickers answered by
`PDFCER_DIAG_COMPACT_PROFILE_LOAD` / `_SAVE`. The loaded file is
`extends = "lossless"`, `resubset-fonts = true`, `merge-font-subsets = false`.
Through the scripted pointer, so it runs under `--no-input`.

## What it asserts, in order

| # | assertion | what its failure means |
|---|---|---|
| 1 | the first `compact-recompressed` reads `fonts_cut=0` | the window cuts fonts unasked, so step 3 proves nothing |
| 2 | the `compact-profile loaded` line reads `resubset=true merge=false` | the file's font keys were not parsed into the settings |
| 3 | the rebuilt copy reads `profile=true fonts_cut>0` | the request was left without a cutter (`OptimizeSettings::install_resubsetter` not called), so the file asked and nothing was cut |
| 4 | the file Save settings… writes contains `resubset-fonts = true` and `merge-font-subsets = false` | the saved file is not what the window ran |

`merge-font-subsets = false` is the engine's non-default, so step 4 cannot be
met by a fresh request written in place of the loaded one.
