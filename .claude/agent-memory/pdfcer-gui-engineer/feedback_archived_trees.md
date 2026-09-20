---
name: an-archived-trees-citations-froze-wrong
description: A citation into a tree that stopped changing is not frozen-correct — it froze in whatever wrong state it was already in, and seven of eleven were off by +2 to +29 lines
metadata:
  type: feedback
---

A tree that no longer changes does **not** make line citations into it safe.
The citations were written against an *in-flight* tree and kept drifting until
the freeze — so what froze was the error. Measure before trusting one.

**Why:** a commit in this repo recorded the opposite as settled — citations
into the archived `D:\Dev\pdfce\crates\pdfce-gui` "cannot rot at all… frozen
evidence" — and
the next sitting measured eleven of them against the archive. Seven were wrong
by +2 to +29 lines. Only two landed exactly. Every wrong one still landed
inside readable prose, because upstream *insertion* shifts a whole file in one
direction rather than scrambling it, so the number keeps pointing at real text
about a real function that has nothing to do with the claim.

**How to apply:** "the tree is archived" is an argument about the *future*, not
the past, and it never discharges a citation. Anchor by symbol regardless of
whether the target still moves. When you inherit a doc or commit asserting a
class of citation is safe, that assertion is a hypothesis with a cheap
measurement attached — run it. See [[a-measured-limit-belongs-to-a-revision-not-a-design]]
and [[an-injected-file-is-a-dated-snapshot]].
