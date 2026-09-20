---
name: a-register-row-outranks-memory-so-correcting-the-row-is-the-work
description: A cold session reads RESUME.md before memory and nothing reconciles the two, so a wrong cause recorded in the register beats a correct diagnosis sitting in memory — and a row citing its own prior existence is a checkable claim that was false in the commit that wrote it.
metadata:
  type: feedback
---

**When a memory entry contradicts a row in the project register — `RESUME.md`,
`HANDOFF.md`, `SWEEP_REPAIRS.md`, `OPERATOR_REQUESTS.md` — correcting the row
IS the work. Writing the memory is not enough and never was.**

**Why:** measured 2026-09-13, at a cost of a day and a misdirected priority.

The correct diagnosis of a SKIP — *the check's own setup deletes the file
holding the O173 suppression, so a real OS window takes its click* — was written
into [[a-fix-that-names-its-victims-can-still-miss-one]] on 2026-09-12. On
2026-09-13, **five places in three project documents** still recorded the cause
as *"the File-tab route"*, a suite-wide ribbon blocker, and `RESUME.md` listed
fixing that route as the next thing to do. The ribbon was never involved:
`ribbon/tabs.rs` emits `ribbon-tab-activated` for every tab unconditionally.

**The mechanism is structural, not carelessness.** A cold session's reading
order is `RESUME.md` first, then the standing docs, and memory is consulted *when
something seems relevant*. A register row that names a cause does not seem to
need checking — it reads as already-measured. Nothing in the process reconciles
the register against memory, and the register is what sets the next session's
priorities. **So memory loses every time**, and the more confident the memory
entry, the wider the gap it leaves open.

**★★ Second finding from the same correction: a clause that cites its own prior
existence is a checkable claim.** What promoted one sighting to *suite-wide* was
the phrase *"already on the housekeeping list from an earlier sweep"*.
`git log --oneline -S "File-tab route" --all -- '*.md'` names **exactly one
commit**, which introduced all five mentions of the phrase in the same change.
The earlier record never existed. An unverifiable appeal to the register's own
history set a whole session's priorities.

**How to apply:**
- Writing a memory entry that names a cause? **In the same breath, grep the
  register for the wrong cause and correct it.** `grep -rn "<the phrase>" *.md`
  costs seconds. The memory is for the *lesson*; the register is for the *state*,
  and the state is what gets acted on.
- Any row saying *"already known"*, *"from an earlier sweep"*, *"as noted
  previously"*, *"long-standing"* — run `git log -S` on the distinctive phrase
  before believing it. Self-citation is the cheapest way for a guess to acquire
  the authority of a measurement.
- Correct a wrong **cause** in place, struck through, rather than deleting the
  row — deleting removes the only place the wrong attribution can be found while
  leaving the documents that copied it repeating it. Delete only a row whose
  cause was right and whose repair landed. (This overrides
  `SWEEP_REPAIRS.md`'s own rule 1, deliberately, and the file now says so.)
- Siblings: [[a-backlog-row-is-a-record-not-evidence]],
  [[an-absence-reported-by-a-check-is-usually-the-panel-going-quiet]],
  [[an-unevidenced-excuse-is-worse-than-silence]],
  [[write-the-row-when-he-speaks-not-when-the-work-lands]].
