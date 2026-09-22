---
name: ocrcer-second-engine
description: A second OCR engine, OCRcer, is being built from scratch at D:\dev\OCRcer for this project — design to the pdfcer-core trait, never to OCRcer
metadata:
  type: project
---

Ken, 2026-09-22: *"Fyi we are building an new additional ocr engine for this
project in d:/dev/OCRcer from scratch."* It is a separate project tree —
**survey it read-only, do not edit it**.

**Why:** the shipped OCR path uses `ocrs`, whose weights are ~12 MB and are not
in the repo, and whose `reports_confidence()` returns `false` **permanently**.
OCRcer is MIT code *and* MIT model, ~2.2 MB, and reports per-word confidence.
Confidence is what makes an OCR review surface possible at all — see
`words_needing_review`.

**How to apply:** every OCR feature in this shell is written against
`pdfcer_core::ocr::OcrEngine` and `RecognizedWord`, never against OCRcer's own
types. Which engine is present is an **R8 capability** — express it by
registering (or not registering) the command, and let `reports_confidence()`
decide whether a confidence column exists at all. Under **R9** an engine that
does not report confidence draws *nothing* there, not a greyed column.

It was designed-but-not-built when surveyed (crates `ocrcer-core`,
`ocrcer-bench`, `ocrcer-build`). Re-check before assuming any of it runs.

Related: [[the-project-is-pdfcer-gui-since-2026-09-03]]
