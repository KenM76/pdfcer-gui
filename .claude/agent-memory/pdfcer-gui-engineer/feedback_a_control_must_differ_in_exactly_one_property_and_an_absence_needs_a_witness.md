---
name: a-control-must-differ-in-exactly-one-property-and-an-absence-needs-a-witness
description: A negative assertion is evidence only when exactly one thing could have made it true — so the control is the damaged file with the trap removed, not a healthy file, and the same launch must carry a positive witness
metadata:
  type: feedback
---

**When a check asserts that something is ABSENT, two things are required: the
control fixture must differ from the subject in exactly ONE property, and the
same launch must assert a positive witness that proves the neighbourhood was
drawn at all.**

**Why:** on 2026-09-13 the recovery-losses disclosure needed a check that the
dropped-objects block draws *nothing* on a file that lost nothing. The obvious
control was a healthy PDF. It is the wrong control: on a sound document
`Document::recovery()` returns `None`, so the entire recovery neighbourhood
draws nothing, and *"the dropped block is absent"* is then satisfied by **three
different states at once** — the document was not recovered, the panel never
mounted, or the block correctly declined. **An assertion satisfied by all three
is not a measurement of which one shipped.**

The control that works is the damaged file **with the two traps removed** —
same numbering, same geometry, same font, still opened by rebuild-by-scan. It
differs in exactly one property, so a difference in the result can be
attributed to that property and to nothing else.

The engine session independently named the same shape the same day and called
it the reusable half of the exchange, having hit it twice from other
directions: a completeness test carrying its own copy of the set it checked,
and a gate reporting "clean" about a directory it never opened. Same family —
**a check that passes for a reason other than the one it claims.**

**How to apply:**

- Before writing an absence assertion, list every state that would satisfy it.
  If the list has more than one entry, the fixture is wrong, not the assertion.
- Pair it with a positive witness **on the same launch** — here
  `properties.recovery` had to be on screen for the absence of
  `properties.recovery-dropped` to mean anything.
- **Assert the fixtures' own properties through the engine, in the cheap unit
  suite.** A fixture that has quietly stopped being what the check assumes
  costs one second to catch there and ninety minutes to catch in a driven
  sweep — and the sweep catches it wearing the costume of an application
  defect, so someone then debugs the wrong file.
- Two assertions in the control, not one, when the properties fail
  independently and in opposite directions ("still recovered" and "still
  lossless" are different facts about one file).
- Assert the numbers, not the count. A count of two is satisfied by finding
  object 8 twice.

Related: [[feedback_a_check_that_cannot_fail_is_not_evidence]],
[[feedback_an_absence_assertion_is_only_as_good_as_when_its_baseline_was_taken]],
[[feedback_an_assertion_both_outcomes_satisfy_is_not_a_measurement_of_which_one_shipped]].
