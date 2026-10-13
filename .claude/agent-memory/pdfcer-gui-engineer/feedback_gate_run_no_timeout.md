---
name: feedback-gate-run-no-timeout
description: Never wrap run-all.sh in `timeout N`; a loaded machine takes over an hour and rc=124 throws the whole run away
metadata:
  type: feedback
---

Run `tools/gates/run-all.sh` with `run_in_background` and no `timeout` wrapper; wait on the task notification.

**Why:** 2026-10-13, `timeout 3500` killed the radio-hotfix gate run at gate 48 of 84 (rc=124) after 58 minutes, with every gate so far green. An hour of gates was lost on a release the coordinator was waiting for. Polling loops keyed on `pgrep` (blind in Git Bash) also kept running after it; they had to be stopped by PID.

**How to apply:** one background run, no timeout, no polling loops; read the log only after the notification. See [[feedback_running_sweep_forbids]] — no .rs edits while it runs.
