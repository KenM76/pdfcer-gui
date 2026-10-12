---
name: park-drafts-in-d-scratch
description: "Drafts parked while gates run go in D:\scratch\<topic>\, never D:/temp. Coordinator directive, 2026-10-12."
metadata:
  type: feedback
---

Park drafts, apply scripts, control binaries and gate logs in
`D:\scratch\<topic>\` — not `D:/temp`.

**Why:** the coordinator asked for it on 2026-10-12 after the arrow-key nudge
draft was found in `D:/temp`; `D:\scratch` is the agreed scratch root, and
free-space work is allowed only inside the project or that scratch.

**How to apply:** while a gate run holds the index (see
[[feedback_running_sweep_forbids]]), write the next change as an apply script plus
drafts under `D:\scratch\<topic>\`, then run it after the commit.
