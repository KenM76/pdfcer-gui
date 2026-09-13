---
name: a-type-reached-only-by-field-access-is-invisible-to-the-drift-gate
description: check-engine-api-drift accounts an item only if both leaf and owner appear as Rust words — a struct read through an inferred binding names nothing, so a rename upstream lands as a silent behaviour change; fix with a type-annotated binding, never a doc comment
metadata:
  type: feedback
---

**If the only way this repository touches an engine type is field access on an
inferred binding, the type's NAME appears nowhere and `check-engine-api-drift`
correctly reports it as unaccounted. Fix it by writing a real type-annotated
binding — not by mentioning the name in prose.**

**Why:** on 2026-09-13 the gate failed with three unaccounted items:
`pdfcer_core::recover::DroppedObject` and both of its fields. Every line that
read them looked like `for d in &report.objects_dropped { … d.number … }`, so
the struct's name was never written. The gate's accounting rule
(`tools/gates/check-engine-api-drift.py`) is *"accounted if `leaf in rust_words`
and (`owner is None` or `owner in rust_words`)"* — with no owner word, nothing
matched.

**The gate was right, and the finding is not paperwork.** That is exactly the
condition under which an upstream rename lands as a **silent behaviour change
instead of a compile error**: field access through an inferred binding keeps
compiling against a renamed type as long as the field names survive.

The repair was one line inside the test that already read the field:

```rust
let dropped: &[pdfcer_core::recover::DroppedObject] = &report.objects_dropped;
```

**How to apply:**

- **A doc-comment mention silences the gate and buys nothing** — it does not
  break the build on a rename, which is the only property that matters here.
  If a fix makes a gate green without changing what the compiler checks, the
  fix is wrong.
- The natural home is the unit test that already asserts the fixture's
  properties: the binding costs a line, and a rename then fails in one second.
- Annotate the comment with **why** the annotation exists, or the next `cargo
  clippy` cleanup deletes it as redundant.
- Same shape applies to any engine enum matched only by its variants, or any
  struct destructured by pattern.

Related: [[feedback_an_api_drift_hit_is_sometimes_a_feature_not_paperwork]],
[[feedback_a_tripwire_keyed_on_your_own_intention_is_not_a_tripwire]],
[[feedback_a_gate_keyed_on_a_name_is_discharged_by_prose]].
