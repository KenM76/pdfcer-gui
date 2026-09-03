---
name: walk-the-series-never-pick-endpoints
description: Two samples either side of a transition look exactly like a system with no transition — read the whole series before claiming a behaviour is absent.
metadata:
  type: feedback
---

When measuring how something behaves across a range — window widths, zoom
levels, page counts — **read every sample, not the endpoints.** A claim that a
transition does *not* exist is only as good as the density of the sampling
behind it.

**Why:** 2026-08-25. Twelve photographs of Word's ribbon were captured to learn
its scaling rules; **three were read.** From frames at 1300 pt and 800 pt I told
Ken *"groups do not re-wrap onto more rows as the window narrows"*. False — the
Font group is 2 rows at 1900 and 3 at 1000. By 800 it had already **collapsed**,
a later stage of the same ladder, so the reflow was on screen in neither frame.
The claim reached a module header, `RIBBON_SCALING.md`, an operator-facing
answer and a commit message, and all four had to be retracted. **The disproving
frames were already committed in the repo.**

★ The shape, because it recurs: **two samples either side of a transition are
indistinguishable from two samples of a system that has no transition.** Both
endpoints agree, so confidence is *high* — that is the trap, not carelessness.
Same failure as testing `f(0)` and `f(100)` and concluding `f` is linear.

**How to apply:** sweep, don't spot-check. This project already had the right
pattern one function away — the collapse ladder's monotonicity test walks 600
widths one point at a time. Apply that discipline to the **evidence-gathering**,
not only to the tests, or the whole chain rests on three photographs. And when
Ken corrects a factual claim, the evidence is usually already on disk: go and
read what was captured before arguing.

★★ Corollary on retracting: fix the claim **everywhere it landed** — source
comments included. A wrong sentence in a doc comment is read as measured fact
by the next session.

Related: [[learn-a-reference-app-by-photographing-it]],
[[a-measurement-of-the-wrong-surface-looks-exactly-like-a-broken-one]],
[[kens-sentences-are-reports-not-measurements]].
