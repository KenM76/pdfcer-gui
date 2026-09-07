---
name: a-contract-you-write-for-someone-elses-function-is-a-claim-to-measure
description: When your own code is replaced by an engine API, its doc comment still describes the OLD one; and a suite that only exercises one sign of a value is not testing the value.
metadata:
  type: feedback
---

**When a local implementation is replaced by somebody else's function, its doc
comment describes the function you deleted until you have re-measured it. And a
test suite that only ever exercises one SIGN of a value is not testing the
value.**

**Why:** 2026-09-07. `canvas::annotquad` began the day as a declared workaround
with its own `/Matrix` reader; the engine shipped
`appearance_placement` / `appearance_rotation_degrees` that afternoon and it
became a thin adapter. The rewrite carried the old `degrees` doc comment across
— *"normalised to `[0, 360)`"* — which was true of the deleted local code
(it applied `rem_euclid`) and **false of the engine's, which returns a signed
`atan2`**. `is_upright` range-tests `(0.1..=359.9)`, so `-89.15` fell outside
it: **every clockwise rotation reported itself upright** and the selection
outline went back to axis-aligned on the turn an operator makes most often.

3,852 in-process tests green. 31 gates green. Every test in the module used a
**positive** angle, so none of them could see it. Found in ninety seconds by
`tools/ui-verify`'s `rotating_a_markup_turns_it`, whose drag happens to go
clockwise.

**How to apply:**

- On any adapter swap, **re-read the replaced function's own doc comment for
  each property you are about to restate** — range, normalisation, units, sign,
  what `None` means. Do not port the sentence; port the measurement.
- Assert **both signs / both ends** in ONE test rather than adding a
  negative-case test beside the positive ones. A test named *"a negative angle
  normalises"* is as easy to leave unwritten as the case it guards; naming the
  property *the round trip is sign-agnostic* is what makes it get written.
- The same shape applies to any bounded quantity: zero, empty, negative,
  wrapped, and past-the-end.

Related: [[a-limitation-sentence-is-a-citation-with-an-hours-long-shelf-life]],
[[a-compile-error-is-an-invitation-to-read-the-reply]],
[[a-tripwire-keyed-on-your-own-intention-is-not-a-tripwire]].
