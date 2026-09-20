---
name: a-doc-comment-that-argues-for-a-guard-is-a-claim-the-guard-exists
description: Deleting a workaround leaves its justification behind — prose that argues at length for a mechanism reads as proof the mechanism is there, and it outlives the code by days because nothing in the toolchain connects the two
metadata:
  type: feedback
---

**When you delete a guard, grep for everything that argued FOR it — and delete
or re-date that too, in the same commit.** A paragraph defending a mechanism is
an assertion that the mechanism exists.

**Why:** 2026-09-14, correcting seven reflow doc sites at once. On 2026-09-05
the shell's over-broad `edit_epoch != 0` pre-flight gate was deleted, correctly,
because engine `Pass 251.0` had started refusing the real case by name. The
deletion was well documented *at the deletion site*. But:

- `ReflowRefusal::PageAlreadyEdited`'s variant doc still opened **"This gate is
  load-bearing and it is NOT merely conservative"** and closed **"this forecast
  is the only thing standing between the operator and losing work he can see on
  the page."** Nine days after the forecast was deleted.
- The function body still carried the gate's *preamble* — *"Asked BEFORE the
  attempt, so the operator is told the remedy rather than shown a silence"* —
  sitting immediately above the comment block announcing the gate's deletion.
- `EngineDeclined`'s doc still said the engine packs *"ten distinct refusals in
  one variant with no discriminant"* and **promised a future** in which the
  recoverable one gets its own variant. Two discriminants had landed. The
  forecast was right and nothing noticed it had come true.

★★ The second and third are the interesting shapes. Prose can be stale by
**describing the other crate's shape** — which changes without your compiler
saying anything — and it can be stale by **asking for something that then
arrived**. A doc comment containing the word *"when X lands"* is a tripwire
nobody armed.

**How to apply:**
- `grep` the variant/constant/field NAME across the crate when its mechanism
  changes, not just the file you edited. Seven sites, five files, one cause.
- Suspect any doc paragraph that **argues** rather than describes. Argument is
  what survives deletion, because whoever removes the code removes the code.
- A body comment that explains a check must be deleted **with** the check.
  A preamble orphaned above its own obituary is worse than either alone.
- Prefer "Corrected YYYY-MM-DD: this said X, which `Pass N` made false on
  YYYY-MM-DD" over a silent rewrite — the mistake is usually worth more than
  the correction, and it is the only thing that tells the next reader this file
  has a drift habit.
- Same family as [[delete-the-workaround-when-the-cause-is-removed]]
  (which is about the code) and
  [[a-limitation-sentence-is-a-citation-with-an-hours-long-shelf-life]]
  (which is about claims). This one is the middle case: the code went, the
  argument stayed.
