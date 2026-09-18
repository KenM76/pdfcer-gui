---
name: a-falsification-can-lie-in-both-directions
description: A red falsification proves the check as a whole, not each assertion in it; a green one may mean the planting script's anchor rotted and it mutated nothing; and a script that scores the run by scanning cargo output can call every working plant a broken build
metadata:
  type: feedback
---

Two ways a falsification run misleads, and they point opposite directions.

## Red, but it proves less than it looks

**A falsification proves a check AS A WHOLE discriminates. It says nothing
about whether each assertion inside it can be reached.** After planting a
defect and watching the check go red, ask of every `if` in it: *what input
reaches this line?*

**Why:** O185's driven check had three assertions. The one its own module
header called load-bearing — the cross-run comparison that is the only thing
ruling out "Cancel and Keep are the same button" — **could never fire**, because
a per-run guard earlier in the file already implied it. Two falsification runs
had both gone red and neither exposed it: the other two assertions caught both
planted builds and reported them well. Being red is evidence about the check,
not about its parts.

**How to apply:** when a check has more than one assertion, falsify it once per
assertion, each plant aimed at only that one — or read the guards above them
and prove by hand which inputs survive to each line. And when an assertion turns
out to be unreachable, the two cases are not the same defect:

- **Unreachable because of ORDERING** — an earlier guard or an earlier
  assertion already implies it. That is a defect. Reorder so the strongest
  claim is tested first, where it can still fail.
- **Unreachable because the DOMAIN is currently too small** — the value it
  guards against has only two possible tokens today. That is a legitimate guard
  against the domain widening, and it must be **labelled in-comment as one**, or
  the next reader counts it as evidence the check does not actually provide.

Related: [[feedback_a_check_that_cannot_fail_is_not_evidence]],
[[feedback_an_assertion_both_outcomes_satisfy_is_not_a_measurement_of_which_one_shipped]],
[[feedback_a_long_green_check_can_be_aiming_at_nothing]].


## Green, because nothing was actually planted

**A falsification script is code with a dependency on the shape of the thing it
mutates. When that shape changes, the script does not fail — it matches
nothing, plants nothing, and reports that the test is still green.**

**Why:** 2026-09-13. `f_o196.py` planted its defect by deleting the line
`options.units = remembered.units;`. Clippy then forced `seeded_options` into
functional-update syntax, so that line became `units: remembered.units,` inside
a struct literal. The script's anchor no longer existed. Re-running it would
have printed a green result for a test that was never actually challenged —
and a green falsification run is the strongest evidence this project accepts.

Two adjacent failures of the same family, same day:

- **An em dash in the prose makes a patch-script anchor fail.** The crate's
  comments use `—` (U+2014) freely; typing `-` looks identical in a terminal.
  Verify with `grep … | cat -A` (`M-bM-^@M-^T`) and then **choose an anchor that
  contains no em dash at all**, rather than trying to reproduce one.
- **`cd` inside a Bash tool call persists into later calls.** A walker run from
  the scratchpad reported "0 declared" and looked like a clean tree.

**How to apply:**

- Every patch and falsification script asserts its anchor count is exactly 1
  and exits non-zero otherwise. That single line converts all three failures
  above from silence into a message.
- Re-falsify after any refactor that touches the mutated function, not only
  after a change to the test.
- The one-line check that the falsification actually bit: the planted run must
  exit **101** (a Rust test panic), not 0 and not 1. A script that exits 0 on
  the "should be red" leg has not run the test.
- Restore from the `.bak` copy the script wrote, never with `git checkout`.

Related: [[feedback_the_write_python_to_a_file_workaround_does_not_protect_an_escape]],
[[feedback_a_check_that_cannot_fail_is_not_evidence]],
[[feedback_never_git_checkout_to_undo_an_experiment]],
[[feedback_a_long_green_check_can_be_aiming_at_nothing]].

---

## Red, and scored by a script that read the wrong word

When a plant harness runs `cargo test` and decides the outcome by scanning
output, key on **`could not compile`**, never bare `error:` — and print the
**names** of the failing tests, not only how many.

**Why:** a red suite ends with `error: test failed, to rerun pass …`. That is
the test failing, not the build. Scoring seven plants with
`if "error:" in out: DID NOT COMPILE` reported six broken builds and one green
— when the truth was six precise reds and one genuine blind spot in the test
set. The misclassification is almost invisible in this particular instrument,
because a plant harness *expects* some deliberate defects not to type-check, so
"DID NOT COMPILE" reads as an ordinary result rather than a broken tool. The
count-only variant hides the neighbouring defect: a plant that reddens the
*wrong* test is itself a finding.

**How to apply:** any time falsification is scored by a script rather than by
eye. Same genus as [[feedback_a_command_judged_through_a_pipe_reports_the_pipes_exit_code]]
— an outer layer's success word standing in for the inner one's — and the
third direction this file is about: a misread harness is a falsification lying
in the direction that looks like diligence. Full recipe in `D:/dev/rag/rust/cargo_test_prints_error_on_a_red_suite_so_a_harness_keyed_on_error_reports_a_build_failure.md`.

---

## Green, because the mechanism you aimed at is IMPLIED by another — 2026-09-18

The third direction, and the only one where the green is a finding about the
**production code** rather than about the harness.

Seven plants against the dock's tear-out tests; **three green**, all three
aimed at the mechanisms that appear to enforce *only one affordance answers a
drag* — two stand-down guards and the stated branch order in the settlement.
All three are implied by the tear's own geometric predicate, so no input
reaches them and no test can redden them.

★ **Do not delete the implied mechanism.** The predicate is expected to move,
and a release build where the implication breaks should do the safe thing. Keep
it, label it in-comment as unreachable-under-today's-predicate, and move the
claim to an assertion where the decision is consumed — every driven test runs
through that.

★★ **Falsifying the replacement takes two plants and the first is expected
green**: widen the predicate and the guard absorbs it — *that* green is the
evidence the guard is live under the one input that reaches it — then widen it
and remove the guard, and the assertion fires by name.

★★★ **A tripwire behind an assertion that fires earlier is an unfalsified
tripwire.** Every test asserting the absence *before* the release failed there
instead, so the assert was never reached; it had to be falsified by deleting
that intermediate line in one test.

Related: [[a-check-that-cannot-fail-is-not-evidence]],
[[a-tripwire-keyed-on-your-own-intention-is-not-a-tripwire]].

---

## The scorer itself lies three more ways, and all three read CAUGHT-or-clean — 2026-09-18

Thirteen plants against two driven dock checks, scored by a shell runner that
greps the report for the defect sentence. Three of the thirteen were scored
wrongly before the runner was fixed.

- **The probe matched the check's own advertisement.** A `ui-verify` report
  prints a `detects:` line describing what the check is *for*, in the same
  words as the defect sentence. The probe `"release put"` matched that line, so
  it read **CAUGHT** while a completely different branch ran.
  ⇒ **Filter the report's self-description out before grepping it.** Anything a
  check prints unconditionally is not evidence about this run.
- **The probe was defeated by line wrapping.** A sentence long enough to wrap
  never matches a multi-word probe. Read **NOT CAUGHT** over a perfect catch.
  ⇒ Flatten newlines and squeeze whitespace before matching.
- **★ The plant silently did not apply, and the run reported PASS.** A python
  edit raised on a failed `assert`, the tree stayed clean, the check passed —
  and a *green that means nothing was planted* is indistinguishable from *green
  because the mechanism holds*. Caught only because the runner prints its own
  verdict per plant rather than relying on the suite's.
  ⇒ **A falsification runner must fail loudly when its own edit did not land** —
  `git diff --stat` after planting, or a non-zero exit from the editor. Same
  genus as [[a-commit-message-can-describe-work-that-never-landed]].

**How to apply:** a falsification score has three inputs — the plant applied,
the check ran, the probe matched — and all three can fail silently in the
direction that looks like success. Print all three per plant.
