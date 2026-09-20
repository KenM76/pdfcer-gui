---
name: a-document-that-teaches-a-rule-cannot-use-a-well-formed-example-of-the-violation
description: A doc teaching a rule by showing the forbidden shape is indistinguishable from a real violation, so the gate that enforces the rule fires on its own teaching example
metadata:
  type: feedback
---

A document that **teaches** a rule by quoting the forbidden shape verbatim
writes a real instance of it. No checker can tell a counter-example from the
thing it warns about, because they are the same bytes.

**Why:** `DEVELOPING.md`'s R5 section illustrated *cite by symbol, never by
line* with a perfectly well-formed engine citation — and
`check-engine-citation.sh` flagged it on its first run over the real tree,
correctly. Three more of the same class turned up in a `.rs` doc comment and
two fixture `.PROVENANCE.py` files. The document was also the highest-traffic
place a reader would learn the shape *from*.

**How to apply:** when writing a counter-example, break the shape while
keeping the meaning — `` `edit.rs` line 4211 `` teaches exactly what the
colon-and-digits form teaches and is not a violation. Spelling the digits as
`NNNN` is the other escape, and it is the one to reach for when the shape
itself is the subject. Do this for every rule
with a machine behind it: forbidden file names, forbidden call sites,
forbidden prose. And expect the reverse too — running a new gate over the real
tree is an **instrument**, not only a guard; this one found seven violations
in files the hand sweep never considered, one of which named the wrong crate
entirely. Related: [[a-lesson-in-a-docstring-is-not-an-instrument]],
[[a-check-that-cannot-fail-is-not-evidence]].
