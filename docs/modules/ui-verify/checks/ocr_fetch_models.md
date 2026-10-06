# `ui-verify/checks/ocr_fetch_models`

`ocr_models_download_into_the_models_folder` — File ▸ Recognise ▸ Download
OCR models… fetches the engine's pinned `ocrs` files into `models/ocrs` beside
the program, and the window then shows its result line (the licence's
attribution).

# What it drives

A copy of the binary in `<out>/fetch-models/`, so the download lands in that
folder's `models/ocrs` and never in the build's own. Off the desktop under
`--no-input` with a `ScriptedPointer`, no document open:

1. File tab, the collapsed Recognise group if needed, then
   `ribbon.item.file.fetch_ocr_models`.
2. `fetch-models.download.ocrs`, then up to four minutes for
   `fetch-models-done` or `fetch-models-failed`. It must be `done` with
   `files=2`, and `fetch-models.result` must be drawn.
3. Each file in `<out>/fetch-models/models/ocrs` must equal, byte for byte,
   the copy in the build's own `models/ocrs` (the packaged pins, whose
   SHA-256 matches `pdfcer_core::ocr::models`).

# Needs

The network, and the `ocrs` models in the driven build's `models/ocrs` as
the reference. Without the reference the check is an error, not a pass.

# Falsified

The download written to the wrong folder (`target_in` joining a different
sub-folder): `done files=2`, then step 3 fails with the files not where
Recognise text looks.

The window drawn below `Dialogs::show`'s no-document guard: with nothing
open it never appears, and step 2 fails with no `fetch-models.download.ocrs`.

