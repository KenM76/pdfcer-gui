---
name: a-plant-harness-keyed-on-error-calls-every-working-plant-a-broken-build
description: A falsification script that classifies cargo output by "error:" reports DID NOT COMPILE for every plant that actually produced reds — key on "could not compile"
metadata:
  type: feedback
---

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
reason [[feedback_a_falsification_can_lie_in_both_directions]] matters: a
misread harness is a falsification lying in the direction that looks like
diligence. Full recipe in `D:/dev/rag/rust/cargo_test_prints_error_on_a_red_suite_so_a_harness_keyed_on_error_reports_a_build_failure.md`.
