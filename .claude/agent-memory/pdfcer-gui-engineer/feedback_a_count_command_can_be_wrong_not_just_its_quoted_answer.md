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
