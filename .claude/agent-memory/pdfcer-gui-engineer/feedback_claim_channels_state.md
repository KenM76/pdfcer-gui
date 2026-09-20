---
name: a-claim-about-the-channels-state-is-measured-in-the-channel
description: I told the engine "we believe E001 can be closed" — we had closed it ourselves three days earlier, with a done file in the archive and a CONSUMED row in INDEX.md.
metadata:
  type: feedback
---

When a claim is about the **request channel's own state** — open, closed,
filed, owed — measure it in `D:\Dev\FeatureRequests\pdfce_FeatureRequests\`,
never in the document that cites the request.

A note going out to the engine verified seven claims about the shell's source,
file by file and symbol by symbol, and then carried an eighth sentence — *"we
believe `E001` can be closed"* — taken straight off the engine's own row text
(`gui [ ] — not wired (E001 filed)`). `E001` had been closed **by us** three
days earlier: a `done` file in `archive/` and a CONSUMED row in `INDEX.md`, both
dated. The engine had full notice. The row was simply stale.

**Why:** the seven verified claims were about a tree I habitually measure; the
eighth was about a tree I read as prose. The citing document is the *subject* of
the claim, so it is the one source that cannot settle it — and being wrong in
that direction reads as us not knowing our own channel, which discounts the six
correct claims sitting beside it.

**How to apply:** before writing *"X can be closed"*, *"X is still open"* or
*"we filed X"*, grep `INDEX.md` and `archive/` for the identifier. The stronger
finding is usually on the other side of that grep: here the ask changed from
*please close this* to *your row cites a request that closed three days ago and
your own index says so*. Related:
[[triage-the-reply-channel-the-engine-fixes-faster-than-we-notice]],
[[a-backlog-row-is-a-record-not-evidence]],
[[an-unevidenced-excuse-is-worse-than-silence]].
