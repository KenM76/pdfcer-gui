# `panels::properties::mkcolour` — one `/MK` colour, wherever it is asked for

A labelled swatch over one of a widget's two `/MK` colour keys, `/BG` and
`/BC`. `OPERATOR_REQUESTS.md` **O202**, whose ask covers both halves of the
life of a form box: *"the forms objects have no way to edit their colour
before or after placement."*

## Why this is its own module rather than two similar rows

Before placement the answer goes into a [`crate::canvas::formfield::Draft`]
field; after placement it goes into a `WidgetEdit` and an undo entry. Those
are different destinations, but every question in front of them is the same
one — Table 189 gives each key four states, two of which no swatch can draw,
and each of those owes the operator a **mark** for the button face and a
**note** in the popup. Written twice, the placement dialog and the
properties pane would answer the CMYK question differently within a month.

So this module owns the reading, and the two callers own only where the
answer goes.

## What it deliberately does not own

The **sentences**. Every string arrives through [`Row`], because what *no
colour* means is a fact about which key this is — a background stating it
paints nothing at all, a border stating it is still drawn black — and that
is knowledge `text::panels::formfield` holds, not this file.

## Rule 4

Nothing here marks the canvas, in either caller. After placement the colour
is applied and from that instant the page shows what the saved file will
show. Before placement there is no content yet to mark; the swatch is part
of the cursor, not part of the document.
