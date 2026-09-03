---
name: a-fixture-that-defeats-a-default-does-not-defeat-a-starting-state
description: An "it must NOT have happened" check is vacuous when the run already stands where the defect would land — plant the defect, don't just pick a good fixture
metadata:
  type: feedback
---

For any assertion of the form *"X must not have happened"*, ask what state the
defect would leave behind and whether the run is **already standing in it**. If
it is, the check cannot fail — and it fails in the direction that reads as
success. Then plant the defect and confirm red. A planted defect that passes
means the check is decorative, however carefully the fixture was chosen.

**Why:** 2026-09-01, driving PDF link following. The engine's fixtures were
built deliberately so that **no link targets page 1**, against exactly the
defect of a resolver returning a defaulted `0` — and the author said so in the
fixtures' own notes. The navigation check aimed at the furthest target. All of
that was right and the *sibling* check was still vacuous: it asserted only that
the page had not changed after clicking a non-navigable link, the fixture opens
on page 0, and the planted defect navigates to page 0. It **passed**.

The distinction that was missed: the fixture's property is *"the correct answer
is not the default"*; what an absence assertion needs is *"the STARTING STATE is
not the default"*. Two different variables. Fixing one does not fix the other.

**How to apply:** move the pre-state away from the defect's destination — here,
zoom in before clicking — and assert on **everything the defect could have
moved** (page, zoom *and* scroll offset), not the single field that first comes
to mind. This is the sharper version of [[a-check-that-cannot-fail-is-not-evidence]]:
that one is about checks that never saw the mechanism, this one is about checks
that saw it and could not distinguish it from the status quo. Filed to
`D:/dev/rag/rust/`.
