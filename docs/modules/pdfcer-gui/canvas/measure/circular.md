# `canvas::measure::circular` — the radius/diameter tool, and the gesture
the operator has to end

The canvas hosting for [`MeasureKind::Circular`] alone: what one click does
to the fit set, what the two endings do, and what has to be resolved out of
the decomposition so the set can be drawn. [`super`] hosts the other two
tools and everything the three share — the memory, the snap resolution, the
preview painting.

## ★ Why this is a file of its own, and what the seam actually is

**R2** (no `.rs` file over 1,500 lines) forced a split when the tool was
armed: [`super`] reached 1,617 lines. But the line count only says *that*
something had to move; it does not say what, and `tools/gates/check-file-size.sh`
says in its own header that shaving prose to fit a threshold is the
behaviour it exists to refuse. So the question was which subject was
separable, and this one is, for a reason none of the other tools give:

> **Linear and two-line gestures end themselves. This one does not.**

A linear dimension is finished at its third click and a two-line dimension
at its second, because both are picks of a **fixed arity** — the pick
machine in [`super::pick`] knows it is done, and [`super::click`] simply
raises whatever the machine hands back. A best-fit circle has no such
number. An arc drawn as four separate polyline objects needs four picks; the
same arc drawn as one needs one; nothing in the geometry can tell pdfcer
which the operator meant. So the operator says when, and the machinery for
*saying when* — two entrances, one commit path, a predicate the ribbon reads
every frame to decide whether the control is even live — is a subject the
other two tools have nothing corresponding to.

That is the seam. Everything here answers *"when is this gesture over, and
what does ending it do?"*; everything left in [`super`] answers *"where did
that click land?"*.

## The two endings, and why there is exactly one commit path

| ending | entrance | why it exists |
|---|---|---|
| **double-click** on the canvas | [`click`], via [`super::click`]'s `double` flag | what every drawing package's multi-pick tool uses; the standing *"make it work the way other programs do"* tie-breaker |
| **`measure.finish`** on the ribbon | [`finish`], via `app::dispatch` | discoverable without knowing the double-click, and reachable when the last picked arc sits somewhere awkward to double-click |

Both call [`commit`] and nothing else raises a circular
`Action::CommitDimension`. Two arms that each assembled a `DimensionKind`
would be two derivations of one answer: they would agree on the day they
were written, diverge at the first change to either, and **the operator
would have no way to see it** — a circle fitted from the same points looks
the same whichever code drew it.

Neither ending is an accept box floating over the canvas, which is what
decision 024 retired at the operator's instruction and what kept this tool
unarmed through Phase 7.

## This module owns no geometry either

The fit is [`pdfcer_core::dimension::fit_circle_taubin`], reached through
[`super::pick::CircularPick`]; the authored value is `pdfcer-core`'s own
`DimensionKind`. Nothing here computes a centre, a radius or a residual.
What it owns is *composition and lifetime*: which objects are in the set,
when the set becomes a dimension, and when it is emptied.
