---
name: a-check-whose-input-is-chosen-for-convenience-tests-the-assertion
description: A driven check typed the cheapest keys because "what is typed does not matter" — and stayed green through the exact defect those keys avoided
metadata:
  type: feedback
---

When a check needs input, choose it from **what the subject could get wrong**,
never from what is convenient to supply. An input picked for the assertion's
convenience tests the assertion.

**Why:** on 2026-09-08 Ken reported that pressing Enter in the Markup text box
produced `?` instead of a new line. The driven check covering that exact route —
arm the tool, drag a box, open the dialog, type, accept — had been green
throughout, and its own comment said why:

> *"Two keys that already exist in `sys::vk`. WHAT is typed does not matter —
> the Accept control is gated on the field being non-empty and nothing else."*

True of what it asserted. And the dialog accepting anything is precisely why the
cheapest input was chosen, and precisely why the one character the route
mishandled was never sent.

**How to apply:**

- Ask: *what input would distinguish this working from this broken?* If the
  answer is "several, and I picked none of them", the check is about its own
  plumbing.
- **Prefer the awkward character.** Newlines, tabs, non-ASCII, the empty string,
  the repeated value, the one glyph the subset lacks. Convenience and coverage
  point in opposite directions here.
- A comment saying *"what is typed does not matter"* is a **finding**, not a
  justification. It means the check has stopped constraining the thing under it.
- Sibling of
  [[feedback_count_a_condition_and_you_will_reason_about_the_other_one]] (build
  test inputs from the subject's own alphabet) and
  [[feedback_a_check_that_cannot_fail_is_not_evidence]].

★ Same session, same shape, opposite direction: my engine request asserted *"the
single-line branch already does the right thing"* from reading its code. Their
test failed on **four** cases, not the three I predicted — the branch was
correct given real newline bytes and never received any. **Reading code
carefully and not running it produced four wrong claims in one day.**
