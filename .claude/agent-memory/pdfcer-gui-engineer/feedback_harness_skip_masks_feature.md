---
name: harness-skip-masks-feature
description: A ui-verify check whose feature surface is missing SKIPs instead of FAILing, because `press` on an absent region is a harness error; guard the feature's own surface explicitly
metadata:
  type: feedback
---

When a driven check presses a control the FEATURE draws (a dialog, a properties
section), breaking the feature makes `press` fail to find the region — a harness
error, which the runner files as SKIP, not FAIL. Falsifying G161 by removing the
dispatch arm turned the add-link check SKIP, not red.

**Why:** a SKIP reads as "environment" and is easy to wave through; the feature
was gone and nothing was red. Same family as [[feedback_skip_red_check_stop]].

**How to apply:** before the first press on a feature-owned surface, assert it
drew — a trace event (`place-armed kind=X`) or a `ui-rect name=<region>` after a
mark — and return a FAIL message if not. Falsify every such guard; each should
go red, not SKIP.
