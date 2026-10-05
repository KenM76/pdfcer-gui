---
name: feedback-ps-blind-sibling
description: Git Bash `ps -ef` in one Bash call cannot see processes started by another call; "no process" plus a stalled log is not a dead gate run
metadata:
  type: feedback
---

A gate run with no RESULT line and no process in `ps -ef` is not proof that it died. Git Bash's `ps` lists only that MSYS session's processes, so a sibling background call's `run-all.sh` does not appear. check-stale-blockers alone sits silent for minutes.

**Why:** on 2026-10-05 I declared a backgrounded run-all dead and started a second one. The first then finished with RESULT: PASS, and I had to kill the duplicate by PID. Two runs also contend for the tight disk/RAM ([[project_disk_tight_target]]).

**How to apply:** check liveness with PowerShell, `Get-CimInstance Win32_Process | ? CommandLine -match 'run-all.sh'`, or wait for the task notification. Only re-run when that is empty AND the log has no `rc=` line.
