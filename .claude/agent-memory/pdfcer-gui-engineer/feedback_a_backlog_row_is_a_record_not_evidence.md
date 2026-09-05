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

## ★★★ "THERE IS NO ART FOR IT" IS NEVER A VALID REASON TO SHIP A CONTROL WITHOUT A GLYPH — 2026-09-04, and this is the SECOND time in one day

Ken's standing ruling, 2026-08-06, already recorded in `Icon::Back`'s own doc
comment: **a missing glyph is AUTHORED, not worked around.** He set it after a
control was reworded to avoid a tofu character, and the note says *"rewording
spends the operator's affordance to protect the font stack; an icon costs one
asset and keeps both."*

Twice on 2026-09-04 a build session's refusal quietly overrode it:

1. `edit.select_all` — refused *"no conventional icon … a marquee would say
   rubber band"*. He answered: ***"add a select-all glyph. I didn't refuse
   that."***
2. `format.bold` / `format.italic` — refused *"Word draws `B` and `I` as glyphs;
   this build has no such art."* He answered: ***"why weren't they made
   automatically as I have instructed to be done for anything that a glyph is
   missing for on multiple occasions?"***

⇒ **The reasons are not interchangeable, and only one kind survives him:**

| refusal reason | verdict |
|---|---|
| *"no art exists in the set"* | ★ **INVALID. Always.** It is a supply statement, and his ruling is that supply is answered by drawing. |
| *"no icon SLOT exists on this surface"* (a custom widget, a menu row of words, the mode selector) | valid — structural, nothing to draw into |
| *"the icon ui-spec argues against one BY NAME"* | valid — a design position with a citation |
| *"any drawing would make a claim the command cannot support"* | valid — the `Signatures` / `Fonts` shape |

**How to apply:** when auditing icon coverage, grep the refusals for the phrase
*"no such art"*, *"no glyph exists"*, *"nothing in the set"* — every one of those
is a defect, not a decision. Draw it.

★★ And the meta-lesson, which is the same as [[feedback_a_backlog_row_is_a_record_not_evidence]]
one level up: **a refusal written by whoever happened to be building that day is
not an operator decision**, and quoting it four times does not promote it. Ask
who said it. He has now had to say so twice in one afternoon.
