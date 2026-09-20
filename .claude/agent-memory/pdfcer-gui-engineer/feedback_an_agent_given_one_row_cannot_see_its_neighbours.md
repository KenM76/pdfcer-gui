---
name: an-agent-given-one-row-cannot-see-that-its-neighbours-already-decided-the-question
description: A subagent handed a single row of a register will re-open a question two other rows already settled; verify a triage verdict against the row's SIBLINGS before applying it.
metadata:
  type: feedback
---

**A per-row triage dispatch is blind to the rest of the file.** Before applying
any subagent's verdict on one row, read the rows around it — especially the
`declined` section, which is where a question goes once it has been answered
*no*.

**Why:** triaging `ENGINE_BACKLOG.md`, an agent given the `LoadOptions` row
proposed narrowing it to *"wanted on three of its four axes"*. Two rows further
down the same file already **declined** exactly those three: a strict open
(`strict()`, `UnreadableObjectPolicy`) on the grounds that *fail-clean never
meant refuse*, and recovered stream extents (`stream_lengths`, `terminators`) on
the engine's own finding that there is nothing there to choose between. The
agent's reasoning about its own row was sound and its conclusion was wrong,
because the evidence that settled it was in a section it never saw.

Applying that verdict would have moved a settled decision back into the
schedule — a register wrong in the *shipped* direction, which is the failure
mode the file exists to prevent.

**How to apply:**
- Fan out per-row triage freely — it is the right shape for the work — but
  treat each verdict as a **finding to check**, not a decision to apply.
- The cheap check is one grep of the whole register for the row's symbols before
  editing it. A symbol appearing in two sections is the tell.
- When the dispatch is worth doing properly, give each agent the `declined`
  section verbatim in its prompt. It is short and it is exactly the context the
  per-row framing removes.
- Related: [[a-backlog-row-is-a-record-not-evidence]],
  [[an-apparent-omission-may-be-an-argued-decision]] (same shape, one level up:
  open the module that owns it).
