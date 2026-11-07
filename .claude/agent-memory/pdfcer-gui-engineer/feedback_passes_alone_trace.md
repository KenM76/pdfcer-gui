---
name: a-sweep-failure-that-passes-alone
description: A sweep-only failure that passes alone still needs its trace read: name the mechanism before calling it load.
metadata:
  type: feedback
---

A check that fails in a sweep and passes alone is not yet a flake: read the sweep's own trace and name what happened.

**Why:** The double-tap check's sweep trace showed the second pair of taps as two single gimbal taps, which turned the view; alone it passed four of four. That names a wall-clock double-tap window under load, which is a different finding from "flaky", and it is owed a fix.

**How to apply:** Compare the failed run's trace with a passing one, line by line at the failure. Record the mechanism in RESUME, then re-drive alone at least three times. [[feedback_suite_shares_state]]
