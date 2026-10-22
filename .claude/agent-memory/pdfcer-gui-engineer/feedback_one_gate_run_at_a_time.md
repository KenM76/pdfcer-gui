---
name: one-gate-run-at-a-time
description: Two background run-all.sh runs writing the same log interleave and neither verdict is evidence; check for a live run before starting one
metadata:
  type: feedback
---

Before starting `run-all.sh` in the background, check that no other run is
alive (`Get-CimInstance Win32_Process | ? CommandLine -match 'run-all'`, in
the PowerShell tool — escaping the regex through Bash breaks it). Across a
context compaction a run started by the earlier half of the session survived
and a second one started on top; both redirected to `gates.txt`, both shared
`target/`, and both "completed" with no RESULT line.

**Why:** two `>` redirects to one file write at independent offsets, so the
log holds pieces of both runs; the shared target dir also makes the build
gates race. A verdict read from it is attributable to neither commit.

**How to apply:** one run at a time; if two are found, kill both trees by PID
(`taskkill /PID n /T /F`) and run once clean. Compaction summaries list the
background task ids — check those before re-launching. See
[[feedback_gate_run_no_timeout]], [[feedback_stopped_background]].

**A failed first step is not a finished chain.** A `{ test; clippy; run-all; }`
chain wrote `TESTS rc=101` and kept going; a rerun started on that line ran
beside its clippy and gates, on the same logs. Wait for the task
notification, or kill the chain by PID, before rerunning.
