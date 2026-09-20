---
name: tidying-an-input-changes-every-instrument
description: Tidying an input is a change to every instrument that reads it — an archiving sweep blinded a gate and killed 18 citations, and a rename commit silently deleted every binary under evidence/, leaving the pixel oracle SKIPping green for twelve days
metadata:
  type: feedback
---

<!-- old-name-exempt-file: one finding here is the rename commit deleting every binary under evidence/, so the old name is quoted as evidence for what that commit did. -->

**Closing an exchange is part of doing the work, not a tidy-up pass. Archive
both files and write the `INDEX.md` row in the same sitting as the `done_*`, or
the folder becomes an append-only log whose count means nothing.**

**Why:** on 2026-09-14 `open/` held **48** files. **Forty-four of them were
already closed** — worked, consumed, written up, several with a
`done_*_CONSUMED.md` sitting in `open/` directly beside the `reply_*` it closed.
Eleven topics (`G002`×2, `G003`–`G007`, `G013`, `G014`, `G016`, widget `/MK /BG`,
plus five loose notes) had never been moved. The sweep took `open/` **48 → 4**
and `archive/` **328 → 374**.

★★★ The channel README's central invariant is *"a session lists `open/` and
nothing else — **EMPTY MEANS NOTHING IS OWED**"*. That invariant had been
**false for three days**, and `RESUME.md` was quoting the folder's file count as
though it were a work count. A session scheduling against that row would have
mis-estimated outstanding engine work by an order of magnitude.

★★★ **And it had already been fixed once, correctly, by me.**
`note_2026-09-11-the-channel-was-audited-end-to-end…` records driving `open/`
from **229 to 13** by hand, and contains the exactly-right sentence *"a move
without a row is not a close, it is a deletion that leaves a file behind."* That
pass was not wrong. It **ran as a sweep and was never adopted as a habit**, so
every topic closed in the following three days wrote its `done_*` into `open/`
and stopped there. ⇒ **Writing the lesson down inside the folder the lesson is
about did not prevent the recurrence.** A generalisation next to its own
instance is not an instrument — the same shape as
[[a-lesson-in-a-docstring-is-not-an-instrument]].

**How to apply:**

- When you write a `done_*_CONSUMED.md`, **the same command block** moves the
  request, the reply(-ies) and the done note to `archive/` under
  `YYYY-MM-DD-<topic>-{request,reply,done}.md`, and adds the `INDEX.md` row.
  Never leave the move "for later" — later is a different session that does not
  know the exchange is closed.
- ⚠ **No gate can ever catch this.** `D:\Dev\FeatureRequests\pdfce_FeatureRequests\`
  is in **no git repository**, so no CI on either side sees it. That is exactly
  why it regressed silently. The only protection is the habit.
- **A `done_*` in `open/` is the tell.** If you list the folder and see one,
  the close was claimed and not performed — check every neighbour before
  trusting any count.
- Before quoting the channel count anywhere, `grep -l 'CONSUMED\|^\*\*Status:\*\* CONSUMED'`
  it, or just read the `Status:` line of every file: `for f in *.md; do
  printf '%-80s %s\n' "$f" "$(grep -m1 -i '^\*\*Status:' "$f")"; done`. Thirty
  seconds, and it is what turned 48 into 4.
- Related: [[a-backlog-row-is-a-record-not-evidence]] — a row saying a
  topic is open is not evidence that it is.

---

## ★★★ The second lesson: **tidying an input is a change to every instrument that reads it**

The sweep was correct and it **broke two things in the same act**, both found
within the hour, both fixed the same day. Neither was visible as a change to
anything — forty-four files moved between two folders that are in **no git
repository**, so no diff, no gate and no review step on either side had
anything to look at.

**(1) It blinded a gate, which then reported OK.**
`tools/gates/check-stale-blockers.sh` exists to catch a document row that says
*BLOCKED on `request_X.md`* after the engine has shipped X. Its only evidence
that a blocker has been closed is a consumption note naming the request, and it
globbed **`open/*CONSUMED*.md`**. The sweep moved all fifty-one of those notes
to `archive/`. A `grep` over an empty file list never matches ⇒ the gate could
not go red however stale a row became ⇒ it printed
*“OK — no row declares a blocker that has been closed”* over an evidence set of
size **zero**, **in the same suite run as the sweep that emptied it**.

**(2) It killed every path-form citation of the channel in this repository.**
Measured: `grep -rho 'open/[A-Za-z0-9._-]*\.md' --include=*.md .` → eighteen
distinct, **zero still live**. Nine sat in documents making claims about today
(`OPERATOR_REQUESTS.md`, `FEATURES.md`, `GUI_ROADMAP.md`, `BENCHMARK.md`,
`ENGINE_BACKLOG.md`); the rest are in `HANDOFF*.md`, which are historical
records and were true when written.

**How to apply:**

- Before moving, renaming, pruning or archiving **any** directory another tool
  reads — the channel, a fixtures folder, `target/scratch/`, a gate baseline,
  a RAG tree — `grep -rl '<dirname>' tools/ *.md` first. The question is not
  *“is anything broken?”* but *“who reads this path?”*, and it takes seconds.
- **Fix the instrument to stop caring where the evidence lives**, rather than
  re-pointing it at the new folder. `check-stale-blockers` now reads `open/`
  **and** `archive/`, recognises a consumption note under either naming scheme
  or by its `**Status:** consumed` line, and so survives the next sweep.
- **An empty evidence set must never report PASS.** It is SKIPPED, with the
  count stated — and the count is printed on GREEN runs too, because “OK”
  otherwise means both *“I looked and found nothing”* and *“I had nothing to
  look at”*. A tally that can be zero is what makes green legible.
- **A citation's durable part is the filename, not the path.** An archived
  exchange keeps its original name in its `Originally filed as:` line and in
  the channel `INDEX.md` row, and the gate has always matched
  `request_[a-z0-9_]*\.md` rather than a path — so de-pathing a citation costs
  nothing and converts a false location into no location.
- ⚠ **Do not “correct” a historical record.** A path in `HANDOFF*.md` was true
  the day it was written; rewriting it destroys the thing the file is for.
  Same doctrine the gate applies to the word *blocked*.

Related: [[a-check-that-cannot-fail-is-not-evidence]],
[[a-skip-is-not-red-so-a-check-can-stop-running-unnoticed]],
[[an-unevidenced-excuse-is-worse-than-silence]].

---

## ★★★ The third instance, and it is the worst shape: **a text sweep does not carry binaries, and a binary is the one artefact class no gate resolves**

2026-09-15, found while repairing dangling `evidence/` citations. `git log
--diff-filter=D` says the **rename commit** — the one that made `pdfce` into
`pdfcer` across the tree — **deleted every binary under `evidence/`**: the
twelve-width Word-ribbon photo series, the app-icon review strip, the crosshair
previews, the Inkscape observation, and `crop_settings.png`. Nineteen PNGs and
the sweep traces, in a commit whose message is entirely about a string
substitution. Almost certainly a copy step that carried text and not binaries.

**Every citation to them stayed, and stayed wrong for twelve days.**

★★★ **The one that mattered was an INPUT in a directory that is otherwise all
outputs.** `evidence/` is where `ctx.out(...)` writes, where `make-icon.py`
writes its strip, where a sweep drops traces — so the whole directory reads as
disposable, and it was. But `crop_settings.png` is **read**:
`profile.rs`'s `pdfcer-legacy` region set is seven headings expressed as
*fractions of that exact 1860×1035 image*, and
`tests/pixel_oracle_against_real_evidence.rs` is the harness's own proof that
its legibility oracle works on a real antialiased screenshot rather than on
synthetic images. Its missing-file path prints a reason and returns. **Green,
for twelve days, over an oracle that could not run.**

**How to apply:**

- **Before any rename, move or bulk copy, list the binaries separately.**
  `git diff --name-status A B | grep '^D'` after the fact is the audit; running
  it *before* pushing the sweep is the check. Text diffs are reviewed by
  reading; a deleted PNG is one line nobody looks at.
- **Ask of every generated directory: does anything READ a file in here?**
  The answer is usually no, which is exactly why the one yes is invisible.
  `grep -rn '<dir>/' --include=*.rs` and then check each hit for
  `ctx.out(` / `write` — an output citation is fine, a read is load-bearing.
- **Restore rather than re-cite when the artefact is not regenerable by a
  committed tool.** The Word series has `tools/word-ribbon-study.ps1` and the
  icon strip has `make-icon.py`, so their citations were rewritten to name the
  **instrument** — an instrument survives a file sweep and tells the reader how
  to disagree with the number. `crop_settings.png` has no such tool (it is a
  crop of the *old* GUI's dialog), so it was restored from history instead.
  ⇒ **cite the instrument where one exists; keep the file only where none does.**
- Filed as `DEFECTS.md` D54, whose repair is the missing gate: resolve every
  `Calibration::Image` path declared in `profile.rs` against the working tree.

Related: [[a-skip-is-not-red-so-a-check-can-stop-running-unnoticed]],
[[a-gate-hit-inside-the-repo-is-not-a-mandate-to-sweep-outside-it]].
