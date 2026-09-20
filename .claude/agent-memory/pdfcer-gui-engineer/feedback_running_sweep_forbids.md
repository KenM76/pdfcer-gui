---
name: a-running-sweep-forbids-the-obvious-use-of-its-own-ninety-five-minutes
description: While a full ui-verify sweep is running, editing ANY .rs or .toml aborts the whole remaining run via two staleness guards; markdown, fixtures and new shell scripts are safe, sweep-full.sh itself is not.
metadata:
  type: feedback
---

**While `tools/ui-verify/sweep-full.sh` is running, do not edit any `.rs` or
`.toml` anywhere in the repository.** Markdown, RAG files, new fixtures and
*new* shell scripts are safe. `sweep-full.sh` itself is **not** safe, because
bash reads a running script incrementally.

⚠ **This is about `sweep-full.sh`, and NOT about `tools/gates/run-all.sh`.**
The two get conflated — both are called "the sweep" — and the conflation is a
scheduling error in the expensive direction: the gate run is minutes, not
ninety-five, and none of the staleness guards below are in it. Before deciding
to wait, check WHICH sweep is running. (What the gate run does share is the
last bullet's other half: it ends in `cargo fmt`/`clippy`/`test`, so do not
start a competing cargo job against it.)

**Why:** two independent staleness guards exist precisely to stop a check from
being believed against a binary that no longer matches its source —
`ui-verify/src/main.rs::refuse_if_self_is_stale` compares the running
`ui-verify.exe` against the newest `.rs`/`.toml` under `tools/ui-verify/`, and
`launch.rs::staleness_complaint` compares the driven `pdfcer-gui.exe` against
its own sources. Either one turns every remaining chunk into an `rc=2` usage
dump, and `sweep-full.sh` aborts the whole run on the first `rc=2`. **A full
sweep is about ninety-five minutes of wall clock, and the obvious thing to do
with ninety-five idle minutes is source work — which is exactly what it
forbids.** Both guards are correct and neither should be relaxed; the cost is
paid in scheduling, not in weakening them.

**How to apply:**

- Start the sweep, then queue **documentation** work: RAG lessons, `RESUME.md`,
  `HANDOFF.md`, `FEATURES.md`, release notes, `OPERATOR_REQUESTS.md` rows, the
  skip/fail triage table. Measurements that only read (`git`, `--list`, a
  backlog walker, a python counter) are fine.
- **Do not run `cargo build`, `cargo test` or `cargo clippy` either** — not
  because of the guards but because the release `ui-verify.exe` is locked and a
  four-job build competes with a timing-sensitive driven run; a check that waits
  on a frame will report a defect that is really CPU contention.
- Triage as the chunks land rather than at the end. The per-chunk tally is in
  `target/scratch/sweep-full.log` (`[PASS]` / `[FAIL]` / `[SKIP`, ending
  `=== SWEEP-DONE`), and writing the repair for a SKIP while its trace artifact
  is fresh costs a fraction of reconstructing it later.
- If a chunk comes back `rc=2` with a usage dump, **suspect your own edit before
  suspecting the harness.** That is the signature.

Related: [[never-drive-the-published-build]],
[[ui-verify-competes-for-the-machine]] and
[[a-stopped-background-task-is-a-claim-about-the-wrapper]].

## Comments count, and so does a crate the sweep is not testing

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
