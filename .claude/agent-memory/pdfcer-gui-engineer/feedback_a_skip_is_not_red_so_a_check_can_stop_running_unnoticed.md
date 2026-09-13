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

## ★★★ 2026-09-13 — a SKIP that is not about the check, the build or the world: the machine was busy

Third full sweep. `passed=183 failed=4 skipped=34` against the previous
`passed=188 failed=4 skipped=29` on the same roster. Same four failures by name,
and **five checks moved PASS ⇒ SKIP**. Re-driven straight afterwards against
**the same frozen binary**, idle machine: `5 passed, 0 failed, 0 skipped`.

Four of the five said *"no window appeared for pid N within 30s"*. The timeout
is wall-clock, so it is a threshold on **how busy the machine is**, and all five
are checks that open a second surface on top of a cold-started application —
the slowest thing the harness does. The sweep was sharing the machine with my
own work.

★★ **Two lessons, and the second is the uncomfortable one.**

1. The diff is still what found it. The totals read *"183 passed, no new
   failures"*, which is true, and which would have shipped five checks' worth of
   coverage quietly missing. Nothing else in the run was red.
2. **The SKIP set is not perfectly stable run to run, so a diff can produce a
   false alarm — and the answer is to RE-DRIVE the movers, not to discount
   them.** Five names is thirty seconds of work. Never reason about whether a
   skip "looks like a flake"; run it. The re-run is what turns *"probably
   load"* into a measurement, and it is also what would have caught a real
   regression hiding among four flakes.

★ **And the refusal sentence was an unevidenced excuse.** It reads *"on a
platform that cannot enumerate windows this is always the outcome"* — a cause
the code never measured, and false here: the same process enumerated windows for
216 other checks in the same hour. A refusal that offers a plausible reason it
did not check reads as an answered question, and nobody investigates an answered
question. Filed as SWEEP_REPAIRS R11: keep the best observation from the poll
loop and print THAT, and print the platform clause only behind a flag that says
enumeration has never once succeeded in this run.

★ **Do not fix it by raising the timeout.** A larger number moves the
boundary without removing it and costs every genuinely dead launch the extra
wait. The defect is not the duration; it is that the duration's expiry is
reported as a property of the platform instead of a property of the moment.
