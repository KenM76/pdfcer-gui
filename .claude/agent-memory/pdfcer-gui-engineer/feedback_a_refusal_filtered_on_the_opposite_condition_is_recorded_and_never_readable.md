---
name: a-refusal-filtered-on-the-opposite-condition-is-recorded-and-never-readable
description: A decline whose re-validation predicate is false by construction on the frame it is written produces SILENCE, not a wrong sentence — and a test that stops at the store is green the whole time
metadata:
  type: feedback
---

**A recorded refusal is not a shown refusal. Assert what the BAR reads, not
what the dispatcher wrote — and when a decline is filtered by a predicate,
check whether that predicate can be true on the frame the decline is
recorded.**

**Why:** 2026-09-11. The decline pipeline here is: dispatcher →
`record_inside_form()` → thread-local `LAST` → `live(ctx, doc)` re-asks
`still_true(...)` → `Declined::line()` → the status bar's `⊗` slot. The
re-validation exists for a good reason: a refusal whose cause has gone away
must not keep showing.

One `&'static str` served two verbs. The second, `format.select_form`, records
a decline when there is **no containing form to reach** — and `still_true`
filtered `InsideForm` on `selection_in_form`, which is **false by construction
on every frame that arm can run**. So the decline was written and discarded in
the same frame, every time, since the verb shipped.

★★★ **The symptom was silence.** Pressing the control with nothing form-interior
selected produced no outline, no movement and no explanation. A *wrong*
sentence is reportable — the operator quotes it back. Silence is not: he
concludes he mis-clicked, or that the button is for something else, and never
mentions it. **The defect class with no bug report is the one that lives
longest.**

★★ **And the sentence it would have shown was wrong in the opposite
direction** — *"That object is inside a form"* at the exact moment pdfcer had
established that **no** object is. One string, two call sites, wrong at both,
in opposite directions. Splitting it into a two-variant enum was not a tidy-up;
it was the only way either sentence could be correct.

★ **The test was green for the whole life of the defect** because it asserted
`recorded_for_test()` — the store the dispatcher writes to — and stopped there.
That is a test that a function was *called*. `live_for_test` had to be written
before the second link could be asserted at all.

**How to apply:**
- For any "the program tells the operator X" claim, the assertion must read the
  **last** stage before the pixels, not the first stage after the decision.
  Recorded → filtered → formatted → drawn: every hop is a place it dies.
- When you meet a re-validation guard, write down the condition under which the
  thing being guarded is recorded, and the condition the guard tests. If they
  are complements, the guard is a delete.
- **Silence is a test case.** An absence check needs a planted subject — see
  [[a-fixture-that-defeats-a-default-does-not-defeat-a-starting-state]] — and
  the operator will never file the report that would have found it.
- Related: [[unit-tests-that-call-the-verb-cannot-see-the-chain-in-front-of-it]],
  [[a-stages-trace-records-what-the-stage-decided]],
  [[a-long-green-check-can-be-aiming-at-nothing]],
  [[an-unevidenced-excuse-is-worse-than-silence]].
