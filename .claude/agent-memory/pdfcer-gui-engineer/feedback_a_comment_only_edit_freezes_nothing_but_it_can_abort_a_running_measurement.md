---
name: a-comment-only-edit-can-abort-a-running-measurement
description: While a driven sweep runs, the source tree is FROZEN — including comments, including a crate that is not the one under test; the harness's own staleness guard aborts the run and the tally still prints
metadata:
  type: feedback
---

**While a measurement is running, do not edit a `.rs` or `.toml` file anywhere
in the tree — not even a comment, not even in a crate the measurement is not
about.**

**Why:** 2026-09-13, mid-release. A 221-check driven sweep was an hour and a
half in. To fill the wait I repaired a stale sentence in a **module header
comment** in `tools/ui-verify/src/checks/raster_wall.rs` — a harness file, not
application code, and a comment at that, so it could not change a single byte of
the binary under test. Chunk 161 then returned **rc=2** and the sweep **aborted**
with 61 checks unrun:

> `ui-verify: STALE BINARY — refusing to run.` … `newest: raster_wall.rs` …
> *"The source is 68 minute(s) newer than the binary."*

The guard is **right** and is one of this project's better instruments: a stale
harness drives the wrong fixtures and asserts on renamed trace keys, so its
failures are confident and about the wrong subject. It does not — and should
not — try to work out whether my edit was semantically inert. `staleness_complaint`
scans the source root for anything with extension `rs` or `toml` and compares
mtimes; **Markdown is not scanned**, so documentation work during a sweep is
safe and source work is not.

★★★ **The part that would have cost a day: the run still printed a TALLY.**
`=== TALLY passed=138 failed=3 skipped=19` is a perfectly ordinary-looking line,
and 138+3+19 = **160**, not 221. A tally that is a count of what happened to run
reads exactly like a tally of the suite. The `=== ABORTED:` line was four
screens above it. ⇒ **Check that a sweep's total equals the roster before
quoting any part of it** — this is [[a-runners-sentinel-is-a-claim-about-the-runner]]
in its other direction: there the runner claimed 210 checks and ran none; here it
ran 160 and the number was simply smaller than nobody was checking against.

**How to apply:**

- Before starting a sweep, decide that the tree is frozen until it ends, and do
  documentation work (`.md`) in the wait — that is the only category that cannot
  abort it.
- Resuming is legitimate **when the application binary was never touched.**
  `sweep-full.sh` copies `target/release/pdfcer-gui.exe` to
  `target/scratch/drive/` at the start and drives that copy, so rebuilding the
  *harness* and re-running the remaining chunks measures the same program the
  earlier chunks measured. Say so in the resume script's header, or the resumed
  half looks like a different experiment.
- The resume must re-derive its chunk boundaries from `target/scratch/checks.txt`,
  which the aborted run already wrote — not from a fresh `--list`, which would
  renumber if a check had been added.

Related: [[never-git-checkout-to-undo-an-experiment]] and
[[a-stopped-background-task-is-a-claim-about-the-wrapper]] — the same family:
what a long-running job actually did is a measurement, not an assumption.
