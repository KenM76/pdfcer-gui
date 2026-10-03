---
name: feedback-cleanup-overwrites-failure
description: A check's trailing cleanup step with `?` (pointer.gone after a FAIL) can turn a detected defect into a SKIP — a found failure must outrank a cleanup error
metadata:
  type: feedback
---

A driven check that found the defect still reported SKIP: the trailing `pointer.gone(session)?` queued behind a scripted step that a now-closed window never acknowledged, and its error replaced the FAIL (O278 plant "Escape closes the viewer").

**Why:** the pointer seam acknowledges steps in order; a window the defect closed leaves every later step unacknowledged, so any cleanup `?` after the verdict errors exactly when the defect is real. A SKIP is a misfiled FAIL here ([[feedback_skip_red_check_stop]]).

**How to apply:** route the verdict through a `park(session, pointer, failure)` that ignores the parking error when a failure is already held; and when a step can be lost because the defect closed its window, check for the close trace before propagating the step error. Falsify with a plant that closes the window, not only plants that leave it open.
