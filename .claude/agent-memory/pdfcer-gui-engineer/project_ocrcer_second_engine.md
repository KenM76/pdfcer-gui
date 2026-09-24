---
name: ocrcer-second-engine
description: OCRcer (D:\dev\OCRcer) is the second OCR engine, integrated 2026-09-24 through the engine's adapter; always the LOCAL newest, never GitHub; LLM add-on is a separate future option (O240)
metadata:
  type: project
---

OCRcer is a from-scratch OCR engine at `D:\dev\OCRcer` — a separate project,
**read-only from here**. Ken, 2026-09-24: *"add ocrcer as an ocr option"* and
*"always use the latest version of ocrcer available in d:\dev\ocrcer . github
might be a few versions behind."*

**State at 2026-09-24:** integrated (O230). The engine vendors OCRcer's newest
local commit into `pdfcer_core::ocr::engine_ocrcer` (feature `ocrcer`,
default on); this shell forwards the feature and offers a Recogniser choice
when both are linked, ocrs default. The model `ocrcer.ocrw` is NOT in either
git tree — `tools/package-portable.py` copies it from
`D:\Dev\OCRcer\model\out\` with OCRcer's LICENSE and NOTICE into
`models/ocrcer/`. OCRcer's local repo has no remote.

**Why it matters:** OCRcer reports per-word confidence; `ocrs` never does. The
dialog's confidence sentence follows `EngineId::reports_confidence`.

**How to apply:**
- Never pin OCRcer from GitHub. Moving the pdfcer pin moves OCRcer with it.
- The layer marker must carry the engine that actually ran (`EngineId::key`),
  not a constant — it was hard-coded `"ocrs"` until 2026-09-24.
- Known OCRcer defect, filed in `D:\Dev\FeatureRequests\OCRcer_FeatureRequests\`:
  digit runs split around `1` (`41177` → `41 1 77`). Its fixture test asks for
  fewer words until fixed.
- The LLM rescoring add-on (`.ocrl`, engine Pass 327.2) is its own opt-in
  option, O240: present only when linked and its file is on disk (R8).
- Tables / drafting lines / accounting structure need a richer core result
  type than words; request it only once OCRcer produces structure.

Related: [[the-project-is-pdfcer-gui-since-2026-09-03]]
