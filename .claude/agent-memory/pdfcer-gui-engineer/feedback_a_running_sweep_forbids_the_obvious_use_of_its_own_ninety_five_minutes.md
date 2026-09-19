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

Related: [[feedback_never_drive_the_published_build]],
[[feedback_ui_verify_competes_for_the_machine]] and
[[feedback_a_stopped_background_task_is_a_claim_about_the_wrapper]].
