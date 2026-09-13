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

## ★★★ 2026-09-13 — the variant where the check picks the WRONG ONE of two right answers

`check-pin-citation.sh` failed a `RESUME.md` row that was correct:

    | Engine pin | <re-measure command> | `3e73a02` — moved from `d86cb19` at 09:40 today |

The extractor ended in `tail -1`, so it read the sha the row says it moved away
from. My first instinct was that the document needed rewording — which would
have been editing a correct sentence to please a broken instrument.

★★ **The reason it is worth a memory is the DIRECTION of the error.** `tail -1`
does not merely mis-read a row that narrates. It reads the pin a genuinely
STALE row is most likely to carry, because the house style everywhere in this
project is *value first, explanation after*:

  - correct row  — `` `new` — moved from `old` `` ⇒ `tail -1` sees `old` ⇒ false RED.
  - **stale row** — `` `old` — will move to `new` `` ⇒ `tail -1` sees `new` ⇒ **false GREEN.**

So the false alarm I was looking at was the *cheap* half. The same line of code
had a silent half, and the silent half is the one the gate exists to catch.

★ **The tell that generalises:** when a check reads one value out of a line
that can contain several, ask which one a DEFECTIVE line would put where the
check is looking. A selector chosen for the shape of a healthy line is not an
assertion about the line; it is an assertion about the layout, and a defect
usually arrives as a change of layout.

★ The repair carries two self-test cases, not one, and the second is the point:
a narrating row must pass, AND a stale value cell must still fail **when the
locked sha appears later in that same row's prose**. Case 8 was verified to be
un-survivable by the old code before the fix was accepted — `head`/`tail` on
the synthetic row printed the two opposite shas.

★ And the widening was resisted: the obvious way to stop the false alarm is
to match the pin anywhere on the row. That is exactly the looseness that lets
case 8 through. **A gate can always be quietened by reading more broadly, and
reading more broadly is how it stops measuring.**
