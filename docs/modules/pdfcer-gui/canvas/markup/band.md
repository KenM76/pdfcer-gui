# `canvas::markup::band` — the two-point rubber band

Rectangle, Ellipse, Arrow and Highlight: **press, drag out a shape,
release.** One of the four gesture families [`super`]'s header tabulates;
[`super::vertex`] and [`super::ink`] own the other shapes.

## ★ The seam against [`super`]

[`super`] holds *what a markup is* — the kinds, the geometry, `spec`,
`action`, the pen. This file holds *how this family is gestured*: the
canvas→page conversion for two points, the one function that touches the
frame, and the band that is drawn while the button is down.

**The seam is a subject and not a line count**, and the test is that the two
sides change for different reasons. A new markup *kind* changes [`super`] —
a variant, an `rgb` arm, a `spec` arm — and does not touch this file unless
it is band-shaped. A change to *how a band is drawn* — a snap, a modifier
that constrains the aspect ratio, a different preview — changes this file
and nothing in [`super`].

## The band draws the shape it is about to author, not a box round it

Rule 4's pre-commit affordance, applied literally. [`draw_preview`] carries
the argument, including what an ellipse previewed as its inscribed *circle*
costs.

## A click with no drag places nothing

[`super`]'s header carries that decision in full, including the two reasons
a 120 × 60 default box and a 4-point page-space threshold are both
deliberately absent. The mechanical half of it lives here: [`drag`] is
reached only from `GestureOutcome::Markup`, which only a real drag produces,
and a zero-extent drag is refused by [`super::action`].
