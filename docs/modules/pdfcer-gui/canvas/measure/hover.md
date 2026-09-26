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

## Item notes

### `const HIGHLIGHT_WIDTH_PT`

Deliberately heavier than the geometry it sits on. A CAD drawing's lines
are hairlines, and a highlight the same weight as its subject is a line that
changed colour — which on a monochrome drawing viewed at a distance is not a
change at all. It is a screen-space width, so it does not thicken with zoom.

### `const HIGHLIGHT_ALPHA`

Under 1.0 for a reason rule 4 cares about: the operator must still be able
to **see the line underneath**. A solid overlay would replace the geometry
with a coloured bar, and *"is this the line I meant"* is a question about the
geometry, not about the bar.

### `fn a_segment_is_highlighted_as_a_line`

The distinction is the whole feature for a diagonal. A box around a
45° line highlights a square region containing every other line that
crosses it, which on a CAD drawing is most of them — it would answer
*"somewhere around here"* to a question that means *"which one"*.

### `fn an_entity_with_no_straight_run_is_outlined`

A curve, a text run or an image. Saying nothing at all would be worse:
the operator would move the pointer over something, see no response, and
conclude the tool had stopped working.

### `fn a_half_mappable_segment_draws_nothing`

The failure this prevents is not a missing highlight, it is a
**misleading** one: a line drawn from a real endpoint to a fallback
position points at geometry that is not there, and the operator would
aim at it.

### `struct Entity`

`Copy`, so it can ride inside [`super::Resolved`] without changing that
type's shape — which matters because `Resolved`'s whole contract is that it
is one cheap value passed from the resolve pass to the paint pass.

### `fn resolve`

`tolerance` is the same page-space catch radius the snap query uses, so the
highlight and the snap agree about what "near" means. Handing them different
radii would produce the state this whole module exists to prevent: a marker
on one line and a highlight on another.

# Why `hit_test_point` rather than the snap candidate's `source_object`

Because a snap candidate is often **not** owned by one object, and the two
questions genuinely differ. `SnapCandidate::source_object` is documented as
`None` for *"a page-axis or grid candidate, or a segment–segment
intersection between two different objects"* — and an intersection is
exactly the case where an operator most needs to be told which line they are
about to take, since by construction there are two.

So the entity is resolved from the **pointer**, independently. When the two
agree, the operator sees a node on a highlighted line and everything is
obvious. When they disagree — an intersection — they see the node at the
crossing and the highlight on the line the click will pick, which is the
information that was missing.

### `fn shapes`

Returns an empty vector rather than `Option` so a caller can `extend` a
painter unconditionally — the same shape [`super::snap::snap_marker_shapes`]
uses, for the same reason.

`to_screen` converts a page point, returning `None` when the point does not
map (off-page, or a degenerate transform). A segment with one unmappable end
is dropped rather than half-drawn: a highlight from a real endpoint to an
arbitrary fallback would be pointing at something that is not there.
