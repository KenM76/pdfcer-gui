# `a_program_ocr_addon_runs_and_is_disclosed`

**A Tesseract program add-on is offered, runs when chosen, and the dialog
names the program it started; refused in Settings, it is listed and cannot be
chosen.**

# Sequence

1. Plant `target/ui-verify/ocr-program/ui-verify-tess/`: the engine's
   stand-in `pdfcer-ocr-test-engine.exe` copied in as `tesseract.exe`,
   `tessdata/eng.traineddata`, and a manifest (`kind = program`,
   `engine = tesseract`) whose SHA-256 lines are computed with `certutil`.
   The stand-in is found beside the default build output
   (`ctx.profile.default_exe`) or at `UI_VERIFY_OCR_TEST_ENGINE`; absent is
   a SKIP, with the build command in the reason.
2. Seed `ocr_folder` = the planted root and `ocr_model = ui-verify-tess`.
3. Launch off-screen on `fixtures/synthetic-image-only.pdf`, open File ›
   Recognise text. Assert `ocr-model name=ui-verify-tess engine=tesseract
   runnable=yes` and `ocr-model-start chosen=ui-verify-tess`.
4. Run. Assert `ocr-started engine=tesseract`, then `ocr-applied words=5
   scored=true disclosed>=1 program=` ending in
   `ui-verify-tess/tesseract.exe` (either separator).
5. Relaunch with `ocr_program_addons = refuse` added. Assert the entry lists
   `runnable=no why=refused-by-policy` and `chosen=none`.

The stand-in speaks Tesseract's command-line protocol and returns five
words at confidence 90 (its four fixed words and one naming the `--dpi` it
was given), so step 4's count and `scored=true` can only come from
the program having run.

# Falsified

- The picker's policy forced to Allow: FAIL at step 5 (`runnable=yes`).
- The program sentence dropped from the disclosure list: FAIL at step 4.

# Not covered

A real Tesseract install, and a stock Tesseract folder outside an add-on
(`ProgramEngine::from_operator_folder`), which this shell does not offer.
