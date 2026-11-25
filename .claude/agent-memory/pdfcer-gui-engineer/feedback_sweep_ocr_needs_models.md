---
name: sweep-ocr-needs-models
description: The release sweep's OCR checks SKIP because the drive copy has no models/ beside it; clear them by re-driving against a scratch exe carrying the packaged models
metadata:
  type: feedback
---

In a release sweep, the five OCR checks skip with "the dialog drew no `ocr-run` control" or "resolved a scope of 0 page(s)". The cause is the sweep's drive copy, which has no `models/` folder beside it. This is not a regression. Still, a skip is not a pass, so clear them. Build `D:\scratch\<rel>\ocr\` holding the release `pdfcer-gui.exe` plus `models/ocrs` (from `D:\Dev\pdfcer\crates\pdfcer-core\assets\models\ocrs`), `models/paddle` (`crates/pdfcer-gui/assets/models/paddle`) and `models/ocrcer` (`D:\Dev\OCRcer\model\out\ocrcer.ocrw` plus LICENSE and NOTICE). Then run `ui-verify.exe --no-input --exe <that>` with the five `--check`s. On 2026-11-25 (v0.5.0-dev.20261125.1) all five passed that way.

`new_document_makes_a_page` also skips on the sweep's one-page `a1-titleblock.pdf`. Re-drive it with `--pdf fixtures/four-pages.pdf`.

**Why:** a missing-asset skip looks just like a refusal regression, so leaving it skipped means OCR ships undriven. See [[feedback_skip_on_missing_asset]].
**How to apply:** any release sweep whose `--affected` set picks up the OCR checks.
