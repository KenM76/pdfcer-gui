---
name: a-runners-sentinel-is-a-claim-about-the-runner
description: A batch runner that finishes and prints its done-marker has said nothing about whether any subject ran — tally what happened, abort on a command-line rejection, read the script's own verdict line not the exit code, and if the runner re-derives its list from another script, inherit that script's variables too
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

Related: [[a-check-that-cannot-fail-is-not-evidence]],
[[a-skip-is-not-red-so-a-check-can-stop-running-unnoticed]],
[[a-trace-grepping-check-passes-on-a-build-that-crashed]],
[[a-commit-message-can-describe-work-that-never-landed]].

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

## ★★★ A RUNNER THAT RE-DERIVES ITS LIST INHERITS THE VARIABLES TOO — 2026-09-21

A foreground slice runner for `tools/gates/run-all.sh`, written so it could not
read a stale list — it re-greps the `run "…"` lines on every invocation — still
carried two silent defects, and both were about the *transformation* it applied
to those lines rather than the list itself.

**Defect one, caught by falsification.** It did `cmd="${cmd//\"/}"` to strip
quotes before `eval`. Planting three fake gates as `run "x" bash -c "exit 1"`,
all three reported **PASS** and the runner exited 0. The strip turned
`bash -c "exit 1"` into `bash -c exit 1` — `exit` with `$0=1`, which exits 0.

The tempting reading is *"my fixture was the wrong shape, the real gates are
fine"*, and it is even true: all 65 registered entries are
`<interp> "$HERE/<script>" [--flag]` with no argument carrying a space, and a
re-falsification with real gate-shaped scripts gave a correct
`FAIL/SKIP/PASS`, `runner exit=1`. **That reading is still the wrong response.**
A transformation that is sound only for the shape you happen to have is a claim
nobody will recheck when the shape changes. The fix is to make the runner
*refuse* a shape it cannot handle — a regex on the post-substitution command,
`exit 2` on a miss — so the next quoted argument with a space aborts loudly
instead of being reported green.

**Defect two, found only because the refusal was written.** Running the shape
regex across all 65 registered commands as a dry check, five did not match:
they are written against **`$ROOT`**, not `$HERE`, and the runner substituted
only `$HERE`. Those five would have run against an empty path — `python
/tools/check-suite-name-absent.py` — and failed for a reason having nothing to
do with the repository. They all sit in entries 23–65, which is the only reason
the earlier 1–22 green survived the discovery.

⇒ **A runner that re-derives its work list from another script has taken a
dependency on that script's whole environment, not just its lines.** Grep the
source for every `$VAR` the list can contain and define all of them, or assert
that the post-substitution command contains no `$`.

**How to apply:** whenever you write a partial/slice/subset runner over an
existing batch script — because the full one is too slow, keeps being killed, or
must run in the foreground. Two questions before quoting it: *what does it do to
each line before running it, and for which shapes is that sound?* and *which
variables does the source script define that mine does not?* Then falsify it
with a fixture of the **real** shape — a fixture of the wrong shape can fail for
a reason the real subjects never hit, which reads as a runner defect and wastes
the falsification.

Related: [[a-falsification-can-lie-in-both-directions]],
[[a-detectors-scope-is-a-claim]], [[a-check-that-cannot-fail-is-not-evidence]].
