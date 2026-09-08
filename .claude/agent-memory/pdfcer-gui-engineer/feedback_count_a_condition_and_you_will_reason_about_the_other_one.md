---
name: count-a-condition-and-you-will-reason-about-the-other-one
description: Two wrong diagnoses of one report in a day, both from counting one half of an AND and inferring the other; the fix is to measure the OUTCOME, and even then check the harness's own input first
metadata:
  type: feedback
---

When a refusal fires on a conjunction — *"this run repeats on the page **and**
spans several show operators"* — **count the intersection, never one side.** If
you cannot count it cheaply, perform the operation and read the answer.

**Why:** on 2026-09-08 Ken reported that a BOM "only sometimes works". I
diagnosed it twice and both were wrong, in the same shape each time:

1. Counted repeated runs (122 on the sheet), reasoned about the pin ⇒ *"the
   planner throws the pin away"*. It does not; it keeps it deliberately.
2. Counted repeated runs again, reasoned about the operator split ⇒ *"a
   repeated cell hits the ambiguity refusal"*. The intersection of the two
   conditions across all four of his sheets is **zero**. 57–133 repeating runs,
   4–11 multi-operator runs, and they never overlap.

The real cause was found by performing 31 real edits, one fresh `EditSession`
each, and reading 31 real answers. It was the **font**: his drawing's subset
fonts accept 46 of 95 printable ASCII characters and **no lowercase at all**.
The pattern was in what he typed, not where he clicked.

★★★ **And the third attempt was ALSO wrong on its first run**, in the way that
mattered most: the probe appended a `Z`, so 30 of 31 cells refused with
`UnsupportedFont`. That looked like a finding about the cells and was a
statement about the letter `Z`. **The harness artefact was the answer** — but
only because I asked why every single cell refused instead of writing it up.

**How to apply:**

- A refusal guarded by `A && B`: measure `A && B`. A count of `A` alone licenses
  nothing, and it is *seductive* because it is the cheap one.
- Prefer performing the operation to predicting it. One `Document::load` per
  trial is a minute of compute against a day of wrong diagnoses.
- **A fresh session per trial is not an optimisation to remove.** A shared one
  measures the Nth edit on a document already edited N−1 times.
- **Build test inputs from the subject's own alphabet.** A replacement string
  containing a character the document has never used measures your string.
- A uniform failure across every trial is about the probe. (This is the sibling
  of `feedback_a_uniform_failure_at_every_rung_of_a_sweep_is_about_the_probe`
  and of `feedback_a_harness_with_a_bad_input_produces_defects_that_do_not_exist`
  — three receipts now, which is why this one is written as a positive
  procedure rather than another warning.)
- **Correct the artifacts you already shipped.** Both wrong diagnoses had
  reached a commit message, an `OPERATOR_REQUESTS.md` row *and* an engine
  request with a fabricated motivating measurement. All three were amended the
  same day. See [[feedback_a_limitation_sentence_is_a_citation_with_an_hours_long_shelf_life]]
  and [[feedback_when_two_things_differ_in_two_ways_the_measured_one_is_not_the_cause]].
