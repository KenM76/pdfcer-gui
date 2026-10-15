---
name: new-variant-orphans-matchers
description: Widening a click to a new enum variant (Anchor::Block) silently broke Reflow, which let-else matched only Anchor::Run; its check SKIPped under --no-input so nobody saw.
metadata:
  type: feedback
---

When a gesture starts producing a new variant of a state enum, every consumer that
`let … else`-matches the OLD variant turns into a refusal, and compiles clean
(no exhaustiveness error on a let-else). Paragraph widening made clicks yield
`Anchor::Block`; `dispatch::text::reflow` matched only `Anchor::Run`, so Reflow
refused every opened paragraph with a sentence about "placing new text" — Ken's
"Reflow never does anything" (O295). Its driven check used OS input, so under
`--no-input` it SKIPped for as long as the defect existed.

**Why:** let-else and `if let` are non-exhaustive by design; the compiler cannot
find the orphaned consumers for you.

**How to apply:** when adding or newly producing a variant, grep every
`Anchor::<Old> {` / `let … = draft.anchor else` site and decide each. Port any
OS-input check that covers it to the scripted pointer (`dimdrive::run_on`) so it
runs under `--no-input`. Related: [[feedback_skip_red_check_stop]],
[[feedback_later_feature_captures_gesture]].
