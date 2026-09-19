---
name: a-preview-must-be-replayed-not-read-off-the-targets-current-rect
description: Any "where will this land" feedback is produced by replaying the operation through the real verb, and the test that calibrates it cannot see an error shared by both sides
metadata:
  type: feedback
---

Feedback that promises an **outcome** — a drop highlight, a move preview, a
paste target, a diff of what a verb would do — is produced by *replaying* the
operation: clone the state, apply the candidate through the same verb the
commit will call, and re-read the result with the same function the frame will
draw with. Never by looking the target up in the current geometry.

**Why:** the naive version is wrong for exactly the operations that change the
layout, which includes the commonest drag: taking the only panel out of a stack
prunes its column, so every surviving compartment is re-laid before the panel
arrives. The operator is shown half a side and given a whole one. It is also
the R8b defect in miniature — showing one thing and committing another — and it
is invisible to a test set that only exercises moves within a settled layout.

**How to apply:** whenever the shell offers pre-commit feedback about a verb
the engine or the model owns. Two tests, not one:

- the **calibration** — render a real frame, assert the shared arithmetic
  agrees with every rect the frame actually drew. This is what makes the replay
  trustworthy, and it goes red the moment the draw path stops sharing the
  definition.
- the **step arithmetic, asserted directly** — because the calibration is blind
  to any error the draw path and the replay *share*. Dropping the splitter from
  the walk moved both together and stayed green; only the direct assertion
  caught it. Same shape as
  [[feedback_oracle_needs_calibration]] and the reason
  [[feedback_an_assertion_both_outcomes_satisfy_is_not_a_measurement_of_which_one_shipped]]:
  name what the *wrong* mechanism cannot produce.

Recipe in `D:/dev/rag/egui/a_drop_preview_read_off_the_targets_current_rect_describes_a_layout_the_release_will_not_produce.md`.

---

## ★★ The same blindness in a DRIVEN check: the promise and the outcome came from one number — 2026-09-18

The calibration rule above is about a unit test. It recurs in `ui-verify` and is
harder to see there, because driving the real binary *feels* like the
independent measurement.

The tear-out affordance draws a window outline, converts its corner to desktop
points by adding the application window's origin, and hands that to the new
window as its position. The obvious check — *did the window open where the
outline promised?* — compared the **published** corner against the **window
that corner placed**. Both carry whatever the conversion produced.

Falsified by deleting the `+ origin` term: a real window opened **788 pt left
and 71 pt up** of the outline the operator was shown, and the check said
**PASS**. Two quantities, one source.

⇒ **The expected value must be computed from a source the program under test
did not produce.** Here: the application window's client origin, read from the
OS through the session handle, plus the outline rectangle as drawn. Same plant
then fails by name, quoting both corners and the offset.

**How to apply — the one question to ask of any driven equality assertion:**
*could a single wrong value satisfy both sides?* If the answer is yes it is a
tautology wearing a measurement's clothes, and it will be green on exactly the
defect it was written for. Two specific smells:

- both sides read from the **same trace line family** (`x=` published, `y=`
  published), rather than one from the trace and one from the OS, the fixture,
  or a constant the check owns.
- the check's failure text could never name a number the program does not
  already believe.

Related: [[feedback_an_assertion_both_outcomes_satisfy_is_not_a_measurement_of_which_one_shipped]],
[[feedback_a_value_cannot_identify_which_producer_made_it]],
[[feedback_a_check_that_cannot_fail_is_not_evidence]].
