# `app::status::filter` — the Select popup: what a click may land on

`OPERATOR_REQUESTS.md` O17's first half. The status bar's **Select**
button, the eleven-row popup behind it, and the standing line that appears
when the operator has left nothing selectable at all.

[`crate::canvas::pick`] holds the model — the eleven classes, the
subtractive invariant, and why the filter composes with the mode as an
`AND` rather than as an override. This file holds only the surface, and the
split is the usual one: that module can be asserted about in a unit test,
while everything here has to be **driven** before it counts (R1).

## Why this is a file and not a section of [`super`]

R2's 1,500-line ceiling forced the split, and — as with
[`super::page_box`], [`super::notes`] and [`super::decline`] before it —
the forced seam turned out to be a real one. Everything else on the bar
answers *what is true about the view*: which page, what zoom, what the last
raster contained, why a command declined. Those are all **reports**.

This is the one thing on the status bar that is not a report. It changes
what the pointer does. That is a different kind of control living on a
surface full of readouts, and it earns its own file for the same reason it
earns its own position in the layout — see [`show`] on why it sits at the
left edge of the fixed cluster rather than inside the zoom group.

## What this module does NOT do

It does not persist anything. The caller compares the filter before and
after and writes it if it moved — see [`crate::app::frame`]'s status-bar
block for why that comparison lives there, and [`crate::app::pickstore`]
for why the write is immediate where the dock layout's is debounced.
