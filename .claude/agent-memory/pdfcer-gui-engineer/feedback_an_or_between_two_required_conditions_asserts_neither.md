---
name: an-or-between-two-required-conditions-asserts-neither
description: A disjunction that looks like tolerance of a rewording is a hole; found only by planting a defect, not by reading the assertion
metadata:
  type: feedback
---

When an assertion checks `a || b` and **both** `a` and `b` are actually
required, it asserts neither. Split it into two assertions with distinct
failure messages.

**Why:** on 2026-09-08 a wording test read
`said.contains("two different things") || said.contains("same everywhere")`.
Planting a defect that deleted the first phrase from the sentence left the test
**green** on the second. The two were not alternative spellings of one fact —
the first said *why pdfcer acted*, the second said *what the document is like
now* — and a disclosure carrying only the second reads as pdfcer discarding
something of the operator's on a whim.

The `||` was written as tolerance of a future rewording. That intention is
reasonable and the code that expressed it was not.

**How to apply:**

- Ask of every `||` in an assertion: *if I delete one side from the subject,
  should this test go red?* If yes, it is two assertions.
- Genuine tolerance of a rewording is a **normalisation** (lowercase, strip
  punctuation) or a regex — not a disjunction over two different claims.
- Same shape as a check whose two arms can stand in for each other, and as
  [[feedback_a_check_that_cannot_fail_is_not_evidence]]. The family rule: a
  condition that another condition can satisfy on its behalf is not being
  tested.
- **It was found by falsifying, not by reading.** Plant the defect for each
  clause separately; a single falsification of a compound condition proves only
  that *something* in it works.
