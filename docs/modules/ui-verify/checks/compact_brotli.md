# `ui-verify/checks/compact_brotli`

Two checks on the compact window's *Compress with Brotli* switch
(`dialogs::compact_streams`) and its *Make compatible with* list
(`dialogs::compact_version`):

- `a_compacted_copy_is_packed_with_brotli` — **the switch reaches the engine
  and Save writes a PDF 2.0 copy with `/BrotliDecode` streams.**
- `an_older_target_leaves_brotli_out_and_lists_what_the_copy_uses` — **with
  PDF 1.4 chosen the same request is skipped with the target named
  (`OptimizeSettings::compatibility`), the features the copy uses are listed
  (`optimize::features_used`), and Save writes a 1.4 copy with no Brotli.**

## What they drive

Both open `fixtures/ocr-layer.pdf` (1.7, Flate streams, nothing past 1.4)
with `PDFCER_DIAG_INVOKE=mode.edit,file.save_compacted` and
`PDFCER_DIAG_SAVE_PATH` naming the copy, through the scripted pointer, so
they run under `--no-input`.

## What they assert, in order

| # | check | assertion | what its failure means |
|---|---|---|---|
| 1 | both | the first `compact-recompressed` reads `brotli=false` | Brotli is on unasked, so the rest proves nothing |
| 2 | both | `compact.brotli` is drawn; pressing it is followed by a recompression reading `brotli=true ok=true` | the switch is missing or does not reach `Choices::brotli` |
| 3 | packed | that line reads `compat=keep above=0` | something was skipped for a target nobody chose |
| 4 | packed | the saved copy contains `/BrotliDecode` and starts `%PDF-2.0` | `StreamAction::Brotli` was not what the engine ran, or Save wrote another copy |
| 5 | older | choosing `compact.version.1` (1.4) recompresses again, reading `brotli=true compat=1.4 above>0` | the target was not set as `OptimizeSettings::compatibility`, so nothing was skipped for it |
| 6 | older | a `compact-lowered` line, then `compact-features to=1.4` with a `list=` | the features the copy uses were not computed |
| 7 | older | the `compact.version.features` fold is drawn exactly when `n>0` | the list is computed and never shown, or shown empty |
| 8 | older | the saved copy has no `/BrotliDecode` and starts `%PDF-1.4` | the target let Brotli through, or Save wrote the unlowered copy |

Step 5's `above>0` is what tells the skip apart from Brotli simply not being
smaller: only `SkipReason::AboveCompatibility` counts there.
