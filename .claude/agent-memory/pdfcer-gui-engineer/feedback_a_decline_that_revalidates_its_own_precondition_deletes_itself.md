---
name: a-decline-that-revalidates-its-own-precondition-deletes-itself
description: A status-line decline whose still_true re-tests the condition that RAISED it is discarded on the same frame — the control goes silent, not wrong, and no test in the repo can see it.
metadata:
  type: feedback
---

**A decline that re-tests its own precondition deletes itself whenever the
precondition IS the complaint.** The verb returns, the sentence is written,
`Declined::still_true` runs a frame later, finds the condition false — because
the whole point of the arm was that it was false — and drops the sentence.
The operator sees a control that does **nothing at all**.

**Why:** 2026-09-12. *Select containing form* had been inert since the day it
shipped. Its decline re-tested `selection_in_form`, which is false by
construction on exactly the arm that raises it ("nothing selected is inside a
form"). Unit tests all passed: the verb returned the right `Declined`, and
they asserted on that return, never on what survived to the next frame.
Same family as [[feedback_unit_tests_that_call_the_verb_cannot_see_the_chain_in_front_of_it]].

⇒ **A silent control is indistinguishable from a broken one, and worse than a
wrong sentence** — a wrong sentence gets reported, silence gets attributed to
the operator having mis-clicked.

**How to apply:** whenever writing a `still_true` predicate, ask *what raised
this?* If the answer is "the absence of X" and the predicate tests for X, the
sentence cannot survive. The staleness condition for such an arm is usually
**"the selection changed"**, not "the precondition is still absent". And test
it by driving, not by asserting the verb's return — only a driven check sees
frame N+1.
