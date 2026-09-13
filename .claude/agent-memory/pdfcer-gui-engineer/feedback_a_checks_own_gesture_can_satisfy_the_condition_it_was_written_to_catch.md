---
name: a-checks-own-gesture-can-satisfy-the-condition-it-was-written-to-catch
description: A scroll that travels through every page leaves nothing for the neighbour-prefetch path to order, so the check can never reach its subject
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
  see [[feedback_a_fixture_that_defeats_a_default_does_not_defeat_a_starting_state]],
  which is this same family.
- Assert on the **positive** event (`...-evicted`) rather than inferring it from
  the absence of a request. An inference from an absence is green when the
  mechanism is merely unreachable.
- The program was correct throughout. A check that cannot reach its subject is a
  harness defect, and in this repository's measured experience that is the way to
  bet — twenty-one harness defects to nil across two full sweeps.
