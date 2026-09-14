---
name: a-count-command-can-be-wrong-not-just-its-quoted-answer
description: RESUME.md's own state table carried a check-count command that measured the wrong thing (434 vs 213) — re-measuring faithfully reproduced the wrong answer for months
metadata:
  type: feedback
---

**Re-running the command in a state table is not enough — audit what the
command counts.** `ui-verify --list | wc -l` had sat in `RESUME.md`'s "State,
as measured — re-measure, never quote" table for weeks. `--list` prints **two**
lines per check plus a header, so the command answers **434** where the answer
is **213**. The quoted value had drifted to 212 and nobody noticed, because
every re-measurement faithfully reproduced a number nobody compared against
reality.

**Why:** this project has been corrected on count drift eight times, and the
discipline that grew out of it — *never quote, always re-measure* — assumes the
command is a correct instrument. It defends against a stale number and is
completely blind to a wrong instrument. The header warning about drift sat four
lines above the drifting row. Fixed by counting check NAMES:
`ui-verify --list | grep -cE '^  [a-z0-9_]+$'`.

**How to apply:** the first time you run any command in a measured table, spend
one line checking its OUTPUT SHAPE — `head -6` before `wc -l`. Ask what one
unit of the thing being counted looks like in that output. A `wc -l` over
anything with a header, a blank line, or a multi-line record is a claim about
formatting, not about the subject. Same family as
[[a-proxy-condition-survives-one-correction]] and
[[a-check-that-cannot-fail-is-not-evidence]]: ask what the mechanism READS,
not whether it ran.

---

**Second instance, 2026-09-13, and it is the REPAIR that was mis-transcribed.**
The corrected command is `grep -cE '^  [a-z0-9_]+$'`. Written without the
trailing `$` it answers **224** where the answer is **222**, because `--list`
prints a *second* small table -- the `--exe` targets -- whose rows are also
two-space-indented lowercase words, so `pdfcer-gui` and `pdfcer-legacy` are
counted as checks. Three different numbers were in circulation (221 carried in
the table, 222 true, 224 from the loose pattern) and the instinct was to pick
one. **Diagnose instead: `--list | sed -n '1,12p'` and look for a second
table.**

* The general form: **a repair to a measurement command has an anchor, and the
anchor is the whole repair.** An end-anchor, a `--include`, a `-w`, a
`':!docs'` pathspec -- each is the part that makes the command mean what the
prose beside it says, and each is the part a re-typing drops. If a count comes
out slightly high, look for a second section of the same output rather than for
a new item.

---

**Third instance, 2026-09-14, and the mechanism is neither a header nor a
second table — it is the SHELL.** Counting this repository's Rust with
`git ls-files '*.rs' | xargs wc -l | tail -1` answers **239,849**. The answer
is **594,445**. `xargs` splits a 1,032-path argument list into several `wc`
invocations to stay under the Windows command-line limit, each one prints its
own `total`, and `tail -1` reads **the last batch only** — about 40% of the
truth, in exactly the shape an answer comes in. There is no warning, no partial
output, nothing on stderr; `wc` did its job perfectly, several times.

It was caught only because the figure contradicted a stale row in the same
document that said 576,229, and a *smaller* number where growth was expected is
a contradiction. Had the drift gone the other way it would have been quoted.

* **The general form, and it is a third distinct mechanism:** the first two
  instances were about what the command's INPUT looks like (two lines per
  record; a second table). This one is about what the shell does to the
  command BETWEEN the pipe stages. ⇒ **Any pipeline containing `xargs` and an
  aggregate is suspect** — `wc`, `sort -u | head`, `grep -c`, `du -c` — because
  `xargs` is licensed to run the command more than once and every aggregate
  then aggregates a fragment.
* **The fix is not a bigger `-n`**; it is to stop aggregating in the shell.
  A Python loop that opens each path and counts `b'\n'` cannot be batched, and
  it is four lines.
