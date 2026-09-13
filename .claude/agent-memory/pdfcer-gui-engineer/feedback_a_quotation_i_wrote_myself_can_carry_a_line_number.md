---
name: a-quotation-i-wrote-myself-can-carry-a-line-number
description: A quoted sentence with a file:line that exists nowhere in the cited crate — not stale, never true; the claim was right and the evidence was manufactured
metadata:
  type: feedback
---

**A citation is a measurement. If a quoted sentence is not a copy of something
on disk, it is a paraphrase, and it must be spelled as one — even when the
claim it supports is true.**

**Why:** 2026-09-13, `UNIT_SURFACES.md` §2. I wrote:

> ★ **There is no `Unit::Point`, and that is correct.** `units.rs:495-498`:
> *"points are what a measurement is before a unit is chosen."*

The **claim** is true: `pdfcer_core::dimension::Unit` has no `Point` variant,
deliberately, and every bare ` pt` in the GUI is therefore the *absence* of a
unit rather than a unit an operator chose. That argument is load-bearing for a
whole operator request (O194, *"in some places only have points as a dimension
type"*) and it is correct.

**The quotation does not exist.** Not at `units.rs:495-498`, not in that file,
not anywhere in `pdfcer-core`, at the pin I read or at the live tree's HEAD.
`grep -rn 'before a unit is chosen'` returns nothing in either. Line 495 is the
middle of `format_feet_inches`'s fraction reducer. I wrote a sentence that
expressed what I believed the engine's reasoning to be, put it in quote marks,
and gave it a line number.

⇒ **This is a different failure from a stale citation and must not be filed
with them.** A stale citation was true once; re-measuring fixes it and the
lesson is about shelf life. This one was **never true**, and re-measuring it
finds *nothing*, which reads like a deleted comment rather than a fabrication —
so the natural repair is to go hunting for where it "moved to", which wastes
the time and then quietly re-points the fake quote at some real line that
almost says it.

**How to apply:**

- **Quote marks around another repository's prose are a promise that the string
  is on disk.** Before shipping one, `grep` for a distinctive five-word run of
  it in the cited crate. If it does not match, it is not a quotation.
- **When the claim is true and the evidence is not, keep the claim and change
  its form** — state it as something one grep falsifies (*"the only `Point` in
  `units.rs` is `DecimalMarker::Point` at `:355`, which is a decimal separator,
  not a length"*). Do **not** re-point the quotation at a different line.
- **The tell in my own writing:** a citation that reads as unusually
  well-phrased *for the argument being made*. Real source comments are written
  about their own subject, not about the point I need them for. When a quoted
  sentence sounds like the topic sentence of my paragraph, I wrote it.
- Record the correction in place, with the fabricated wording shown, rather
  than silently deleting it. A reader who met the quote once needs to know it
  was never real.

★ Related but distinct: [[a-citation-that-supports-my-theory-may-be-about-a-different-document]]
is a **real** citation about the wrong subject; this is **no** citation wearing
one. Both share the root — I went looking for support and stopped when the
sentence sounded right instead of when the string matched.
