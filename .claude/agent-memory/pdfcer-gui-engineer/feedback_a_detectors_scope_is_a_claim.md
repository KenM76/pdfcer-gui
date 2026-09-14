---
name: a-detectors-scope-is-a-claim
description: One gate reported clean over three different blind spots — 77% of the tree, 16% of files, 42% of titles — each time in the words a real all-clear uses; widen the input set, then falsify against the newly admitted shape
metadata:
  type: feedback
---

**A detector has two independent claims in it: *what it looks at* and *what it
looks for*. Only the second one ever gets tested. "No violations found" is
evidence about the second and says nothing at all about the first.**

**Why:** `tools/gates/check-orphan-docs.py` has now reported `clean` over three
separate blind spots, one after another, and each time in exactly the words a
real all-clear uses:

| date | the scope it did not have | how much it could not see |
|---|---|---|
| 2026-09-12 | scanned `crates/` only | **77%** of the tree's `.rs` files |
| 2026-09-12 | read files as LF | **16%** of files are CRLF |
| 2026-09-13 | title regex required `**` immediately after `/// ` | **893 titles, 42%** of the convention |

The third one is the cleanest illustration. The gate exists to police doc-comment
titles. This crate's titles may open with a decoration run — `★`, `★★★`, `⚠`,
`→`, `⇒` — and every decorated title was invisible **to the instrument built to
police titles**. Widening the regex to `(?:[★⚠→⇒]+ )?` took the title count from
1,223 to 2,116 and the reported seams from 1 to 8. Seven real defects had been
sitting behind the scope, in a tree the gate called clean, for as long as the
gate had existed.

The failure mode is worse than a red gate that gets ignored, because a green
gate over a shape it cannot express reads as *positive evidence*. Every "clean"
it printed made the next person less likely to look.

**How to apply:**

- When you touch or trust a detector, read **what it globs, what it opens, and
  what its regex requires** before reading what it asserts. Write down the input
  set as a number: files scanned, matches found. A gate that prints its own
  census (`titles matched: 2,116`) can be seen to go blind; one that prints only
  `clean` cannot.
- ★★★ **Widening without falsifying is documentation, not reach.** After
  admitting a new shape, plant that exact shape and confirm rc=1 — twice: once
  in the self-test (cheap, proves the regex) and once in a real source file run
  as a subprocess (proves the call site). And if the widened thing is a *set*,
  exercise **each member**: a test that plants a `★` orphan measures `★`, not
  `DECOR`.
- The narrow-regex leg is done by **overriding the constant on the imported
  module** from the falsification script, so nothing on disk changes and there
  is no restore to get wrong.
- Suspect the scope first whenever a long-green instrument suddenly finds a lot.
  The finding is usually not "seven defects landed"; it is "seven defects were
  always there and the scope just moved".

Siblings: [[a-check-that-cannot-fail-is-not-evidence]] (an assertion that
cannot go red), [[a-long-green-check-can-be-aiming-at-nothing]] (the right
assertion pointed at the wrong surface), [[a-rename-can-blind-an-instrument-silently]]
(a scope that *was* right and got rewritten),
[[a-gate-whose-input-set-comes-from-git-measures-the-index]] (a scope that
changes under you), and
[[a-hand-written-list-inside-a-completeness-test-is-the-gap]] (a scope
enumerated by hand). And the two-leg discipline above is the same one as
[[falsify-the-gate-against-the-real-files-and-the-fix-against-a-control-binary]].
