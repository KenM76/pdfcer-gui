---
name: anchoring-a-citation-without-opening-it-freezes-the-error
description: Prefixing a version onto an unversioned citation makes a wrong coordinate permanent — open every one in the source; four of thirty-one were wrong
metadata:
  type: feedback
---

Version-anchoring an unversioned citation (`style.rs:1135` →
`egui-0.35.0/src/style.rs:1135`) looks like a mechanical prefix. It is not. The
prefix converts a coordinate that was *going* to be checked into one nobody
will ever check again. **Open every one in the vendored source before you
anchor it.**

**Why:** thirty-one were anchored in one sitting and each was opened first.
Four were wrong: a quoted sentence one line below the cited line; a range
starting two lines early; a range whose claimed content was thirteen lines
down; a line holding a `// Retrocompatibility` comment rather than the code
described. Two more named the wrong directory. None of that is visible from
the citation — it reads exactly as a right one does — and anchoring without
opening would have made all six permanent and authoritative.

**How to apply:** treat the anchoring pass as the *audit*, not the paperwork.
The same logic covers a citation you are merely re-indenting or reflowing:
touching it is the cheapest moment it will ever be checked. Related:
[[an-archived-trees-citations-froze-wrong]],
[[a-quotation-i-wrote-myself-can-carry-a-line-number]].
