---
name: use-the-conventional-interaction-never-invent-one
description: "Ken's strongest directive, 2026-08-19: use the most common method other programs use. Inventing an interaction model is a defect even when every part of it works."
metadata:
  type: feedback
---

**When there is a conventional way to do something, use it. Do not invent a
better one.** An invented interaction is a defect even when every component of
it works correctly.

Ken, 2026-08-19, on week two of the new shell:

> *"The selector should be predictable like other programs. It seems a lot of
> ideas are getting invented instead of just using the LLM weighting that would
> have produced the most common method expected."*

**Why:** he was reporting that the canvas was unusable — could not edit text,
could not make new text, could not see or move end points. **Every one of those
features already worked.** What was invented was *reaching* them:

- to type one character: Edit mode → the Edit tab → *Edit text* → click. Four
  steps, none signposted.
- to move an end point: click → double-click → double-click, descending a rung
  ladder with **nothing drawn at any stage** saying a deeper rung existed.

Each individual decision had a written justification and most of them were
locally sound. The result was a program nobody could use. The old GUI, which he
calls "wonky and buggy", had these working in week one.

The fix was to adopt the layout every editor in the class already uses — `V`
Select, `A` Points, `T` Text, `H` Hand, the tool decides what you are selecting
— and it took a few hours because the underlying machinery was all present.

**How to apply:**

- Before designing an interaction, ask *what do Illustrator, Inkscape, Acrobat,
  Word and the old shell do?* If they agree, that is the answer. **The
  convergence IS the specification.**
- A design that requires the operator to learn a new model needs to buy
  something enormous. "It is more principled" is not enormous.
- Be especially suspicious of a *model* you are proud of — the rung ladder was
  the most elegant thing in this codebase and it was the defect.
- Watch for the tell: **the feature works and the operator cannot find it.**
  That is never a documentation problem; it is an interaction problem.
- Bare single-letter tool chords are safe here and are what his hands know.
  `RIBBON_IA.md` §3's rule already said this — take the chord the product class
  settled on unless it collides.

Related: [[never-defer-on-an-external-blocker]] (its sibling — that one is about
not inventing a *blocker*, this is about not inventing a *design*).
