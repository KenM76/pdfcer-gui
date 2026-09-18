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

## A second instance, and this time the liveness check was the defect

A full gate sweep was reported *"stopped because the system is running low on
memory"*. It had not stopped. It ran to completion and wrote
`RESULT: PASS - 61 passed, 0 failed, 0 skipped`, fmt and clippy included, into
the same log, roughly fifteen minutes after the notification.

What made me believe it: I checked the log twice, one second apart, saw the same
byte count, and read that as death. **A no-growth window shorter than the
slowest single unit of work is not evidence of anything** - some gates in that
suite take minutes, so a quiet log is the normal appearance of a healthy run.
Measure over longer than the slowest step, or do not measure that way at all.

And the second half: I reached for `tasklist | grep -c bash.exe`, which answered
`16` and told me nothing, because `tasklist` does not print a command line. The
query in this entry - `Get-CimInstance Win32_Process ... Select CommandLine` -
is written the way it is precisely because the count is useless and the command
line is the whole answer. Having the right query recorded and reaching for a
convenient wrong one cost twenty-seven gates re-run by hand.

**A completion notification CAN still arrive after a kill notice**, so the
earlier entry's *"no completion notification is coming"* is a property of that
2026-09-12 case, not a rule.

Related: [[a-runners-sentinel-is-a-claim-about-the-runner]],
[[a-launch-failure-blamed-on-a-resource-count-needs-a-control-binary]],
[[a-measurement-of-the-wrong-surface-looks-exactly-like-a-broken-one]].
