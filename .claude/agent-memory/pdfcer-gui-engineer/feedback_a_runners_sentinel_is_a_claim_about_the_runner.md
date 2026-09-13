---
name: a-runners-sentinel-is-a-claim-about-the-runner
description: A batch runner that finishes and prints its done-marker has said nothing about whether any subject ran — make it tally what actually happened and abort on a command-line rejection
metadata:
  type: feedback
---

A runner's completion sentinel proves the **runner** finished. It is not
evidence that anything it was supposed to drive ever started. Every batch script
needs a **tally counted out of its own log** printed beside the sentinel, and a
**command-line rejection must abort**, not be repeated once per batch.

**Why:** measured 2026-09-11. `tools/ui-verify/sweep-full.sh` was started against
a harness whose source was 32 seconds newer than its binary — I had edited a
check's doc comment after the last `cargo build`. `ui-verify`'s staleness guard
refused, correctly and at length, **once per chunk**. Eleven chunks and two
ALONE records, every one `rc=2`, the whole sweep over in under a minute, and the
log ending in `=== SWEEP-DONE`. The script's own header said *"its absence means
the sweep was killed"* — and that sentence was true, which is exactly why it was
useless. **210 checks reported as swept; zero launched anything.** Had I not
noticed the elapsed time was wrong, that log would have been quoted as the
driven sweep RESUME has wanted for three releases.

The three-part fix, and each part closes a different half:

1. **Build the binaries inside the runner.** The whole failure existed because
   the script took "somebody ran cargo build" on trust.
2. **`rc=2` aborts.** Exit 2 is the harness saying *the command line was wrong* —
   a fact about the invocation, identical for every remaining batch. Nineteen
   more copies of the same usage dump is not more evidence; it buries the one
   line naming the cause under a thousand lines of help text.
3. **A tally that must add up.** `passed=0 failed=0` printed next to the
   sentinel is a number a reader can disbelieve. A sentinel alone is not.

**How to apply:** any time a script loops a sub-command and prints a completion
marker — sweeps, gate runners, fixture generators, release packagers. Ask *what
number would be wrong if nothing ran?* and print that number. If the answer is
"none of them", the runner reports only on itself.

Related: [[feedback_a_check_that_cannot_fail_is_not_evidence]],
[[feedback_a_skip_is_not_red_so_a_check_can_stop_running_unnoticed]],
[[feedback_a_trace_grepping_check_passes_on_a_build_that_crashed]],
[[feedback_a_commit_message_can_describe_work_that_never_landed]].

## ★★★ AND THE EXIT CODE THAT COMES BACK IS THE WRAPPER'S — 2026-09-13

`bash tools/gates/run-all.sh` overran the 600-second tool timeout, was moved to
the background, and the completion notice read:

    Background command "Run all gates" completed (exit code 0)

with the log file ending `[exited with code 0]`. The log's own last three lines,
immediately above that, read:

    45 passed, 2 failed, 0 skipped
    RESULT: FAIL — 2 gate(s) found a violation.

`run-all.sh` ends `if [ "$nf" -gt 0 ]; then ... exit 1; fi`. It exited 1. The
zero belongs to the harness's backgrounding wrapper, which succeeded at running
the thing.

⇒ **A long command's reported exit code stops being the command's the moment
it is backgrounded.** This is the same shape as the stopped-task memory — the
status that comes back describes the wrapper's job, not the work's — and it is
worse here, because `exit code 0` on a gate runner is precisely the sentence
that ends an investigation.

**How to apply:** for anything whose verdict matters — gates, sweeps, test
runs — read the **script's own printed verdict line** out of the log and never
the reported code. If a runner has no such line, that is the first defect to
fix: make it print a tally that can be zero, and a `RESULT:` line that can say
FAIL, before trusting it at any length. And prefer `run_in_background: true`
from the start for anything that might overrun, so the exit code was never going
to be the oracle in the first place.
