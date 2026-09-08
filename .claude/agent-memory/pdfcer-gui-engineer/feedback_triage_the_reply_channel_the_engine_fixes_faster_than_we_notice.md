---
name: triage-the-reply-channel-the-engine-fixes-faster-than-we-notice
description: The engine answers within hours, so our own docs go stale within hours — read the unanswered replies periodically, because no gate can find this class
metadata:
  type: feedback
---

**Periodically read every `reply_*` in the request channel and check whether the
shell consumed it — and whether any shell document still asserts the absence it
closed.** Delegate it; it is three parallel readers' work.

**Why:** 2026-09-07. O142 turned out to have been fixed for two days. That
triggered a hand triage of all 41 replies, and it was **a class, not an
incident** — five documents and three operator-facing strings were asserting
absences the engine had already closed:

- `MANUAL.md` told Ken deleting pages from a SolidWorks set might refuse to save
  (fixed 2 days earlier — his primary workflow).
- A panel row said *"pdfcer cannot sign a document"* three days after signing
  shipped, two clicks from the Sign command.
- Every failed reflow blamed page reordering the engine could no longer report.
- `EncryptError::RedactionPending` was unmatched, putting two internal Rust
  function names into a dialog.
- Widget `/MK /BG` unconsumed, with two source comments calling it an
  architectural boundary.

★ The engine's own commit that day said the `/MK` colours *"had read and write
on opposite keys"* — so **a capability we had not consumed was also broken on
their side**, and neither knew. An unconsumed capability is untested upstream
too.

**How to apply:**
- **No gate can find this.** `check-stale-blockers` detects *"we wired it and
  forgot the row"*; it structurally cannot detect *"the engine fixed it and we
  never noticed"* — the rows say `⬜` not `⛔`, and the requests were never
  consumed, which *is* the defect. And no gate can be built: `open/`'s
  `reply_*` and `done_*CONSUMED*` names share **no key**. A note proposing a
  shared topic key was sent 2026-09-07; the engine invited it and has the same
  problem.
- Ask each reader for three things per reply: what shipped (name the API),
  whether the shell consumed it (**greps that came back empty**, not
  impressions), and **any `file:line` still asserting the closed absence** —
  the third is the highest-value output.
- Correct by **quoting the original and dating the supersession**, never by
  deleting: the absence is what the engine request was argued from.

Related: [[a-backlog-row-is-a-record-not-evidence]],
[[a-limitation-sentence-is-a-citation-with-an-hours-long-shelf-life]],
[[an-absence-claim-is-a-claim-about-every-route]],
[[the-engine-session-runs-in-parallel-and-answers-within-the-hour]].
