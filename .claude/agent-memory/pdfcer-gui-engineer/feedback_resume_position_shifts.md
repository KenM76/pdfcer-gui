---
name: feedback-resume-position-shifts
description: A sweep's resume position belongs to one selection; committing changes the --affected list, so derive it again and drive what slid ahead of it
metadata:
  type: feedback
---

`--start=N` is a position in one list. Re-derive it after any commit, and drive separately whatever the new list placed ahead of it.

**Why:** the crash cut a sweep at 181 of 547. My Escape commit changed the selection to 560; the same check sat at 183, and two newly selected checks fell ahead of the resume point where the sweep would never reach them.

**How to apply:** also re-drive the neighbours of the fix that ran earlier on the old binary; those positions carry a stale verdict.
