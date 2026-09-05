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
