---
name: a-stopped-background-task-is-a-claim-about-the-wrapper
description: "'The background command was stopped' kills the task handle, not the detached bash it launched; check for the script's own command line before writing a recovery plan"
metadata:
  type: feedback
---

When a background task reports *"was stopped because the system is running low
on memory"*, that is a statement about **the harness's handle on the job**, not
about the job. Establish that the work stopped before acting on the belief that
it did — one query:

```powershell
Get-CimInstance Win32_Process -Filter "Name='bash.exe'" | Select ProcessId, CommandLine
```

and look for the **script's own command line**. Follow it with
`find target/scratch -type f -newermt '-3 minutes'`, which catches a job whose
stdout is buffered.

**Why:** on 2026-09-12 a full `ui-verify` sweep was reported stopped partway
through chunk 6. It was not. `bash tools/ui-verify/sweep-full.sh` ran for
another hour — same log, same file descriptor, same chunk sequence — while the
session recorded it as dead, diagnosed a nonexistent orphaned process as the
cause, and wrote a resume-from-check-101 plan that was pure waste. Three things
hid it: the notification's wording is about the task; `ui-verify` buffers stdout
to a file so a healthy chunk shows nothing for twenty minutes; and **no
completion notification is coming**, because the sender is what was stopped, so
the silence confirms the wrong story.

**How to apply:** any long `run_in_background` job — sweeps, builds, gate runs.
Before "resuming" anything, prove it is not still running. And the near-miss
that makes this urgent: believing the sweep dead, the next planned step was to
edit `sweep-full.sh` (safe from the stale-binary guard because it is a `.sh`).
**Bash reads a script by byte offset**, so editing a script a live `bash` is
executing resumes it inside arbitrary new bytes. Never edit a shell script any
process might currently be executing.

Related: [[a-runners-sentinel-is-a-claim-about-the-runner]],
[[a-launch-failure-blamed-on-a-resource-count-needs-a-control-binary]],
[[a-harness-with-a-bad-input-produces-defects-that-do-not-exist]].
