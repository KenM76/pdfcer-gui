---
name: an-index-handed-across-a-boundary-must-be-numbered-in-the-callees-own-list
description: An integer subscript is not a description — when a verb takes `usize`, the only correct answer is one numbered in the list that verb itself builds, and a caller that numbers it in its own list is wrong in a way no type, test or gate can see
metadata:
  type: feedback
---

**When another module's verb takes an integer index, resolve that index against
the list THAT MODULE builds — never against your own list of the same-sounding
things.** Read the callee's source for which options it re-derives the list
with, and call the same constructor.

**Why:** O198 claim 2. `EditSession::reflow_block(page, block, req)` takes a
`usize`. The shell answered *"which paragraph did he click in?"* correctly, with
`EditableTextModel::recognize(&text, &BlockRecognitionOptions::default())` — the
caret's own recognition, the one every other text surface uses. But
`reflow_block` throws the caller's context away, **re-extracts the page and
re-recognises it with `reflow_recognition_options()`**, a relaxed configuration
(`indent_ratio = 1.0e6`) that MERGES ragged-left lines the default splits apart.
Two lists, two lengths. On his own drawing, page 0, the run under *"USE
SPACERS"*:

```
caret recognition : block 106 of 144
reflow recognition: block  49 of  70
```

So the shell handed over 106 — a subscript into an array the engine never
builds — and the engine reflowed whatever paragraph happened to be 106th in
*its* list, or refused. Reflow appeared to "work near the top of the sheet and
stop working further down", which is exactly what a numbering that drifts
produces, and is why the operator's report contained the word *"seems"*.

**★ Nothing could see it.** Both sides are `usize`. Both functions are correct
in isolation and well tested. The driven check
(`reflowing_a_paragraph_rewraps_it`) was **green throughout**, because its
fixture is a flush-left six-line paragraph that both recognitions call *block 0
of 1* — see [[an-assertion-both-outcomes-satisfy-is-not-a-measurement-of-which-one-shipped]].
Eight unit tests passed. The probe printed 106 and nobody asked *106 of what*.

**How to apply:**
- Treat `fn f(&self, i: usize)` in another crate as a **question about which
  list**, and go read the answer before writing the call. Grep the callee for
  where it materialises the collection it subscripts.
- Suspect this class whenever the same domain concept has **two configurations**
  — two recognition option sets, two sort orders, two filters. If both sides
  build their own copy, they will diverge the first time one is tuned.
- The generalisation is broader than indices: **a condition and its operand must
  come from the same resolver.** The shell hit the same shape with
  `selection.text_runs`, and solved it by making the ribbon condition, the
  read-back and the dispatch all read one function
  (`app::textoperand`) — which is the fix pattern, not just the diagnosis.
- Write the correcting comment where the mapping is made, and name the
  measurement (`106-of-144` vs `49-of-70`), not the conclusion. A later reader
  needs to be able to re-run it.
- A probe or check that PRINTS an index must print the list length beside it.
  `block 49 of 70` is a measurement; `block 49` is a number.
