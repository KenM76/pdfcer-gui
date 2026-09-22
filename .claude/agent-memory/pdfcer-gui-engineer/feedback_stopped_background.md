---
name: a-stopped-background-task-is-a-claim-about-the-wrapper
description: "'The background command was stopped' kills the task handle, not the detached bash it launched; a quiet log lags the run and is not a stall; and killing the runner makes the wrapper report exit 0, so only the RESULT: line separates a killed sweep from a passed one - and a RESULT: grep aimed at the wrong log file makes exactly the same silence"
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

## Third instance, and killing it reports EXIT 0 - 2026-09-20

The same wrong inference with the right instrument in hand. I ran the process
query this entry prescribes, it answered that `check-strong-text.sh` was ALIVE,
and I killed the sweep anyway - because I had also watched the log sit at the
same byte count for two minutes and believed the log over the process. It was
not stalled: seconds after the kill the tree still held a live
`check-plate-colour.sh`, two gates further on. **The log lags the run. The
process list does not.** When the two disagree, the process list is the
measurement and the log is a cache.

The new half, and it is the dangerous one: **killing the runner makes the
wrapper report `[exited with code 0]`.** A sweep destroyed mid-flight and a
sweep that passed are indistinguishable by exit code, and the notification
says *completed*. The only thing separating them is the runner's own
`RESULT:` line, which a killed sweep never writes - the same rule as
[[a-command-judged-through-a-pipe-reports-the-pipes-exit-code]], one layer
out.

**How to apply:** never conclude a stall from a static byte count alone; and
before quoting any sweep green, grep its log for the `RESULT:` line rather
than reading the task notification.

## Fourth instance — the kill notice arrives BEFORE the log is finished

A gate sweep was reported *"stopped because the system is running low on
memory"*. Tailing the log at that moment showed 65 gate headers and
`>> cargo fmt` as the last line, which reads exactly like a run cut off two
gates from home. Minutes later the same file held
`RESULT: PASS — 65 passed, 0 failed, 0 skipped`, fmt and clippy included. The
tell that it was still alive was in a command run for another purpose: a manual
`cargo clippy` printed *"Blocking waiting for file lock on build directory"*,
which is a live cargo saying so.

⇒ **A `tail` taken at the instant of the kill notice measures the log's lag,
not the run's state.** Grep for `RESULT:` — and if it is absent, grep again
later before re-running anything. Re-running a 95-minute sweep that had already
passed is the cost of reading the tail once.

## Fifth instance — I grepped the WRONG log and called it "no verdict"

A gate sweep was reported stopped for memory. I looked for its verdict, found
no `RESULT:` line, and concluded the run had died two gates from home with 70
banners and nothing decided. The sweep had in fact finished and written
`RESULT: FAIL - 68 passed, 2 failed, 0 skipped` — into a **different log**. I
had located the file by the newest-looking name in the scratchpad instead of by
the redirect the command itself was given, so the absence I measured was an
absence in a file the run never wrote to.

⇒ The four rules above all assume you are reading the right file. Add one
before them: **identify the log by the command's own redirect target**, from the
task's recorded command line or the tool result that launched it — never by
`ls -t`, never by remembering what it was called. A `RESULT:` grep against the
wrong path and a killed run are the same silence, which makes this failure
invisible from the inside. Same shape as
[[grep-manufactures-absence]].

And the verdict it hid was cheap: one of the two failures was a one-line
crossref typo in a memory file, and the other was the known clippy
`0xc0000142`, which cleared on a single `-j 2` re-run.

Related: [[a-runners-sentinel-is-a-claim-about-the-runner]],
[[a-launch-failure-blamed-on-a-resource-count-needs-a-control-binary]],
[[a-measurement-of-the-wrong-surface-looks-exactly-like-a-broken-one]].
