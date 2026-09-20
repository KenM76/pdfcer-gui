---
name: one-remedy-banner-on-any-nonzero-exit-sends-the-reader-to-break-a-right-figure
description: A tool with two independent failure modes and one closing "here is what to do" banner prints the wrong remedy for the other mode — and the remedy was "rewrite the headings", which were correct.
metadata:
  type: feedback
---

When a checker can fail for two unrelated reasons, the closing advice must be
gated on the reason that actually fired — not on `exit != 0`. Track each
failure in its own flag.

`walk-engine-backlog.py` had one `bad` flag set by both a heading disagreement
and an over-cap row. On a pure cap failure it still printed *"The headings do
not agree with the walk. Rewrite them FROM THESE FIGURES"* — under a table
whose figures it had just confirmed agree. The banner is an instruction, and
following it means retyping five correct numbers.

**Why:** a gate's output is read by a cold session that did not write it, and
the last paragraph is the one it acts on. A remedy that fires unconditionally
is indistinguishable from a remedy that was diagnosed, and it is wrong more
often than it is right — here, any of the 184 rows growing past the cap
triggered it while every heading was correct.

**How to apply:** when adding a second failure condition to an existing
checker, split the flag before writing the message, and falsify **both
directions**: plant a violation of each kind in a copy of the real input and
confirm each prints its own remedy and *not* the other's. Related:
[[feedback_a_gate_that_crashes_after_its_headline_reads_as_a_broken_tool]],
[[feedback_a_falsification_can_lie_in_both_directions]].
