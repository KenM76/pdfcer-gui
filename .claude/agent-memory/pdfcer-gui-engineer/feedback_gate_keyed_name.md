---
name: a-gate-keyed-on-a-name-is-discharged-by-prose
description: A coverage gate that asks "does this name appear" is satisfied by a doc comment; 25 engine verbs were scored consumed while every mention of them was a comment.
metadata:
  type: feedback
---

A completeness gate that greps for an **identifier** is satisfied by any
sentence containing that word. It measures vocabulary, not wiring. Require a
**syntactic shape that only code can have** — for a function, `name` followed
by `(` with comments stripped — and the gate starts measuring the thing it is
named for.

**Why:** 2026-09-05. `tools/verb-coverage.py` scored an engine verb "consumed"
on `\bname\b` anywhere under `crates/pdfcer-gui/src`. `pdfcer-core` shipped
`pdfcer_core::sign` — 101 public items, a whole signing subsystem, built in
answer to *this project's own* request — and the gate marked
`EditSession::sign` consumed because the word `sign` appears in a doc table
about **the arithmetic sign of `/Count`**. A capability the operator asked for
was discharged by a comment about positive and negative numbers.

Tightening it to call-shape moved **25 verbs** from "named" to "named
nowhere", and spot-checking found *every* mention of them was a comment —
including one whose comment explained why the verb was deliberately not used,
an answer living in prose instead of in the register that exists for it.

⇒ **Every verb whose name is also an ordinary English word — `sign`, `merge`,
`split`, `count`, `set`, `move`, `insert`, `close`, `open` — was permanently
and silently exempt.** The exemption correlated with nothing except vocabulary,
so no amount of review would have surfaced it.

**How to apply:** when writing or auditing any "is X used?" check, ask what
string would satisfy it that is *not code*. Doc comments in this project are
enormous by directive (R5), which makes prose-matching far more likely here
than in a normal repo — the documentation culture and the naive gate interact
badly, and that interaction is the finding. Related:
[[a-proxy-condition-survives-one-correction]],
[[a-check-that-cannot-fail-is-not-evidence]],
[[a-hand-written-list-inside-a-completeness-test-is-the-gap]].
