# `viewer::deep` — where the view is, when the scroll offset can no longer say

`OPERATOR_REQUESTS.md` **O24**, step 2. The operator:

> *"how do we get to the insanely high limit? … You should be able to have a
> new algorithm take over for bigger zooms?"*

He is right that something has to take over, and this is it — but it is not
about how pixels are made. **It is about where the viewport's position is
stored.**

## ★★★ The measurement that makes this necessary

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

★ **That is the whole justification for this module**, and it is worth
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

★ It is the same shape the engine reached for the same problem — its own
commit says *"the fix is one subtraction moved into `f64`"*, and its region
renderer takes a page-space rectangle rather than a device offset. This is
that idea carried one layer up, into the shell's own position model.

## What this module deliberately does NOT do

It does not render, does not touch `egui`, and does not decide *when* the
deep path takes over — [`crate::render::strategy`] owns that question and
answers it from the pixmap ceiling. This is four numbers and the arithmetic
that keeps them consistent, which is what lets every claim above be a unit
test rather than something to be observed in a window.
