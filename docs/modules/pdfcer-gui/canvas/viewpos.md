# `canvas::viewpos` — **where the view sits this frame**, decided before
the strip is drawn


## Why this is the seam

[`super::present::show_in`] does two separable things in sequence. First it
settles **where the view is** — how much pasteboard slack this frame has,
whether the zoom is deep enough that the `f32` scroll offset can no longer
address a pixel, and which of six competing sources gets to say what the
scroll offset is. Only then does it draw: the strip, the pages, the
overlays, the selection, the gestures.

The two halves communicate through exactly three values — `overhang`,
`deep`, and an optional forced offset — and nothing in the second half
writes anything the first half reads. That is a seam rather than a cut: a
reader here needs to know nothing about how a page is painted, and a reader
there needs to know nothing about the anchor handshake or the deep tier's
hand-over.

## What this module does NOT own

Each of the decisions below belongs to a module of its own and is *called*
from here, not made here: [`super::tier`] measures the overhang,
[`super::deep`] owns the two hand-overs across the deep threshold,
[`super::fit`] spends a fit command's placement request, and
[`super::offset`] ranks the six sources. This module is the **order** they
happen in and the one frame's worth of geometry they all read — which is
itself load-bearing, and is why it is one function rather than four calls
scattered through a 1,300-line one.

## The one thing it deliberately does not do

It does not apply the offset. [`Position::offset`] is returned, and
`show_in` is where the `ScrollArea` is configured — for the same reason
[`super::offset::decide`] returns rather than applies: **one place
configures the area**, so a second opinion cannot be added by accident.
