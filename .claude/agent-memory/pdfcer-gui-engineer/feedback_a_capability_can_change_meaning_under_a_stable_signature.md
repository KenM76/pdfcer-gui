---
name: a-capability-can-change-meaning-under-a-stable-signature
description: A predicate the shell quotes in prose can be redefined by the engine without changing its name, type or arity — nothing breaks, nothing warns, and every sentence explaining the old behaviour silently becomes a lie
metadata:
  type: feedback
---

**When the engine changes what a function MEANS without changing what it looks
like, every sentence in this shell that explains the old behaviour becomes
false, and no build, gate, test or compiler can see it. Re-read the engine's
body, not its signature, whenever prose here quotes its behaviour.**

**Why:** 2026-09-11. `pdfcer_core::vector::FormLeaf::is_editable` used to mean
*nothing inside a form is editable* — it returned `false` for every leaf. At
`Pass 188.0` it became:

```rust
pub const fn is_editable(&self) -> bool { matches!(self.object, VectorObject::Path(_)) }
```

Same name, same arity, same return type, same `const`. **Ten days passed.** In
that window six form-scoped geometry verbs shipped beside it and this shell
wired all six on 2026-09-01, so the capability was live in every release binary
— while **four prose sites in this tree still explained why it could not be**,
including an operator-facing refusal sentence reading *"That object is inside a
form — pdfcer cannot edit inside one yet"*, and a test-module header arguing
that *"the ladder stops at the Object rung for a leaf"*. It does not stop.

The engine's own release note stated the hazard precisely, and it is the best
single sentence on this: *"a shell that greys out a node handle on `false` is
still right, and a shell that greys out **the whole container** on `false` is
now wrong."* A narrowed predicate does not break the callers that were already
narrow. It breaks the ones that were **wide**, and being wide is not a
compile-time property.

**★ The dangerous half is the one that survives.** The verbs were wired; what
survived was the *paragraph explaining why they could not be*. A refusal
sentence is the program's own voice telling the operator what not to attempt,
so a stale one does not merely misinform — it stops him trying the thing that
works, and he has no reason to doubt it.

**How to apply:**
- Before trusting a doc comment here that quotes engine behaviour, open the
  engine function's **body**. `git log -p` on that file across the pin bump is
  cheaper than the investigation the stale sentence will cause.
- The tell is a predicate whose name is a *judgement* — `is_editable`,
  `can_*`, `*_refusal`, `is_supported`. Those are the ones whose meaning is
  allowed to move. A getter's meaning cannot.
- Whenever a pin bump delivers a verb, ask separately: **what prose here said
  that verb was impossible?** Wiring the verb and deleting the excuse are two
  edits, and only the first one has a compiler.
- Related: [[a-limitation-sentence-is-a-citation-with-an-hours-long-shelf-life]],
  [[triage-the-reply-channel-the-engine-fixes-faster-than-we-notice]],
  [[a-tripwire-keyed-on-your-own-intention-is-not-a-tripwire]],
  [[a-wrong-refusal-sentence-hides-a-defect-in-whoever-believes-it]].
