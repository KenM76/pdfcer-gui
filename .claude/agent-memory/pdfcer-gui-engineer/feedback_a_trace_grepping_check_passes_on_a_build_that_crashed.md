---
name: a-trace-grepping-check-passes-on-a-build-that-crashed
description: A check whose oracle is a trace line cannot see a crash that comes after that line — the whole ui-verify suite reported PASS on a build that aborted on open.
metadata:
  type: feedback
---

**A driven check that asserts on a trace line has no opinion about whether the
program is still alive.** If the crash comes after the line it greps for, the
evidence it wants already exists and it reports PASS.

## The incident, 2026-09-03

An outside reviewer opened `pdfcer ▸ Keyboard shortcuts` on a fresh launch and
the **process aborted**, taking their unsaved markup. `ui-verify`'s
`dialogs_open_in_their_own_window` drives that exact dialog, and had been
reporting:

> ★ Keyboard shortcuts is a real OS window: [[186.0 209.0] - [606.0 689.0]]

**PASS, on the crashing build**, run after run. Not by luck: the
`viewport-inner` line it reads is written *before* the panic. The check was not
wrong about what it asserted — it simply never asked the other question, **and
neither did any of the other 152.**

⇒ This was never one check's defect. **Every trace-reading check in the harness
could pass on a build that crashes**, provided the crash came after its line.

★ Note the reviewer's own sentence was *"the suite drives many dialogs but not
this one"* — an absence claim, and it was **wrong**. The dialog was driven. The
truth was worse than the report, which is worth remembering: when an outside
report says "you don't test X", check whether you do, because *"we test it and
it passes anyway"* is a much bigger finding.

## The fix, and where it had to go

**In the one function they all call**, never as a rule each check remembers —
that is the hand-written-list shape this project has now been caught by four
times.

1. `Session::trace` refuses to return a trace from an exited process, unless the
   check said `session.expect_exit()` first. Opting out is **greppable and a
   statement**, rather than an omission that looks identical to not having
   thought about it. Two checks legitimately expect an exit and now say so.
2. **The refusal is FATAL, not a skip.** `Error::fatal` +
   `CheckReport::from_error`, and all **152** `Err(skip) => report.skip(...)`
   arms rewritten in one mechanical pass. This project's own record:
   *a SKIP is not red, so a check can stop running unnoticed* — and a crashed
   program reported as "did not run" is barely better than one reported green.
3. **Falsified**: with the crash planted back in, the same check that reported
   PASS reports **FAIL** and quotes the panic line.

★ The plant had to go in the **outer viewport closure**, not the inner one. The
first attempt put a sentinel inside the `Frame::show` body, which is re-created
per outer invocation, so it never fired and the falsification "passed" —
a falsification that itself needed falsifying, for the second time in one day.

## How to apply

- **Ask what a check would do if the program died.** If the answer is "it would
  still find its line", the check is blind to crashes.
- Liveness belongs in the shared reader, not in the check.
- When a guard can only report through a channel that maps to SKIP, **change the
  channel**. A red result is not a nicety; it is the whole point.

Related: [[feedback_a_skip_is_not_red_so_a_check_can_stop_running_unnoticed]],
[[feedback_a_check_that_cannot_fail_is_not_evidence]],
[[feedback_a_hand_written_list_inside_a_completeness_test_is_the_gap]],
[[feedback_an_absence_claim_is_a_claim_about_every_route]].
Full write-up:
`D:\dev\rag\egui\show_viewport_immediate_may_run_its_callback_twice_per_frame_so_a_fnonce_body_aborts.md`
