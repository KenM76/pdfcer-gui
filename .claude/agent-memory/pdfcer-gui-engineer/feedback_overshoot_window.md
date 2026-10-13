---
name: feedback-overshoot-window
description: A check probing a band between two thresholds must prove it sampled inside the band; "a few notches more for safety" landed past the far wall and passed on the defect
metadata:
  type: feedback
---

A driven check whose defect lives between two walls must picture at the first
wall and SKIP when the trace shows the far side — never overshoot "to be safe".

**Why:** the off-page halo-wall check stepped three notches past the halo's
wall, crossed the sheet's wall too, and PASSED on the unfixed build because the
sheet's region tier draws the content regardless. Only the pre-fix falsification
caught it.

**How to apply:** for any threshold-band check, read the pictured frame's
decision from the trace and assert it is the in-band one. Related:
[[feedback_assertion_both]], [[feedback_check_cannot_fail]].
