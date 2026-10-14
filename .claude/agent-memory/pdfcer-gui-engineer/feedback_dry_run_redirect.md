---
name: feedback-dry-run-redirect
description: A "dry run" made by sed-rewriting a script's hard-coded root did not match, so the script edited the real tree mid-gate-run
metadata:
  type: feedback
---

A dry run that works by rewriting a script's hard-coded path is only dry once
the rewritten line has been checked. A sed `s|D:\\Dev...|...|` silently matched
nothing (backslash escaping). `apply_slice1.py` then ran against the real source
tree and wrote six .rs files while a full gate run was going. The gate run was
invalidated and had to be killed by PID and restarted (about an hour lost).

**Why:** sed exits 0 when nothing matches, and the script printed "applied",
which looked exactly like a dry-run success.

**How to apply:** give apply scripts a root argument (`sys.argv[1]`, with no
default) instead of a constant. If you must rewrite a constant, `grep` the
substituted line and abort unless it names the scratch root. Never run any
apply script while a gate or ui-verify run is going. Related:
[[feedback_running_sweep_forbids]], [[feedback_verify_result_diff]].
