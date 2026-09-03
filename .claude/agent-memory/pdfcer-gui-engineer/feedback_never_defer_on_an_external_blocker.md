---
name: never-defer-on-an-external-blocker
description: "A pdfcer-core gap is not a reason to defer a GUI feature — decompose the operation into the verbs that exist and build it. Ken's directive, 2026-08-19."
metadata:
  type: feedback
---

**A missing or slow-to-arrive `pdfcer-core` verb is not an acceptable reason to
leave a GUI feature unbuilt.** Decompose the operation into the verbs that
exist and build it; file the engine request afterwards, if at all.

Ken, 2026-08-19: *"Get everything unblocked on phase 5 — no excuses about
slowness of feature from pdfcer as a reason not to implement."*

**Why:** on the day he said it, **three** features whose recorded blockers had
sat in `FEATURES.md` for weeks turned out to have no blocker at all, and all
three shipped that afternoon:

- *"`EditSession` has no scale verb"* — true, and irrelevant. `move_nodes`
  takes a list of `(node, point)` and **a scale is a list of per-node points**.
- *"No multi-run text edit"* — true, and not his bug. His bug was editing one
  run on a **shared line**, which `FollowerDisposition::Pin` had always done.
- *"Edit a Bézier handle"* — `EditSession::move_handle` had shipped in Pass
  30.1 with a planner, an enum and a disclosure contract, unnoticed.

Each blocker was produced by grepping the engine for a verb whose **name**
matched the operation. That only works when the library's vocabulary and the
GUI's concept happen to coincide.

**How to apply:**

- Before writing "blocked on `pdfcer-core`", read the **whole** verb list
  (`grep -oE "pub fn [a-z_]+" edit.rs` — 157 of them, ten seconds) **and the
  enums**. Both misses above lived in parameter types, not function names.
- Ask *what does this operation decompose into in terms of verbs that exist*,
  not *is there a verb called this*.
- If the operator has reported a bug, check that your blocker and his report
  describe the same thing. Twice they did not, and the mismatch sat unnoticed
  for weeks.
- Write every external blocker as a **dated, falsifiable citation**
  (`MarkupSpec has no Cloud variant — annot_author.rs:280, checked
  2026-08-14`), never as a verdict. A blocker naming a repo this project does
  not build cannot fail a test, so CI stays green *because the feature is
  absent*.
- A genuine blocker still exists sometimes — the object clipboard is one, and
  the correct response was to record it as a dated measurement and **not** file
  a request, because nothing was measured about how he wants paste to behave.

Full write-up:
`D:\dev\rag\rust\a_missing_verb_is_often_an_existing_verb_you_did_not_decompose_the_operation_into.md`.

Related: [[scope-a-request-to-the-whole-expected-behaviour]],
[[the-engine-session-runs-in-parallel-and-answers-within-the-hour]].
