# `text::images` — the words the Insert-image window shows

## What this surface is

`edit.insert_image` was registered, drawn on Edit ▸ Insert, listed in
`reach`'s `SCAFFOLDED` set with **no recorded reason at all**, and inert.
`EditSession::add_image` has shipped the whole time.

## This catalog's hardest job: saying what a resolution MEANS

An image placed into a PDF has no resolution of its own — §8.9.4 maps it
onto the **unit square** and the content stream's matrix scales that square
to whatever size the page asks for. So *"is this picture big enough?"* is a
question about the **placement**, not about the file, and it has no answer
until the operator has said how large the picture will be on paper.

`pdfcer-core` reports the answer as a number rather than as a warning, and
its own doc comment is the argument this catalog follows:

> Not a warning — a number. An operator dragging a 4000-pixel photo into a
> 2-inch box gets 2000 dpi (wasted bytes) and one dragging a 100-pixel logo
> across a page gets 12 (visibly soft), and **neither is visible on screen
> at editing zoom**.

That last clause is why the sentence exists: both mistakes look perfect
until the sheet is plotted.

## It IS previewed now, and the request that got it is worth the paragraph

This section used to say the resolution could not be shown before the
commit, because `NewImage` offered `placed_rect()` as a pure preview and
nothing for the resolution — and computing `pixels / (points / 72)` locally
would have been the second derivation `placed_rect()`'s own doc warns
about: *"re-deriving the arithmetic in the GUI is how a preview and a result
drift apart."*


**And the four-line version this shell nearly wrote would have been
wrong.** Under `ImageFit::Contain` the placed rectangle is the *letterboxed*
sub-rectangle, not the box the operator typed — so measuring `rect` reports
a resolution low by exactly the letterbox ratio. The pure sibling is not
saving four lines; it is saving the letterbox.

A **zero-area** placement reports `(0.0, 0.0)` and `below_screen_resolution`
is `true`. Zero is not a resolution, but it is a number a label can render,
and the engine chose it over `inf` and over `NaN` — the last of which would
have made its own preview/outcome equality test pass vacuously.
