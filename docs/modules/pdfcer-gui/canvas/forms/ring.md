# `canvas::forms::ring` — the order Tab visits a page's fields in

`OPERATOR_REQUESTS.md` O204, the field half.

## Contract

[`rings`] answers, for one document revision, *which boxes of
[`super::placed`]'s list are tab stops, on which page, in what order* —
together with everything that had to be inferred or dropped to say so.
Stops are **indices into the caller's `&[WidgetBox]`**, because that list is
already the thing every other part of this module addresses a field by.

## The order is the engine's, and this module sorts nothing

`EditSession::page_tab_sequence` implements §12.5.1 — `/Tabs` `/A`, `/W`,
`/R`, `/C` and `/S`, the `/Rect` normalisation, the page `/Rotate`, the
`/R2L` viewer preference and the four exclusions a reader never visits. A
second implementation of a rule that fiddly disagrees with the first the day
a page is rotated, and the disagreement would show up as *the wrong field*
rather than as an error. So nothing here compares a rectangle.

What this module does is the one step the engine's own doc says a caller
owes it: **collapse each radio group to a single stop**, which is a grouping
of fields rather than of annotations and so cannot be done from an `/Annots`
walk.

## Where the disclosure goes

`TabSequence::notes` are operator-readable sentences about inferences the
operator cannot see — a `/Tabs` pdfcer chose a convention for, a tail the
standard contradicts itself about. Rule 4 requires them to be reported and
forbids marking the canvas with them, so they are carried on [`TabRings`],
traced, and left for an off-canvas surface to print verbatim. Nothing here
paints.
