---
name: a-falsification-can-lie-in-both-directions
description: A red falsification proves the check as a whole, not each assertion in it; a green one may mean the planting script's anchor rotted and it mutated nothing
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
