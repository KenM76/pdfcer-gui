---
name: a-checks-own-gesture-can-satisfy-the-condition-it-was-written-to-catch
description: The check's own arrangement satisfies the condition it was written to catch — a scroll through every page leaves nothing for prefetch to order, and a fixture that defeats a default still does not defeat a starting state
metadata:
  type: feedback
---

Before blaming a fixture for a check that cannot reach its subject, ask whether
the **gesture the check performs** satisfies the precondition the check is waiting
to observe being violated.

**Why:** on 2026-09-13 `pages_stay_drawn_when_you_scroll_back` was finally given
the eight-page document it had been skipping for want of, and still could not
reach its subject. It watches `strip-raster-requested`, which is emitted at one
place only: the scan for a page that is **visible, not the current page, and has
no raster**. The check scrolls. The scroll is a decaying smooth glide, so at the
fit zoom the view passed *through* pages 1, 2 and 3, each became the current page
on the way, and each was rastered by the current-page path. **The cache filled
itself by the very act of travelling through it**, and scrolling *further* makes
that more true, not less.

The same fact disarmed a second assertion in the same check: `strip-raster-evicted`
never fired either, so two assertions shared one unreachable precondition while
appearing to test different things.

**How to apply:**

- Read the **emitter** of the event before the fixture. One grep for the trace
  slot's only `emit` site gave the whole answer here; three sessions of fixture
  work would not have.
- **Jump, do not travel.** Set the page number, or Ctrl+End — a gesture that
  arrives without visiting the intermediate state is the one that leaves the
  intermediate state unsatisfied.
- A bigger input is the instinctive repair and is often the wrong axis entirely;
  see [[a-checks-own-gesture-can-satisfy-the-condition-it-was-written-to-catch]],
  which is this same family.
- Assert on the **positive** event (`...-evicted`) rather than inferring it from
  the absence of a request. An inference from an absence is green when the
  mechanism is merely unreachable.
- The program was correct throughout. A check that cannot reach its subject is a
  harness defect, and in this repository's measured experience that is the way to
  bet — twenty-one harness defects to nil across two full sweeps.

## The fixture variant — defeating a default is not defeating a starting state

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
