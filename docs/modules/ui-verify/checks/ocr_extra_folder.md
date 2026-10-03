# `an_extra_ocr_folder_adds_its_models_to_the_dropdown`

**A folder added in Settings reaches Recognise text's model list, and the
model chosen from it is the one that runs.**

# Sequence

1. Plant `target/ui-verify/ocr-extra/` (absolute): `uv-ocrs/` with the ocrs
   weights and a manifest naming `ui-verify-ocrs`; `uv-vl/` with a manifest
   for engine `paddle-vl` and no weights. The weights come from
   `UI_VERIFY_OCRS_MODELS`, else the engine checkout the GUI manifest's
   `file:///` git URL names; absent weights are a SKIP.
2. Seed `ocr_model = ui-verify-gone` in the sandbox preferences.
3. Launch off-screen on `fixtures/synthetic-image-only.pdf` with
   `PDFCER_DIAG_INVOKE=file.settings` and `PDFCER_DIAG_OCR_FOLDER` set to the
   planted root. Click the OCR models page, Add…, Save.
4. Open File › Recognise text. Assert: `ui-verify-ocrs` listed runnable;
   `ui-verify-vl` listed with `why=no-vl-runner`; `ocr-model-start
   chosen=none remembered=ui-verify-gone`.
5. Click the VL entry: no `ocr-model-chosen` may follow. Click the copy:
   `ocr-model-chosen name=ui-verify-ocrs`.
6. Run: `ocr-started … source=extra-folder model=ui-verify-ocrs`, then
   `ocr-recognised` with a non-zero word count.

The scratch binary has no `models/` folder, so the only runnable model is the
planted copy: a pass cannot come from the bundled ocrs.

# Falsified

- `catalog::roots` ignoring the extra folders: FAIL at step 4 (`roots=1`, the
  copy not listed).
- Unrunnable VL entry drawn selectable: FAIL at step 5
  (`ocr-model-chosen name=ui-verify-vl`).
