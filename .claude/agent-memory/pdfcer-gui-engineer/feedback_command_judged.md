---
name: a-command-judged-through-a-pipe-reports-the-pipes-exit-code
description: `cargo build ... | tail -15` reported exit 0 on a build that FAILED — the rc belongs to `tail`. Redirect to a file and read the command's own `$?`. Same family: `hits=$(awk … "$f")` discards awk's status, so a gate whose scanner crashed on all 545 files printed "clean".
metadata:
  type: feedback
---

**A command piped into `tail`/`head`/`grep` reports the PIPE's exit code, not
its own.** `cargo build --release --workspace 2>&1 | tail -15` came back
**"exited with code 0"** on a build that had failed outright — and the harness's
own completion summary repeated the 0, which is what made it convincing.

**Why:** 2026-09-15, cutting the sixth release. The build died with
`error: failed to remove file target/release/pdfcer-gui.exe / Access is denied
(os error 5)` — an **off-screen smoke launch from four hours earlier had never
been closed and still held the exe**. The error text was right there in the
15 lines `tail` printed; I read the exit code instead of the lines, and moved on
to package a **stale binary**. What caught it was the exe's mtime, not any rc.

**How to apply:**

- For any build, test run, gate sweep or packaging step whose result you will
  ACT on: `cmd > file 2>&1; echo "RC=$?"` then `tail` the file. Never
  `cmd | tail`. (`set -o pipefail` also works, but it is not on by default in
  these one-shot Bash calls and a forgotten `pipefail` looks identical to a
  green build.)
- **The tell for this class is a suspiciously fast success.** The relink after
  the failure took 3.31s; a real release build of this workspace is ~80s.
  Treat "finished far too quickly" as a rc-you-did-not-measure until proven
  otherwise.
- **Close the off-screen smoke launch as part of the smoke check, by PID
  verified against `Path`.** A `PDFCER_DIAG_VIEWPORT` launch left running is
  invisible (it is at -4200,-4200), holds `target/release/pdfcer-gui.exe`, and
  the next release build fails at the LINK step — i.e. after everything
  compiles, which is the point in the run where output is least likely to be
  read. See [[never-kill-pdfcer-gui-by-name-he-uses-it-all-day]] for
  how to kill it safely.

**It recurred on a gate sweep, 2026-09-19, and the shape was the opposite.**
`bash tools/gates/run-all.sh 2>&1 | tee "$TEMP/gates.txt"` reported
**"exited with code 0"** while its own last line read `RESULT: FAIL — 1 gate(s)
found a violation.` The rc was `tee`'s. I nearly wrote it up as a runner defect
— *"run-all.sh exits 0 on FAIL"* — and only reading the script's tail showed it
does `exit 1`, `exit 3`, `exit 0` correctly. **A pipe does not only hide a
failure; it manufactures a false finding about the tool you piped.** Before
blaming a script's exit code, check whether you measured the script or the
pipeline. `tee` is the sneakiest of the three, because unlike `tail` it is
there to be helpful about output rather than to truncate it.

Related: [[a-commit-message-can-describe-work-that-never-landed]] (a
`;` chain let a failed edit look committed — same family: the shell reported on
something other than the step that mattered), and
[[the-thing-you-measured-is-never-the-thing-you-ship]].

**Third recurrence, 2026-09-20, and here the exit code was never read at all.**
`check-ui-strings.sh` printed `ui-strings: clean` with rc=0 while its awk
program did not compile, over a tree holding **64 known violations** — twice in
one session, two different awk defects. The loop was the natural shape:

```bash
file_hits=$(awk '…program…' "$file")
[ -n "$file_hits" ] && violations+=("$file_hits")
```

Command substitution captures **stdout only**; awk's diagnostics went to stderr
and into a per-gate log nobody reads unless the gate is red. `var=$(cmd)` does
set `$?`, but nothing read it, and `set -e` does not fire on a status consumed
by a later conditional. So a fatal parse error produced empty stdout — read as
*"this file is clean"* — 545 times in a row.

★ **A scanner's "no findings" output and its "I could not run" output are the
same bytes.** Every source-scanning gate in this repo has that shape.

**How to apply:**

- Guard every per-file scan with `|| { … exit 1; }`, and make the banner name
  the **gate** as the fault, not the file — a reader's first instinct on red is
  to go open the file the gate named.
- Falsify the guard by planting a deliberate awk syntax error
  (`BEGIN { rawhash = index(1,2,3) }`) and running against the real tree. A gate
  scanning two roots has two invocations and needs two guards.
- ⇒ **Decide what a CRASH looks like before deciding what a violation looks
  like.** A `--self-test` proves only that a planted *violation* reddens it, and
  says nothing about the case where the scanner never ran.

Recipe: `C:/personal_rag/claude_code/lesson_20260920_command_substitution_hides_awk_crash_gate_reports_clean.md`.
