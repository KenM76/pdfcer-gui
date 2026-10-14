# `ui-verify/checks/ocr_models_install`

`ocr_models_install_from_a_zip_and_a_folder` — Manage OCR models…'s Install
from file and Install from folder, on a copy of the binary whose `models/`
starts empty (`ocr_models_common`).

The native file pickers cannot be driven by a scripted pointer, so the
binary reads the chosen path from `PDFCER_DIAG_OCR_ADDON_ZIP` and
`PDFCER_DIAG_OCR_ADDON_FOLDER` when they are set. The path is fixed at
launch; the check rewrites the zip at that path between presses.

# What it drives

The zip is built by the check from the build's own `models/ocrs` — whose
manifest carries two `sha256` lines — as `ocrs/<file>`, deflated, in the
layout `zipfile` and the engine's packager write.

1. **A tampered add-on**: the first byte of `text-rec-checkpoint.rten`
   flipped before compression, so the zip's own CRCs agree and only the
   engine's fingerprint check can catch it. Must give
   `ocr-models-install-failed why=verify`, no `models/ocrs`, and no
   `.pdfcer-installing*` folder left beside `models/`.
2. **The good add-on**: `ocr-models-installed name=ocrs checked=2`, the
   next listing is exactly `ocrs`, the result line is drawn, and both files
   are byte-identical to the source.
3. **The same add-on again**: `why=exists`.
4. **Install from folder** with a copy of the bundled `paddle` (no manifest):
   installs as `paddle`, and its three files are byte-identical.

# Falsified

`OcrModel::verify`'s error ignored in `manage::install`: red at step 1, the
tampered add-on was not refused.
