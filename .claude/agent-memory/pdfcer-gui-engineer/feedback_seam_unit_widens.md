---
name: seam-unit-widens
description: A harness seam that replaces "the draft" changes meaning when the product widens what a draft is — the check then deletes what it watches
metadata:
  type: feedback
---

`PDFCER_DIAG_TYPE` replaces the whole draft. When clicks began opening a whole
paragraph (Edit ▸ Paragraphs, on by default), the seed silently replaced three
lines with one. `text_edit_pins_an_aligned_tail` then failed as "clicking away
committed nothing", although the paragraph commit had applied. The check's
subject, an untouched right-aligned line, had been deleted by the seam.

**Why:** a seam is defined against a unit (a run). The product changed the
unit, and nothing tied the seam to it.

**How to apply:** when a driven check that uses a seam fails after a feature
widened selection, caret or draft scope, read the trace for the *wider* path's
commit event before calling it a regression. Reach the old unit by the
operator's own switch (here Edit ▸ Paragraphs off), not by narrowing the
product. Related: [[feedback_later_feature_captures_gesture]], [[feedback_checks_own_gesture]].
