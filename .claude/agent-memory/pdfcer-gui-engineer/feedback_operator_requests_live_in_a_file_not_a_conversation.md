---
name: operator-requests-live-in-a-file-not-a-conversation
description: Every ask Ken makes goes into D:\Dev\pdfcer-gui\OPERATOR_REQUESTS.md the moment he makes it, and only he closes a row — read it at the start of every session.
metadata:
  type: feedback
---

**Write every request Ken makes into `D:\Dev\pdfcer-gui\OPERATOR_REQUESTS.md`
the moment he makes it, before starting work on it. Read that file at the start
of every session, alongside `PROJECT_PLAN.md`. Only Ken closes a row.**

**Why:** Ken, 2026-08-20, after asking for canvas text editing and clipboard
shortcuts for at least the third time each:

> *"Where do you need to put these requests so they just get auto-repeated over
> and over again so I don't have to keep requesting they be done over and over
> again, or can you finally do them?"*

That is not impatience, it is an accurate report of a defect in how this project
tracks work. A request made in conversation lives exactly as long as the
conversation: sessions end, context is compacted, and an ask made early in a
long session is gone by the end of it. He had been carrying the backlog in his
head because nothing else was.

**How to apply:**

- **On every new ask** — add the row first, then work. A row added after the
  work is a row that only exists if the work finished.
- **A status is evidence, never a claim.** Either name the driven check, or
  write NOT VERIFIED in those words. "Done" is not a status.
- **Shipped ≠ closed.** Built, gated and driven moves a row to *Shipped —
  awaiting your verdict*. It leaves the file only when Ken has used it and said
  so. This matters because several things this project called done were broken
  on screen (see [[feedback_smoke_launch_offscreen_when_the_desktop_is_blocked]]
  and the ui-verify R1 rule).
- **Never silently rescope.** Shipping half an ask leaves the row open, saying
  which half.
- **A blocked row names the request file** in
  `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\`. "Blocked" with nothing
  behind it means the work was not done.

**★ The related judgement error to avoid.** On 2026-08-19 I wrote to the engine
session *"we are not asking for it yet"* about the page-content clipboard —
after Ken had already asked for it. **Deciding an operator's ask is not urgent
is not my call.** If he asked, it is asked. File it.

Related: [[feedback_scope_a_request_to_the_whole_expected_behaviour]] — the same
failure seen from the other end, where enumerating deferrals moves the work onto
him.
