---
name: a-falsification-scripts-anchors-rot-and-a-rotted-one-reports-nothing
description: The script that plants a defect anchors on the code's current shape; change the shape and it silently plants nothing, which looks exactly like a script nobody needed to run
metadata:
  type: feedback
---

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
