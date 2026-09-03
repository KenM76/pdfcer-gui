---
name: a-skip-is-not-red-so-a-check-can-stop-running-unnoticed
description: A driven check that drifts into permanent SKIP is invisible — diff the SKIP set, and when a fix is "call one extra function", grep for every site
metadata:
  type: feedback
---

A driven check that has drifted into permanent **SKIP** is indistinguishable
from one that is correctly inapplicable, and it decays silently: the summary
says `0 failed`, so nobody reads the per-check reasons. Periodically diff the
SKIP set against the last known one — **a check that used to PASS and now SKIPs
is a defect, never a neutral event.**

**Why:** 2026-09-01. `ocr_recognises_a_page_and_the_document_keeps_it` had been
reporting SKIP rather than PASS for an unknown number of runs. At the window's
default width the `file` tab's Recognise group **collapses**, so
`ribbon.item.file.ocr` is never declared and the harness reported *"no control
to click"* — which reads as the command having been removed. It was found by
accident, while writing an unrelated check that hit the same wall.

**The second half, which is the more general one:** `Session::maximize`'s own
doc comment describes that exact symptom. The lesson had been learned and
written down; **the call site simply never got it**, because `maximize()` was
added to the checks that were failing at the time and not to the ones that were
already green.

⇒ **When a fix is "call this one extra function", grep for every site that
should call it, not just the one that surfaced the bug.** A per-call-site remedy
with no enforcing gate gets incompletely applied, and the incomplete half fails
in the quiet direction.

**How to apply:** on any harness work, read the SKIP reasons even on a green
run. When a check reports *"the application declared no `ribbon.item.X`
region"*, grep the trace for `ribbon.group.*collapsed` and `ribbon.overflow`
before believing X was removed — they are two different reflow mechanisms with
two different region names. Related: [[a-check-that-cannot-fail-is-not-evidence]]
and [[a-long-green-check-can-be-aiming-at-nothing]]. Filed to `D:/dev/rag/egui/`.
