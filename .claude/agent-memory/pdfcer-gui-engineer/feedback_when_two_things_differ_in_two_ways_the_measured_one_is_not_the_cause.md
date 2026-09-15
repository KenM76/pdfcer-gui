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

## ★ THIRD, 2026-09-15 — in a codebase that is half comments, the DOCUMENTED mechanism attracts the blame

`RESUME.md` said of O201: *"`fill_strip` asks for one visible page per frame and
`RenderWorker` has a single slot, so nothing is ever rendered ahead."* The second
clause is false and the first is nearly a tautology. `RenderWorker`'s single slot
is deliberate, correct, and irrelevant — a prefetch never needs a second slot,
because it is only ever issued on a frame when the worker is already idle.

The real cause is that `fill_strip` builds its candidate set from
`doc.strip_visible` and nothing else, so no page outside the viewport is ever a
candidate at any zoom.

⇒ **Why the wrong half got named: `RenderWorker` has a fifteen-line header
arguing its single-slot design, and the `strip_visible` scope has no comment at
all.** A mechanism that explains itself is the one a reader recalls when asked
*"why is nothing rendered ahead?"*, and this repo is 52% comments, so that
distortion is everywhere. The silent mechanism is not less likely to be the
cause — it is only less likely to be **remembered**. Same failure as the two
above with a different source of visibility: a report column, then an
enumeration bullet, now a doc comment.

**How to apply:** when a `RESUME`/backlog row states a cause, the row is a
*claim*, not a measurement — and the clause that cites a well-documented
component is the one to falsify first. The falsification here was one grep:
does anything outside `strip_visible` ever reach `rasterize`? Then correct the
row, because a cold session reads it before it reads the source
([[a-register-row-outranks-memory-so-correcting-the-row-is-the-work]]).
