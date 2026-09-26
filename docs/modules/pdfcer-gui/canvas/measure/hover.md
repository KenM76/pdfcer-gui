# `canvas::measure::hover` — showing what a measuring click will pick,
before it picks it

## The report this exists for


> *"The measuring tools themselves don't give me any indication of what is
> being selected either when I use them. I should be able to hover over a
> line or node and have it indicate that is what will be selected for the
> tool to use."*

Asked which he wanted — the whole entity or the snap point — the answer was
**both**, and both is right: they answer different questions and an operator
aiming a dimension needs both answers at once.

| affordance | answers |
|---|---|
| the **node** marker (already drawn, [`super::snap`]) | *"your click will land exactly here, not where your pointer is"* |
| the **entity** highlight (this module) | *"and it will be taken from THIS line, not the one crossing it"* |

The second is the one that was missing, and its absence is worst exactly
where this application is used. A CAD drawing is a field of near-identical
strokes; an endpoint marker floating in that field says a click will land
*there* and says nothing about which of the four lines meeting there it
belongs to. For the two-line angular tool that is not a nicety — the whole
measurement is *which two lines*, and picking the wrong one gives a
confident, plausible, wrong angle.

## Rule 4: this is a cursor, not content marking, and the distinction is
exact

`pdfce_FeatureRequests/README.md` rule 4 forbids drawing pdfcer's own
uncertainty into the page — no badge, tint or dashed outline on *applied*
content. It explicitly welcomes the opposite thing:

> *A pre-commit affordance is not content marking. Snap indicators, hover
> highlights, rubber-bands and selection handles are the **cursor** and are
> welcome.*

Everything here vanishes when the pointer moves, describes what the **next**
click would do, and is never drawn over content that has been committed. The
one-line test the rule gives — *would a screenshot of the editing canvas
differ from the same document saved and reopened?* — is passed because
nothing here survives a click, let alone a save.

## Why the entity is resolved beside the snap and not beside the paint

[`super::Resolved`] exists because the indicator and the click must read
*one* derivation of "where would this land" — its own documentation records
what happened when they were two: a marker drawn over an endpoint and a
commit somewhere else, surviving four days because both functions were
individually correct.

The entity has the identical hazard and a worse failure. It needs
`PageObjects`, which is borrowed only during `canvas::interact` and dropped
before anything is painted, so a paint-time query is impossible anyway — but
if it were possible, a highlight resolved at paint time against a pointer
position read at paint time would drift from the click by whatever moved in
between. **The operator would be shown one line and would measure another**,
and the trace would show a perfectly consistent measurement of the line they
were not looking at.

So it rides on `Resolved`, computed in the same pass, from the same query
point, against the same model.

## What is highlighted, in order of preference

1. **The segment** — `pdfcer_core::vector::linepick::pick_line`, the exact
   start-to-end run the pick would use. This is what the two-line tool
   consumes, so highlighting it is showing the operator the literal operand.
2. **The object's bounds**, when there is no segment: a curve, a text run,
   an image. There is still an entity under the pointer and saying nothing
   about it would be worse than saying "this one, and it is not a straight
   run".

Both are drawn in the snap indicator's own colour, because they are one
affordance with two parts and two colours would read as two states.
