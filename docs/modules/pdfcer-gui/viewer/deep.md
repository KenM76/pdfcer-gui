# `viewer::deep` — where the view is, when the scroll offset can no longer say

`OPERATOR_REQUESTS.md` **O24**, step 2. The operator:

> *"how do we get to the insanely high limit? … You should be able to have a
> new algorithm take over for bigger zooms?"*

He is right that something has to take over, and this is it — but it is not
about how pixels are made. **It is about where the viewport's position is
stored.**

## The measurement that makes this necessary

Today the position is an `egui::ScrollArea` offset into a content rectangle
of `page × zoom`, and those offsets are `f32`. A screen pixel is exactly one
unit of that content space, so the spacing between representable offsets
**is** the positioning error in pixels:

| zoom | content extent | `f32` step | error on screen |
|---|---|---|---|
| 100 % | 1,584 pt | 0.0001 | none |
| 10,000 % | 158,400 pt | 0.02 | none |
| **1,000,000 %** | 15,840,000 pt | 1.00 | **1 pixel** |
| 10,000,000 % | 158,400,000 pt | 16.00 | **16 pixels** |
| 100,000,000 % | 1,584,000,000 pt | 128.00 | **128 pixels** |

Computed by taking the actual `f32` successor of each value, on a 1,584 pt
sheet. So the scroll offset stops being able to say where the operator is at
about a million percent, and by ten million the view moves in sixteen-pixel
jumps — it judders, then sticks.

**That is the whole justification for this module**, and it is worth
having in numbers because the first attempt at deriving it was wrong: an
earlier table divided by the zoom twice and concluded the error stayed
sub-pixel for ever, which would have made step 2 unnecessary. It is not. The
error is one content unit, and one content unit is one screen pixel, at
every zoom.

## What replaces it

[`DeepAnchor`] — **a page-space point in `f64`, plus where on screen it
sits**. The position stops being "how far the scroll area has scrolled" and
becomes "this point of the page is under that pixel of the window", which is
a statement whose precision does not decay with the zoom: `f64` carries 53
bits of mantissa, so a page coordinate stays exact to far beyond any zoom a
person will type.

It is the same shape the engine reached for the same problem — its own
commit says *"the fix is one subtraction moved into `f64`"*, and its region
renderer takes a page-space rectangle rather than a device offset. This is
that idea carried one layer up, into the shell's own position model.

## What this module deliberately does NOT do

It does not render, does not touch `egui`, and does not decide *when* the
deep path takes over — [`crate::render::strategy`] owns that question and
answers it from the pixmap ceiling. This is four numbers and the arithmetic
that keeps them consistent, which is what lets every claim above be a unit
test rather than something to be observed in a window.

## Item notes

### `fn a_page_point_survives_the_round_trip_at_every_depth`

The property the canvas depends on, and the one the `f32` scroll offset
loses: at 10,000,000 % the offset's representable step is sixteen screen
pixels, so a position could not survive this round trip at all.

The tolerance is in **page points**, scaled by the zoom — a tenth of a
screen pixel at whatever magnification is under test. A fixed page-space
tolerance would get easier as the zoom rises, which is backwards.

### `struct DeepAnchor`

# The invariant, stated first because everything here serves it

**The page point [`Self::page`] is drawn at the window point
[`Self::screen`].** Panning moves `page`; zooming leaves `page` and
`screen` alone and changes only the scale applied between them. That is why
a zoom about the cursor is expressible without any large intermediate: the
anchor is *already* the thing being held still.

# Why `f64` for the page and `f32` for the screen

They are different magnitudes doing different jobs. A page coordinate at
deep zoom is the value that needs precision — it is what a scroll offset was
failing to carry. A **screen** coordinate is bounded by the window, a few
thousand at most, where `f32` is exact to a small fraction of a pixel and
always will be.

Mixing them is deliberate rather than sloppy: making the screen point
`f64` too would imply the window can be large enough to need it, which is
the kind of false suggestion a type makes silently.

### `fn to_screen`

The forward half of the pair. Every large magnitude is subtracted
**inside `f64`** before the result is narrowed, which is the whole
technique: `point - self.page` is small even when both are billions, so
the product with `zoom` is small, and nothing large ever reaches `f32`.

### `fn to_page`

The exact inverse of [`Self::to_screen`], and the two are tested as
inverses rather than each against a hand-computed number — a pair that
round-trips is the property the canvas actually depends on.

### `fn panned`

The sign is the same convention [`crate::canvas::geometry::pan_offset`]
uses and for the same reason — dragging right moves the page right,
which means the page point under the cursor moves *left* in page space.
Getting this backwards produces a canvas that works and feels wrong,
which is harder to notice than one that is broken.

### `fn zoomed_about`

**This is the operation the whole module exists for.** In the scroll
-offset model, zooming about the cursor means solving for a new offset —
which is where the large magnitudes and their lost precision came from.
Here it is a re-statement: read which page point is under the cursor,
then declare that *that* point is now anchored there. No large number is
formed, so nothing is lost, at any zoom.
