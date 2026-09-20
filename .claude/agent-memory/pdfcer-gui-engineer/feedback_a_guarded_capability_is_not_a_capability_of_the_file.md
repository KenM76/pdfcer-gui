---
name: a-guarded-capability-is-not-a-capability-of-the-file
description: Before writing a capability into public copy, find the `if` that turns it on and count how often it is true — the CMYK ink path engages for 15 files in 4,012 and the draft README said it happened to every CMYK page
metadata:
  type: feedback
---

**Before printing a capability on a public surface, find the branch that
enables it and measure how often that branch is taken. A capability that is
real in the code can be rare in the files.**

**Why:** the O197 README rewrite (2026-09-13) was going to say *"a page
authored in CMYK is composited as ink on separate colorant planes."* Every
word of that is supported by `pdfcer-render/src/cmyk_buffer.rs` — the four
`f32` colorant planes are there, the spot planes are there, they are wired to
fills, images and shadings. What is not there is the *reach*.
`pdfcer_render`'s `render_impl_rasterize` gates the whole thing on
`page_space.is_subtractive()`, and the comment beside the switch counts it:
**13 of 51 print-conformance files and 15 of 4,012 external fixtures.**
Everything else keeps the sRGB path, and ISO 32000-1 §8.6.6.4 makes that the
*specified* behaviour on an additive device — so the narrow reach is correct,
not a shortfall.

The trap is that reading the implementing module gives you a completely
truthful impression of a capability and tells you nothing about how often it
happens. I had read `cmyk_buffer.rs` end to end and was confident. A
subagent sent to verify the claims found the guard in one grep.

**How to apply:**

- Public copy, release notes, FEATURES rows, replies to Ken: for each
  capability sentence, grep for the condition that switches it on. `if
  something.is_x()` around the feature is the thing to find.
- Then ask for a *count*, not a yes/no. The engine's own comments frequently
  carry one ("13 of 51", "15 of 4,012") because the engine author had the
  same question.
- If the reach is narrow, look for the part that IS universal and lead with
  that instead. Here it was the measured 1,296-point CMYK conversion table in
  `pdfcer-core/src/color/mod.rs`, which every `DeviceCMYK` colour in every
  file goes through — a better claim than the one that was wrong, and it was
  sitting next door.
- Scope the sentence rather than deleting it: *"when a file asks to be
  composited in ink, it is"* is honest and still impressive.

This is the frequency sibling of
[[a-capability-claim-in-product-copy-needs-the-same-citation-as-a-limitation-claim]]
(which is about *inventing* a claim) and of
[[a-capability-can-change-meaning-under-a-stable-signature]] (which is about a
claim going stale). This one is about a claim that is sourced, current, and
still wrong because its scope was never measured.
