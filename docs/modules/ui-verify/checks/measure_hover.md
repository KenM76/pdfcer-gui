# `ui-verify/checks/measure_hover`

`measure_hover` — **hovering with a measure tool armed says what the click
will take, before it takes it.**

# The report


> *"The measuring tools themselves don't give me any indication of what is
> being selected either when I use them. I should be able to hover over a
> line or node and have it indicate that is what will be selected for the
> tool to use."*

Asked whether he meant the entity or the snap point, the answer was
**both** — and this check asserts both, because either alone leaves the
question the other answers open.

# ★★ Why this cannot be a unit test, in the specific rather than the general

The two halves are resolved in one pass, in `canvas::measure::resolve_hover`,
while the page decomposition is borrowed — and then painted several
functions later, after the borrow is dropped. Everything in between is a
`Copy` struct being handed along.

Every piece of that is individually testable and the composition is not:
the failure mode is *the pointer position one of them read*. That is exactly
how the defect the `Resolved` type exists to prevent got in — the marker
resolved against a raw screen position while the click used a converted
canvas one, so the two disagreed by the scroll origin over the zoom, which
is **zero at the top-left of an unscrolled page at 100 %**. It survived four
days and looked like *"sometimes it is fine"*.

A driven run with the page scrolled and the pointer somewhere real is the
only thing that has ever caught that class here.

# What it asserts

| Phase | Does | Expected |
|---|---|---|
| A | arm Measure ▸ Linear | `measure-tool tool=Measure(Linear)` |
| B | move the pointer over page geometry, **without clicking** | `measure-hover-entity object=… segment=…` |
| C | the same frame's snap marker | `measure-snap-marker` within tolerance of the pointer |
| D | move to blank paper | the entity line stops being emitted |

★ Phase D is the half that stops this passing on a build that highlights
*everything*. A highlight that never retires is not an indication of what
is under the pointer, it is a decoration — and it would satisfy every
assertion above it.
