---
name: a-gate-whose-evidence-you-write-is-blind-to-what-you-forget-to-write
description: check-stale-blockers keys on a CONSUMED note this side authors, so wiring a capability without filing the note leaves every stale row about it unguarded — and a corrected row still contains the word it was corrected about.
metadata:
  type: feedback
---

**A gate whose evidence is an artifact you author is blind by construction to
the artifact you forget to author.** Filing the record is not paperwork; it is
the act that arms the check.

**Why:** 2026-09-09. `check-stale-blockers.sh` fires when a row says `BLOCKED`
and names a request the engine has **CONSUMED**. The predicate is deliberately
`*CONSUMED*.md` in `open/` — written by *this* side — and that is **correct**:
a reply that schedules, defers, refuses or merely explains must not clear a
blocker, and only this side knows when a capability has actually been taken.
The gate's own header records the first version firing on a TRUE warning and
warns it "would have had somebody delete a true warning to make a build go
green".

The consequence nobody had drawn: four markup asks were answered by the engine
**the same day they were filed**, wired here within a day, and **no CONSUMED
note was ever written**. So three `BLOCKED` rows stood for three days with the
gate green — and even after `ENGINE_BACKLOG.md` was added to its scope (see
[[a-hand-written-list-inside-a-completeness-test-is-the-gap]]) it would *still*
have passed, because the evidence did not exist. Two independent causes, either
one sufficient. The gate was not broken; it was **uninformed**, and it had no
way to tell the difference.

**How to apply:** when a capability lands, write the CONSUMED note in the same
sitting as the wiring — before the commit, not "later". Then **falsify**: put
the stale document back from a copy, confirm the gate goes RED naming the right
rows, restore. That run is the only proof the note actually armed anything.

## ★★ The other half: a corrected row still contains the word it was corrected about

Re-running the fixed gate on the **fixed** files reported two more hits, and
both were **correct** rows narrating their own history — a ✅ row saying *"THE
⛔ ON THIS ROW WAS STALE AND IS CORRECTED"*, and a **Reachable** row saying
*"this row was `blocked` for about six hours"*. Both name the request file,
because this project requires a correction to keep its citation so it can be
audited.

⇒ **A false positive on a correctly-updated row is the same hazard as a false
positive on a true warning, pointing the other way**: the cheapest way to clear
it is to delete the history sentence, and the history sentence is the most
valuable part of a corrected row.

The fix that generalises: **a row's verdict lives at the START of a cell.** A
cell-initial closure token (`✅`, `**Reachable`, `**Consumed`, `**WIRED`,
`**UNBLOCKED` — measured from the documents, not invented) settles the row for
today, so a blocked token elsewhere on the line is narration. And the excused
lines are **counted and printed on every run, green ones included** — an
exclusion nobody can see is how a gate quietly narrows its own scope. See
[[a-skip-is-not-red-so-a-check-can-stop-running-unnoticed]] and
[[a-check-that-cannot-fail-is-not-evidence]].
