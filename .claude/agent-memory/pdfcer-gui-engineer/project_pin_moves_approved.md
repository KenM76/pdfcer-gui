---
name: project-pin-moves-approved
description: Moving the pdfcer engine pin to main is pre-approved when a queued item needs it — own commit, full gate run, never mid-series
metadata:
  type: project
---

Engine pin moves to pdfcer main are approved whenever a queued item needs one (coordinator, 2026-10-02).

**Why:** the OCR add-ons item needs `pdfcer-ocr-host`, which postdates pin 716e616f; asking each time stalled the queue.

**How to apply:** do the move in its own commit with a full `run-all.sh`, never in the middle of a multi-step series (e.g. O279 P2–P7). Run `cargo update` on core/render/print first ([[feedback_update_engine_before]]).
