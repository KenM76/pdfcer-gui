---
name: backlog-key-matches-engine
description: check-engine-backlog keys a row by its opening words, matched against the ENGINE's FEATURES.md wording at the pin — a fully-qualified path where the engine wrote a short one is "unaccounted"
metadata:
  type: feedback
---

A new upscale row opened `pdfcer_core::ocr::upscale`; the engine's FEATURES row
says `ocr::upscale`, so the gate reported the row unaccounted although it was
there. Renaming the opening to the engine's own words fixed it.

**Why:** the gate cannot know two spellings name one module; it matches text.

**How to apply:** when adding rows for a new pin, copy the opening words from
the engine's `docs/FEATURES.md` at the pin, then add the full path later in the
row if the drift gate needs it. After moving a row between sections, re-run
`tools/walk-engine-backlog.py --write <date>` — it reports rows whose verdict
contradicts their section. See [[feedback_type_reached_field]].
