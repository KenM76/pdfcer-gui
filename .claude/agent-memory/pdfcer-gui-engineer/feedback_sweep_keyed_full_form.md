---
name: a-sweep-keyed-on-the-full-form-cannot-see-the-abbreviated-form
description: A pattern written for the full form of a reference is blind to its abbreviated continuation, and the abbreviation is the form that accumulates fastest
metadata:
  type: feedback
---

A sweep keyed on the **full** form of a reference cannot see the
**abbreviated continuation** of that same reference, and the abbreviation is
usually the commoner of the two, because a cell or paragraph that has already
named the file adds its next sites without repeating it.

**Why:** every engine-citation sweep here was keyed on `<file>.rs:<N>`. It
could not see `` `:NNNNN` `` — a backticked line number with no file name,
reading as the second half of the citation before it. There were twelve of
them in `FORMS_PARITY.md` alone, more than a third of the drifted total in
that file, and they were found only by measuring what the repair had *left
behind*, not by the sweep that drove it. Folded into the gate afterwards as a
fourth detector with its own falsification.

**How to apply:** before quoting a sweep's count, write down the shorthand a
human author would naturally use for the same thing and grep for **that**
separately — a bare number, an ellipsis, an `ibid`, a `"` ditto, a `…and the
next two`. Then widen the pattern and re-measure; the delta is the part of the
population the first count silently excluded. Related:
[[an-absence-claim-is-a-claim-about-every-route]],
[[a-pattern-over-a-human-document-is-a-claim-about-its-notation]],
[[a-drifted-citation-lands-on-a-real-unrelated-symbol]].
