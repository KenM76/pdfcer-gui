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
