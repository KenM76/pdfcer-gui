---
name: a-pattern-over-a-human-document-is-a-claim-about-its-notation
description: A regex written over a hand-authored register encodes the notations you happened to look at; sample the document for the shapes it actually uses, and measure the widening's YIELD against real rows rather than a self-test.
metadata:
  type: feedback
---

**When a checker's input is a document a person wrote, the extraction rule is
the part that will be wrong — not the checker's logic.** Enumerate the
notations the document actually uses by sampling it end to end, then prove the
rule against those samples.

**Why:** building `check-backlog-verdict-drift`, the rule that pulls engine
symbols out of a register row's first cell anchored its symbol pattern at
end-of-token, because every example in front of me was a bare path
(`EditSession::run_repertoire`). `ENGINE_BACKLOG.md` also writes a verb **with
its parameter named** — `with_duplicate_keys(policy)` — and a **brace group** —
`DuplicateKeyPolicy::{KeepLast, Refuse}`. Both were silently dropped. The
checker's hit rule, its exemption model and its scope were all fine; the
blindness was one regex upstream of them, and it would have shipped a gate that
printed *clean* over the exact rows it was built to catch.

The self-test could not have found it: the plants were written by the same
person, in the same hour, in the same notation the regex already handled. What
found it was **widening the rule and then measuring the yield against the real
register** — candidate pairs went 20 → 27 and three rows appeared, one of which
(`add_reply`) nobody had triaged at all. A widening that produces no new hits is
either unnecessary or still blind; either way the number is the evidence.

**How to apply:**
- Before writing a pattern over a human-authored file, grep that file for the
  construct in two or three different spellings and see which ones exist.
  Backticked cells, table cells and prose bullets all drift in notation.
- Report the widening as a **before → after count on the real input**. "It now
  handles brace groups" is an intention; "27 pairs, up from 20, three new rows"
  is a measurement.
- Related: [[falsify-the-gate-against-the-real-files-and-the-fix-against-a-control-binary]]
  (the same rule for the checker's logic), [[a-gate-keyed-on-a-name-is-discharged-by-prose]]
  (the opposite failure — a pattern too loose for the same document).
