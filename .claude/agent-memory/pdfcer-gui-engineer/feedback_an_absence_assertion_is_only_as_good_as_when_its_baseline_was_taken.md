---
name: an-absence-assertion-is-only-as-good-as-when-its-baseline-was-taken
description: A count taken before the setup and again after the action asserts "the total grew", not "the action produced one". Plant the defect and watch the check stay green.
metadata:
  type: feedback
---

**A check that counts events before an action and again after it is asserting
that the TOTAL grew.** Everything the setup emitted in between satisfies it —
and the setup is usually the richest source of the very event the check wants
to see absent.

## The evidence, 2026-09-05

`scrolling_far_keeps_the_canvas_its_pointer_input` moved the pointer (1
`canvas-pointer` line), turned the wheel, moved the pointer again, and required
`after > before`.

A defect was **planted** in `canvas::trace::pointer` — an early `return` past a
scroll offset of 700 pt, exactly the failure the check claims to detect. The
check **passed**: sixteen pointer lines had accumulated *while the wheel was
turning and the offset was still small*, so `16 > 1` held and the final move —
which produced nothing at all — was never isolated.

Corrected to compare against the count taken **immediately before the final
move**, it fails against the plant with the intended message, and passes on two
fixtures once the plant is reverted.

★★ **The plant is the only thing that found this.** The check had been repaired
that same hour and re-run and looked healthy; it was green for the wrong
reason.

## The same check's other fault, for the shape of it

Its original failure message asserted *"the page is still drawn and its rect is
still published — only the input is gone."* The trace of that very run ended:

```text
canvas-unavailable reason=nothing-visible
ui-rect-gone name=canvas-viewport
ui-rect-gone name=page
```

Forty wheel notches had scrolled a one-page document clean off the viewport.
**The message stated a precondition the run never measured.**

## The rules

- Take the baseline **at the last possible instant before the action under
  test**, after all setup has settled. Keep the pre-setup control as well and
  say what each is for — one separates *"the feature broke"* from *"this build
  never emits that line"*, the other makes the assertion about the action.
- **Never state a precondition in a failure message that the run did not
  measure.** If the message says "the page is still drawn", the check must have
  asked.
- **Plant the defect.** A check repaired and re-run is not a check verified; it
  is a check that agrees with the current build. This applies to every
  monotonic counter — event counts, file sizes, undo depths, render counts.
