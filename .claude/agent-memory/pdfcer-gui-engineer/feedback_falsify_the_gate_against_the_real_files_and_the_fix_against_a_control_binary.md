---
name: falsify-the-gate-against-the-real-files-and-the-fix-against-a-control-binary
description: A gate's self-test proves the gate parses its own fixtures; a control binary built from the pre-fix source proves the FIX. Neither substitutes for the other, and a self-test has already printed PASS over a live defect.
metadata:
  type: feedback
---

**Two falsifications, not one, and they measure different things.**

1. **The gate, against the REAL files.** Plant the actual pre-fix line back into
   the actual source file, delete one real exemption marker, run the gate,
   assert it names exactly those and nothing else, then restore **from a `cp`
   copy**. A gate's `--self-test` only proves it parses the fixtures its author
   wrote.
2. **The fix, against a CONTROL BINARY.** Rebuild the pre-fix source, run the
   control under the condition the defect needs, and confirm it goes red. Then
   run the fixed one under the same condition and confirm it does not. Without
   the control, "it passes now" is compatible with "the defect was never
   reproducible in this harness in the first place".

**Why:** 2026-09-13, the fixed-`%TEMP%`-path class. The gate's ten-block
self-test passed on the first run, which proves nothing about the eleven real
call sites — and this project has already shipped a gate whose **mechanism 4 of
`check-trace-names` passed its own self-test while printing PASS over the live
defect it was written to find**, and another (`check-gate-input-scope`, first
draft) that reported **zero of three planted violations while printing PASS**.
A self-test is written by the same person, at the same moment, with the same
mental model as the bug.

The control binary is the half that is usually skipped, and on this occasion it
paid for itself immediately: the pre-fix binary run twice concurrently went red
3 rounds of 3 and named **two different victim tests** than the failure that
opened the investigation. That is not a footnote — it *is* the finding. The
casualty is whichever test lost the race, so the same defect reports a different
feature broken every time, which is exactly why the class survived so long.
Without the control I would have written "fixed" against a single anecdote and
never learned the shape.

**How to apply:**
- Any commit whose message says *fixed* about a race, a timing window or a
  shared resource owes a control-binary paragraph in that message. If the
  condition cannot be reproduced on demand, say so in the message rather than
  letting silence imply a measurement.
- Restore planted defects from a `cp` copy, never `git checkout` — see
  [[never-git-checkout-to-undo-an-experiment]].
- If the gate's clean path can print a count, make **zero a FAILURE**. A gate
  that stopped matching anything looks identical to a clean tree.
- Related: [[a-check-that-cannot-fail-is-not-evidence]] (the driven-check form of
  the same rule), [[a-driven-failure-is-a-claim-about-the-check-too]] (the
  reverse direction — a red result also needs triage).
