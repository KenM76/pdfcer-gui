---
name: cheap-to-read-is-not-no-need-to-reread
description: A comment justifying "we don't refresh this" on cost grounds is answering the wrong question; check whether the argument is about cost or correctness.
metadata:
  type: feedback
---

**When a comment explains why something is NOT refreshed, read whether its
argument is about COST or about CORRECTNESS. "It is cheap to read" and "it does
not need re-reading" are different claims, and the first gets used to justify
the second.**

**Why:** 2026-09-07. `AnnotSelection::outline` — the box the canvas selection
outline and all nine grips are drawn from — was stale after *every* edit to an
annotation: move, resize, rotate, a typed Apply, an undo of any of them. It only
returned to the mark when the operator clicked away and clicked back. The field's
own doc comment had a section headed *"Why it needs no `resolved_for` twin"*
arguing: *"content bounds cost a `decompose_page`… an annotation's outline is its
`/Rect`, four numbers in a dictionary, so it is re-read on the frame the
selection is made and carried on the selection itself."* Every clause true. The
conclusion — that no refresh was needed — does not follow from any of them. The
fix (`SelectionState::resolve_annot`) is cheap in exactly the way the comment
claimed, which is why running it was the answer rather than invalidating harder.

3,074 unit tests and 31 gates were green while this was happening. A driven
`ui-verify` run found it in ninety seconds, and only because a *new* feature
happened to read the same field.

**How to apply:** any time you meet a cache with a written justification for not
invalidating, ask what changes the value and whether anything watches for that.
The pattern to look for is a comment that compares costs where the question was
about staleness. Related: [[a-long-green-check-can-be-aiming-at-nothing]],
[[unit-tests-that-call-the-verb-cannot-see-the-chain-in-front-of-it]].
