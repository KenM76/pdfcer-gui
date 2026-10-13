---
name: neighbour-checks-encode-defect
description: Fixing a shared funnel can redden passing checks that had encoded the old defect as expected; re-drive every check reading the same trace line
metadata:
  type: feedback
---

Fixing behaviour in a shared funnel can turn green neighbouring checks red, because they had
been written against the defect. When the insert funnel began landing on the pages it added,
two checks (clip_pages and dropped_pdf) failed. One expected `first=2`; the other assumed the
view was still on page 0.

**Why:** a check written by observing the binary records what the binary did, including its
defects. Its green status was evidence of consistency, not of correctness.

**How to apply:**
- After changing a funnel, grep ui-verify for every check that reads the same trace line (here
  `insert-pages-landed` and `canvas page=`) and re-drive them.
- Update a check only after you confirm the old expectation was the defect.
- Report which neighbours SKIP under `--no-input`, because they are undriven against the change.

Related: [[feedback_later_feature_captures_gesture]], [[feedback_unit_tests_call_verb]].
