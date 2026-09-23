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

Ken, 2026-09-22 (O230): add it as an OCR option **when this project judges it
ready** — the timing is ours. The bar is in O230: implements `OcrEngine`
(OCRcer chunk 7), not worse than `ocrs` on proportional faces, and a reject
stage. Surveyed 2026-09-22: runs end-to-end, none of the three met; 43% vs
Tesseract.js 85% on real scans; zero commits. He also wants tables, drafting
line work and accounting documents — the `OcrEngine` result is words only, so
that needs a richer `pdfcer-core` type, requested only once OCRcer produces
structure. Re-survey `D:\dev\OCRcer\RESUME.md` before quoting any of this.

Related: [[the-project-is-pdfcer-gui-since-2026-09-03]]
