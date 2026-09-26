# `canvas::deep` — everything the canvas does about the `f64` position tier

## Why this is its own module

`OPERATOR_REQUESTS.md` O24 introduced a second mechanism for *"where is the
view"*. While the content extent `longest_page_pt × zoom` stays under
[`crate::viewer::ceiling::SUB_PIXEL_CONTENT_EXTENT`] the `egui` scroll
offset holds it, as it always has; above, an `f32` measured in screen pixels
over that content space can no longer address every pixel — at an extent of
20.5 billion px, the figure `viewer/ceiling.rs` actually drove on a Letter
sheet, it moves in 2,048-px jumps — so
[`crate::viewer::deep::DeepAnchor`] takes over, holding a page point in
`f64` and the screen pixel it sits under.


Two mechanisms means **two hand-overs**, and the whole of this module's
subject is the seam: seeding the anchor on the way in, restating it on
every zoom while it holds, moving it on a pan or a wheel, and converting it
back into a scroll offset on the way out.

It lives in its own file because the seam is where the defects are and
it had become impossible to see them together. O24f found three faults in
the upward hand-over. O26e then found that the **downward** one did not
exist at all — the anchor was discarded and the `f32` machinery resumed
from the zero this tier forces, which put the page's own origin under the
pointer and lost twelve million pixels of drawing. Both sets of code were
inside a 1,700-line `canvas::show`, hundreds of lines apart, and R2's rule
exists for exactly that: *"nothing could be reasoned about locally"*.

## The invariant this module owns

> **While the tier holds, the scroll offset is meaningless and must not be
> read as a position; on the frames either side of it, the two mechanisms
> must agree about where the view is to within the `f32` offset's own
> resolution.**

Everything here is in service of the second clause. The first is why the
caller forces the scroll offset to zero while `deep` is true, and why
`canvas::show`'s current-page tracker skips its scroll-derived answer at
this tier: both would otherwise be reading a value that says nothing.

## What is deliberately NOT here

The **placement** — where the strip is drawn from the anchor — and the
**visible rect** the raster region is derived from. Those are branches
inside `canvas::show`'s scroll-area closure, where the rects they produce
are consumed a few lines later, and lifting them out would trade one
locality for a worse one. They read the anchor; they do not maintain it.
