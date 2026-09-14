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

## The 2026-09-14 sweep: half of what it called *owed* was already closed

The same folder, the other direction. A pass took `open/` from **48 files to
4** and wrote each survivor's reason into `INDEX.md`. **Two of the four were
not owed at all**, and the interesting part is that neither was findable by
reading it — both honestly describe unfinished business, which is the only
question a sweep asks.

- ★★★ **A superseded document cannot be identified from its own contents.**
  A 2026-09-04 reply posed a design fork and closed *“Tell me which, and I'll
  build it next”*. It had been answered, shipped and consumed **the same
  day**, and both arms of the fork exist today. The only thing that falsifies
  it is a **LATER** document in `archive/` — and the convention tells a
  session to list `open/` **and nothing else**. ⇒ The fix is at the writing
  end, not the reading end: **when a second reply supersedes a first, file
  BOTH with the request and band the superseded one.**
- ★★★ **A misfiled answer is indistinguishable from an absent one, and a
  name asserting the wrong DIRECTION hides a document more thoroughly than a
  wrong topic would.** An eighteen-day-old question about `operator_span`
  contiguity had its answer in `archive/` from the same day — filed as
  `…-CONSUMED.md`, the suffix this channel uses for *the GUI wired it*. The
  searcher greps for a `reply_`, finds none, and stops. A wrong **topic**
  still surfaces in a grep for the words; a wrong **direction** ends the
  search. Renamed and banded, because leaving it leaves the trap set.
- ⇒ **The answerable question is not “is this file stale?” but “has the thing
  it asks about been answered ANYWHERE?”** — and *anywhere* includes **both
  git repositories**. The second one's measurement had been quoted in a
  `pdfcer-gui` doc comment since 2026-08-28: one `git grep` would have closed
  it, and the channel was never the cheapest place to look.
- ★ **An enumeration is a stronger claim than a count, it decays faster, and
  it is the only reason either was caught.** Naming the four is what let two
  be checked against `archive/` and fail; a bare *“4 open”* is unfalsifiable.
  **Date the enumeration — do not replace it with a number.**
- ⚠ The `RESUME.md` row carrying that count was **wrong four times in one
  day, in four different ways**, each correction written by the session that
  had just fixed the previous one. A count of a folder is a proxy for the
  quantity a reader wants, which is **how much is owed**, and the two diverge
  in silence because nothing in a file's contents says whether it is still
  owed.

Related: [[a-backlog-row-is-a-record-not-evidence]],
[[a-limitation-sentence-is-a-citation-with-an-hours-long-shelf-life]],
[[an-absence-claim-is-a-claim-about-every-route]],
[[the-engine-session-runs-in-parallel-and-answers-within-the-hour]].
