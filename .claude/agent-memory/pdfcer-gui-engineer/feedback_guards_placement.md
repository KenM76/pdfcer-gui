---
name: a-guards-placement-decides-which-surface-must-explain-it
description: Reading a predicate's body tells you what it rejects; only its CALL SITES tell you which of our surfaces already greys out, and getting that backwards falsified three of our own documents at once.
metadata:
  type: feedback
---

**Reading the predicate is not reading the guard.** A predicate's *behaviour*
is in its body. *Which of our surfaces has to explain it* is in its call site,
and nothing about the body tells you that.

**Why:** 2026-09-12, wiring the engine's new `validate_partial_name` into the
tab-order Register box. Three of this project's own documents — a
reachability table, a test's doc comment, and the third section of a request
already sent to the engine — all said that box had **no gate**. It had been
gated since the day the engine shipped `reject_dotted_partial`, because that
predicate sits inside `adopt_plan`, `adopt_preview` **is** `adopt_plan` with
the writes dropped, and the register row calls `adopt_preview` every frame and
`add_enabled(false, ..)`s on the refusal. The button had been greying for
days. I had read the predicate, confirmed what it rejects, and concluded
nothing here called it — because I never walked up to the callers.

⇒ The second, nastier half: **a predicate moving between two private functions
inside the engine is invisible from here while silently changing which of our
sentences is owed.** No signature changes, no compile error, no gate. Same
shape as `FormLeaf::is_editable`, which changed *meaning* not signature and
left a correct-sounding paragraph lying about for ten days.

**How to apply:** before writing "surface X has no gate for Y", grep for every
caller of the predicate and every caller of *those*, up to something this
shell actually invokes. A greyed control found that way is evidence; a
predicate read in isolation is not. And when a document asserts an **absence**
of a guard, re-derive it rather than citing the document — see
[[an-absence-claim-is-a-claim-about-every-route]] and
[[a-backlog-row-is-a-record-not-evidence]]. Related:
[[a-substring-match-on-another-crates-prose-survives-a-narrowing]].
