---
name: a-rule-with-a-classification-in-it-is-where-the-next-exception-goes
description: Prefer one exact token plus a written exemption over a rule that enumerates acceptable alternatives — the enumeration is the seam a future exception slips through.
metadata:
  type: feedback
---

**When writing a gate's rule, spend a token of conformance to keep the rule one
sentence with no taxonomy in it.** *"`std::process::id()` in the path, or
`// temp-path-exempt: <reason>`"* — not *"a process-unique component, and here
is how the checker recognises one."*

**Why:** 2026-09-13. Four scratch-path helpers were already safe on a wall-clock
nanosecond stamp; no realistic pair of processes would collide. The honest rule
would therefore have been *"a pid **or** a sufficiently fine timestamp"* — and
that version contains a classification, which means every future site arrives
with an argument about which class it is in and the gate acquires a second
branch to settle it. Those four took the pid anyway, for one token each. The
`nanos` stayed, because it separates repeated runs *inside* one process, which
the pid does not.

The same shape, stated generally: **a rule that enumerates acceptable forms has
to be read; a rule that names one token can be grepped.** A gate is only as
durable as the simplest description of what it forbids. Every clause added to
accommodate a legitimate exception is a clause a later illegitimate one can
argue itself into.

⇒ The corollary is that **exemptions must be explicit and must carry a reason**,
not implicit in the gate's blind spots. `// temp-path-exempt: nothing is ever
created here` is a decision on the record that a reviewer can disagree with.
A site the gate silently fails to notice is a decision nobody made.

**How to apply:**
- Writing a new gate: pick the narrowest literal token that discharges the
  hazard, then bring the already-safe outliers into conformance rather than
  widening the predicate to admit them.
- If a real site genuinely cannot take the token, that is an exemption with a
  written reason — never a new branch in the rule.
- Related: [[an-assertion-both-outcomes-satisfy-measures-neither]] and
  [[a-gate-keyed-on-a-name-is-discharged-by-prose]] — both are the same family:
  the predicate's shape decides what can hide from it.
