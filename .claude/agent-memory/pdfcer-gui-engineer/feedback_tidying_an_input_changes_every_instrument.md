---
name: tidying-an-input-changes-every-instrument
description: Two lessons from one afternoon — closing an exchange is part of doing the work, not a later sweep (open/ went 229→13 then back to 48 in three days); and the sweep that fixed it blinded a gate and killed 18 citations, because tidying an input is a change to every instrument that reads it
metadata:
  type: feedback
---

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
[[feedback_a_lesson_in_a_docstring_is_not_an_instrument]].

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
- Related: [[feedback_a_backlog_row_is_a_record_not_evidence]] — a row saying a
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

Related: [[feedback_a_check_that_cannot_fail_is_not_evidence]],
[[feedback_a_skip_is_not_red_so_a_check_can_stop_running_unnoticed]],
[[feedback_an_unevidenced_excuse_is_worse_than_silence]].
