# `canvas::present` — **drawing the canvas**: the scroll area, the pages in
it, and the geometry the frame hands back

[`show`] and its body [`show_in`], plus the constants and helpers that only
they use. Everything else about the canvas — what a click means, what a drag
does, what is selected — lives in the sibling modules `canvas` indexes.

## Why this is its own file


⇒ It was. R2's own instruction is *"when a file approaches the limit, that is
the signal to find the seam, not to raise the limit"* — and a module index
that cannot accept a new entry without something else being deleted is a
file that has stopped being an index.

## The seam, stated

`canvas/mod.rs` is now **only** a module index and the canvas header: 53
`pub mod` lines, each with the paragraph that says why that module exists.
Adding a 54th costs nothing and takes nothing away.

This file is the one thing that file also happened to contain — a thousand
lines of *drawing*, which is a different subject from *what the canvas is
made of*. They change for different reasons, which is the test this project
applies to every split it makes.

## Nothing moved except its address

The move is textual: the same items, in the same order, with the same
documentation. `show` and [`Sampled`] are re-exported from `canvas`, so
every call site still says `canvas::show(...)` and no caller learned that
this file exists.

## Item notes

### `fn show_in`

Returns the context-menu tokens *and* what the frame learned about where
its pages ended up — see [`CanvasGeometry`] on why that has to travel
outwards rather than be read again.
