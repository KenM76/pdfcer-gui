---
name: when-two-things-differ-in-two-ways-the-measured-one-is-not-the-cause
description: A contrast the operator noticed had two explanations at once; I blamed the one that has a column in a report and filed a wrong engine request on it.
metadata:
  type: feedback
---

When the operator hands you a contrast — *"this half works, that half doesn't"* —
enumerate **every** way the two halves differ before choosing one. The
difference you can already measure is not more likely to be the cause; it is
only more likely to be **noticed**.

**Why:** 2026-09-05. Ken: *"the lines I added below `price)` are editable, but
everything else that existed when I got the pdf is not."* I ran `list-fonts`,
saw every arriving face marked `verdict=blocked-identity` against pdfcer's own
`WinAnsiEncoding` resources, and filed a request at the engine asking it to
invert `/ToUnicode`.

`AAAAAA+Arimo-Bold` — one of the three faces I named as blocked — **edits end
to end on that very page.** So does `pdfcer-core`'s own test fixture carrying
the identical verdict line. The real cause was that his producer writes **one
show operator per glyph**, and `edit_text` matches inside one operator, so a
five-character `find` could never match. His added lines are one operator each.
Both differences were real; only one was the cause.

⇒ **`list-fonts` prints a font verdict. Nothing prints a producer's
batching.** The hypothesis that had an instrument won, and the instrument was
not measuring the question.

**The tell we both walked past:** the engine answered `NoMatch`, **not** a font
refusal. *A font refusal names the font.* When a refusal's category does not
match your hypothesis's category, your hypothesis is the thing in doubt — not
the refusal's wording.

**How to apply:** before filing anything at the engine on a
works-here/fails-there contrast, list the differences on paper and try to
**falsify** the chosen one directly — here, one command editing the supposedly
blocked font would have taken ninety seconds and saved a wrong request. Related:
[[a-limitation-sentence-is-a-citation-with-an-hours-long-shelf-life]],
[[a-driven-failure-is-a-claim-about-the-check-too]].

## ★ SECOND, 2026-09-13 — and this time both differences were in MY OWN analysis

Two surfaces printed different whole millimetres for one sheet. They differ in
two ways: one converts in `f32` and the other in `f64`, and one rounds with
`{:.0}` while the other rounds with `.round()`. I blamed the precision — and
published that — because precision had a bullet of its own in the enumeration
and felt like the kind of thing that causes a numeric disagreement.

Both paths land on exactly `210.5000000000`. The cause was the rounding rule
alone, which the bullet directly *above* had already named correctly.

⇒ **The tell here was available with no instrument at all: a tie.** `210.5` is
exactly representable, so a precision story has to explain how two values that
are equal print differently — and it cannot. When your hypothesis needs the
inputs to differ, **print the inputs** before writing the sentence. Full account
in [[a-capability-claim-in-product-copy-needs-the-same-citation-as-a-limitation-claim]].
