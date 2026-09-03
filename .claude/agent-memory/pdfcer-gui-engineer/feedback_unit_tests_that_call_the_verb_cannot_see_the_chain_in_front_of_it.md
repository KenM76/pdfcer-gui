---
name: unit-tests-that-call-the-verb-cannot-see-the-chain-in-front-of-it
description: A test that calls the action directly and reads the document back proves the last link only; write the driven check BEFORE believing the feature works, and expect it to find defects the unit tests structurally cannot.
metadata:
  type: feedback
---

**When a feature is a chain of surfaces ending in an engine verb, the unit tests
prove the last link and nothing in front of it. Write the driven check before
believing it works, and budget for it finding real defects — not for it
confirming.**

**Why:** 2026-08-27, the text-restyle feature. Eight unit tests called
`textstyle::apply` directly and asserted on the **document after the edit**
(already the good discipline — never on the function's own report of itself).
All eight passed. The first press of Bold in the running program restyled **one**
piece of a fourteen-piece selection and stopped.

Driving found four defects, and every one was structurally invisible to those
tests because they supplied hand-picked run indices:

1. a text *run* is not a show *operator* — real producers emit runs that span
   several, so the operand was wrong on real files and right on fixtures;
2. derived-whitespace runs cannot be pinned and the loop **stopped** on them,
   so the first one ended the gesture;
3. the refusal path returned with **no trace line**, so the harness reported
   "nothing happened" over eleven completed edits;
4. two trace lines shared an event name, so the check read the wrong one.

The tests were not weak. They could not construct the case, because the case is
*what a real page's real producer emitted* — see
[[a-check-that-cannot-fail-is-not-evidence]].

**How to apply:**
- In the driven check's header, write the **chain table**: one row per link,
  and a column saying which links have their own test. The rows with ★ nothing
  are what the check is for. If every row already has a test, the check is
  redundant and should say so instead of being written.
- Aim the check at the operator's own file (`SW41177.pdf`), not at a fixture.
  Three of the four defects above are properties of his producer's output.
- Falsify in the same session — cut the feature out, rebuild, confirm the check
  fails with the operator's own symptom, put it back.
- Expect the first two or three runs to fail for **harness** reasons and read
  each one as a question about what it SAMPLED before concluding anything about
  the program — see [[a-measurement-of-the-wrong-surface-looks-exactly-like-a-broken-one]].
