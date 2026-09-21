---
name: a-census-of-what-the-program-emits-cannot-be-completed
description: Never build a check on an enumeration of what the program produces — names reach a channel by routes a static scan cannot see, and an incomplete census does not miss cases, it accuses live code.
metadata:
  type: feedback
---

**A check that compares a documented thing against a CENSUS of what the program
actually produces is unsound, because the census cannot be completed.** Build
the negative test instead: ask only whether the thing appears *anywhere*, by the
loosest search that still means something.

**Why:** the first cut of `check-trace-names`'s mechanism 5 compared trace names
documented in module headers against every first token of a `format!` inside a
trace call — a reasonable-looking enumeration of emitted names. It immediately
accused `ui-rect`, which is live and emitted every frame. Trace names reach the
diagnostic channel by at least four routes: `format!`, a bare `"...".to_owned()`
literal, an `eprintln!` inside `diag`, and helpers like `diag::ui_rect(name)`
whose argument is a **runtime string that no static scan can enumerate at all**.

The asymmetry is the whole point. An incomplete census does not merely miss
defects — **it blames correct code**, and a gate that fails on correct code is a
gate somebody switches off, taking its real findings with it. A loose test errs
toward missed defects; a precise-looking incomplete one errs toward false
accusations. Loose in the safe direction beats precise in the unsafe one.

**How to apply:**
- Writing a check of the form *"X is documented/declared/registered but never
  actually …"* → the second half must be a plain substring or grep, not an
  enumeration of the constructs you believe produce it. Then state in the gate's
  own "what it cannot see" section which direction its errors fall in, and why
  that direction was chosen.
- Before believing any new check, **falsify it against real files, not only its
  self-test** — see
  [[falsify-the-gate-against-the-real-files-and-the-fix-against-a-control-binary]].
  Mechanism 5 was run over
  the sources at the commit before its defect was fixed: it named the dead
  entry at the right file and line and cleared the other 25. A self-test proves
  only that a mechanism fires on input its own author planted.
- This is [[an-absence-claim-is-a-claim-about-every-route]] mechanized. That
  rule governs an absence claim written in prose; this one governs an absence
  claim compiled into a gate, where the same mistake runs on every commit.
