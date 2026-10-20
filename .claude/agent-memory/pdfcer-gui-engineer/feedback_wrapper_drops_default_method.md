---
name: feedback-wrapper-drops-default-method
description: A wrapper implementing an engine trait silently loses any method with a default body; installedfaces dropped SubsetAugmenter::candidates and every composite font fell to Helvetica
metadata:
  type: feedback
---

When you wrap or delegate an engine trait, list **every** method the trait
has, including the defaulted ones, and forward each. Do not stop at what the
compiler demands.

**Why:** `InstalledFaces` forwarded `augment` and `addable` but not
`candidates`. Its default returns an empty list. Simple fonts kept working.
Every `/Type0 Identity-H` subset was offered no letters, so the GUI promised
Helvetica for a `z` the installed Century Gothic had (O311). The engine reply
for G075 had named `candidates` weeks earlier. No test could see it, because
`AugmentRequest` has no constructor. The finding is filed as G210.

**How to apply:**

- Before implementing an engine trait, open its definition and grep it for
  `fn .*\{`, which are the methods with default bodies.
- Measure with a fix/control pair on a real file. A repertoire query also
  needs `disposition::typing()` options: `EditOptions::default()` answers
  false even with the fix in place.

Related: [[feedback_grep_sibling_crate]], [[feedback_capability_change]].
