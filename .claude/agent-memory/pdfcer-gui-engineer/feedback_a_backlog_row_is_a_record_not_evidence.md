---
name: a-backlog-row-is-a-record-not-evidence
description: Never quote OPERATOR_REQUESTS.md, FEATURES.md or a prior row as proof of what the code does — check the source, it costs a minute
metadata:
  type: feedback
---

**A row in `OPERATOR_REQUESTS.md` (or `FEATURES.md`, or `NO_SURFACE.md`) is a
record of what was true when it was written. It is not evidence about the code
now.** Before repeating any capability claim from one — especially a claim that
something is ABSENT or BLOCKED — verify it against source with `git log -S` and
a grep.

**Why:** on 2026-08-21 three documents said *"there is no rotate handle on the
canvas"*. The grip had shipped the previous day in `560280a` — painted,
hit-tested from the same predicate as the eight resize grips, with a ghost and
15° Shift-snapping. The rows were simply never updated. I then read those rows,
trusted them, and **re-published the false claim into two brand-new rows** as
though it were current — telling Ken to his face that a feature he already had
did not exist.

That is the exact failure the backlog exists to prevent, committed inside the
backlog. It is also the fourth-through-sixth instance of a shape this project
has now spent a great deal on: `markup.cloud` ("the ONLY kind still absent for
an engine reason" — it had shipped, and he asked three times over three weeks),
`NO_SURFACE.md`'s opacity blocker, and `FEATURES.md`'s theme-preset claim.

**How to apply:** the moment a document says a capability is missing, blocked or
unbuilt, and you are about to *act* on that — schedule it, tell Ken, write it
into a new row — spend the minute:

```
grep -rn '<the thing>' crates/            # does the code exist?
git log --oneline -S '<symbol>' -- <path> # when did it arrive?
git log --oneline --diff-filter=A -- <file>
```

**An absence claim is the dangerous direction**, because nothing fails when it
is wrong — the feature simply sits there unused while everyone plans to build
it. A presence claim gets caught the first time somebody tries it.

Corollary that applies just as hard: when you FIX something, sweep for every
document that describes it. The fix and the record are one change, not two.

Related: [[never-defer-on-an-external-blocker]] — same instinct, one step
earlier. A "blocker" and an "absence" are both claims about the world that
deserve a minute of checking before they shape a plan.
