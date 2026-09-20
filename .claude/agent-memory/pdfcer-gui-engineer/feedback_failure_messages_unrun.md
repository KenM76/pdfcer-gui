---
name: a-checks-failure-messages-are-the-half-nothing-executes
description: A green run formats none of a check's failure sentences and a planted violation fires only the branch it aims at, so the wording of every other arm is unmeasured — the remedy is a unit test per arm, each falsified by its own plant
metadata:
  type: feedback
---

**A check's FAILURE messages are the half of it that nothing executes.** A green
run formats none of them. A gate's `--self-test` plants one violation and
therefore fires exactly one branch. Everything the other arms say — the wrong
identifier, the wrong remedy, the wrong file, a sentence that reads as *someone
made a mistake* where the truth is *this build is narrow* — is unmeasured, and
goes on being unmeasured for as long as the check stays green, which is the
whole point of the check.

**Why:** `egui-shell`'s `MergeReport` exists so that a dropped ribbon item does
not present to the operator as *the application removed a control*. Its entire
value is the sentence it produces. `Skip`'s `Display` had four arms and
`MergeReport` four accessors, and **not one unit test**, inside a crate with
633 of them. The same shape sits in every refusal type, every `SkipReason`,
every `--self-test` message and every gate's remedy banner in this tree.

**How to apply:**
- When a type's reason for existing is the sentence it emits, the sentence gets
  a test per arm — not one test that formats one arm.
- Test the **distinctions**, not the wording: that the capability-absent arm and
  the unknown-command arm cannot be confused; that only the layer-wide arm says
  a layer was dropped; that a version-mismatch sentence carries BOTH numbers,
  because one alone cannot say which end to update.
- **Falsify each test with a plant aimed at it alone**, applied to the shipped
  source, run, reverted, and the file checked byte-identical afterwards. Record
  which tests each plant reddened: a plant that reddens three tests has proved
  one of them and flattered two.
- ⚠ A comparison test between two sentences is the one most likely to be
  unfalsifiable. If the two fixtures differ in their identifiers, the strings
  differ whatever the wording says, and the test passes on a pair that has
  collapsed to identical prose. **Give the fixtures the same ids and the same
  site, so the reason is the only thing that can make them differ.** Same family
  as [[an-assertion-both-outcomes-satisfy-is-not-a-measurement-of-which-one-shipped]].
- Related: [[a-check-that-cannot-fail-is-not-evidence]],
  [[a-falsification-can-lie-in-both-directions]],
  [[one-remedy-banner-on-any-nonzero-exit-sends-the-reader-to-break-a-right-figure]]
  — the last is this same defect reaching the operator: a single banner printed
  for every failure told him to rewrite figures that were right.
