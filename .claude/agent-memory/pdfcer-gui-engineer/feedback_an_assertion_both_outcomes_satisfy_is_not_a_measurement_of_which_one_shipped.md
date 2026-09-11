---
name: an-assertion-both-outcomes-satisfy-is-not-a-measurement-of-which-one-shipped
description: Before trusting a green test about WHICH mechanism ran, name the observable the wrong mechanism cannot produce — "an edit happened" was equally true of the faked bold and the real one for five days.
metadata:
  type: feedback
---

**When a test is supposed to prove that the RIGHT mechanism ran, ask what the
WRONG mechanism would have done to the same assertion. If both satisfy it, the
test is not measuring the thing its name claims.**

**Why:** 2026-09-11, the bold ladder. The engine had grown a four-rung style
ladder on 2026-08-30 — use a real face on the page, else the standard-14 sibling
of the run's own family, else a donor folder, else thicken the strokes. This
shell kept calling the old verb for **five days**, so on his commonest page (a
title block carrying `Helvetica` and nothing else, therefore no bold resource on
the page) Bold thickened the letters while `Helvetica-Bold` was one line away.

`bold_applies_on_a_page_with_no_bold_face` was green the whole time. It asserted
**one** thing: that the edit epoch moved. **The edit epoch moves on the faked
weight too.** The test's name described the interesting case; its assertion
described a property both outcomes share, so it reported "bold works on a page
with no bold face" while what shipped was the outcome the name was written to
rule out.

**How to apply:**

- **Name the discriminating observable before writing the assert.** Here it was
  the run's `/Font` **resource key changing** — synthesis cannot do that, it
  thickens strokes on the same resource. One field, and it separates the two
  branches cleanly.
- **The tell is a test whose name contains a mechanism and whose assert contains
  a side effect.** "…uses a real bold face" asserting "something changed" is the
  shape. So is "…falls back to X" asserting "no error", and "…picks the nearest
  Y" asserting "a Y was picked".
- **Falsify by putting the OLD call back**, not by breaking the input. That is
  what proved the new assertion discriminates; breaking the input only proves
  the test can go red for some reason.
- Distinct from [[a-check-that-cannot-fail-is-not-evidence]], which is about a
  check that never **reached** the mechanism. This one reaches it, observes it,
  and measures a property that does not depend on it. Both end in a quoted
  green; only this one survives a precondition assertion.
- Distinct from
  [[unit-tests-that-call-the-verb-cannot-see-the-chain-in-front-of-it]], which
  is about the surfaces in front of the verb. This defect is **inside** the
  verb's own test.
- The five-day gap has its own cause, recorded in
  [[a-limitation-sentence-is-a-citation-with-an-hours-long-shelf-life]]: the
  engine shipped the capability and nothing here noticed. The green test is why
  nothing looked wrong in the meantime — same family as
  [[write-the-row-when-he-speaks-not-when-the-work-lands]].
