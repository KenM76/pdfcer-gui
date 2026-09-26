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

## Item notes

### `fn handover_offset`

`OPERATOR_REQUESTS.md` O26e. See the branch in [`show_in`] that calls it
for why the hand-over back had to exist, and
[`viewer::deep::DeepAnchor::page_local_offset`] for why the arithmetic is
there and not here.

# What this function contributes beyond that arithmetic

**The last zoom step.** `DeepAnchor` describes the position at the zoom it
was last updated for — `doc.frame.deep_zoom` — and this frame is running at a
*new*, lower zoom whose step nothing has applied to the anchor: the deep
branch's `zoomed_about` call is inside the `if deep` arm, and this frame is
not in it. Handing the stale anchor straight over would keep the position
but discard the final notch of zoom-about-the-cursor, so the last step out
of deep zoom would be the one step that did not hold the pointer.

The anchor point is the pointer when it is over the canvas and the
viewport's centre when it is not — the rule stated once in
[`zoom::anchor_point`] and applied here in the same words the deep branch
applies it in, against the same `ui.max_rect()` so that a pointer resting
on a ruler gutter counts as "not over the page" for both.

`None` when there is no zoom to describe a placement at, which the caller
treats as *"fall through to the ordinary anchor"* — the behaviour before
this function existed, and safe because it is only reachable for a zoom
that is not a positive finite number.

### `fn strip_placement`

The anchor says *this page point sits under that screen pixel*, so the
current page's top-left lands at `anchor.screen − anchor.page × zoom` from
the scroll content's origin, and the strip's origin is that less where the
page sits inside the strip.

Every large magnitude is subtracted **inside `f64`** before anything
narrows. At a trillion percent `anchor.page × zoom` is around 10¹², where
an `f32` cannot represent the difference of two neighbouring screen pixels
at all — this is the same technique the engine's own deep-zoom work
describes as *"one subtraction moved into `f64`"*, and it is the reason the
tier exists.

Falls back to [`viewer::deep::DeepAnchor::origin`] rather than declining,
because a frame in this tier must draw something and the origin is the one
placement that needs no history. The caller seeds a real anchor on the same
frame it first becomes `deep`, so the fallback is reachable only on a frame
where the seed itself failed.

### `fn visible_in_strip`

The scroll offset cannot answer this at this tier, because it has been
forced to zero and describes a place nobody is looking at. The strip's own
placement on screen is the truth instead: whatever of it overlaps the
viewport is what can be seen, so the viewport's origin expressed in strip
space is simply `content_min − strip_min`.

A two-line function with a paragraph of reasoning, deliberately. Below the
threshold the same quantity comes from
[`crate::canvas::geometry::scroll_to_strip`] and the two look
interchangeable; they are not, and one line of the wrong one is the whole
of `canvas-unavailable reason=nothing-visible`.
