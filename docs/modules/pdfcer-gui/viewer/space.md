# `viewer::space` — the coordinate spaces a canvas gesture crosses

Two bridges, four functions, and one rule: **every conversion between the
screen, the page raster and an authoring API goes through here.** A second
formula anywhere else is a second thing to keep in sync with the renderer.

## The three spaces, named because they are easy to conflate

- **Screen space** — egui points in the window, what a pointer event
  carries.
- **Canvas space** — page-device points at zoom 1.0: Y-**down**, origin
  top-left, `/Rotate` already resolved into a possibly-swapped
  width/height. This is the space [`super::page_extent_pts`] measures and
  the space the on-screen raster is drawn in. [`screen_to_page`] and
  [`page_to_screen`] cross between screen and canvas space, and they carry
  **no rotation logic** — rotation is already baked into the `extent` they
  are handed.
- **PDF user space** — Y-**up**, origin at the *un-rotated*
  MediaBox/CropBox lower-left: exactly what an annotation `/Rect`, a
  content-stream operand or the object model expresses.
  [`canvas_to_pdf_space`] and [`pdf_space_to_canvas`] cross between canvas
  and user space.

## Why the second bridge inverts the renderer instead of deriving

The canvas⟷user pair reuses — and inverts — the **same** device transform
`pdfcer_render::page_device_geometry` computes to rasterize the page, so
the interaction geometry and the render agree by construction. Two
hand-derived rotation formulas would agree on the day they were written
and drift the first time either side learned about a new `/Rotate` case.

## The failure contract, and why it differs between the two bridges

The screen⟷canvas pair answers [`egui::Pos2::ZERO`] for a degenerate page
or zoom, matching `fit_scale` and `clamp_zoom`: there is no sensible canvas
coordinate for a page with no area, and a painter asking for one every
frame must not get a NaN. The canvas⟷user pair answers `None`, because its
callers are **authoring** — a commit declined is right where a commit of
garbage geometry is not.

## Item notes

### `fn screen_to_page`

`image_rect` is the canvas Response's own `.rect` for this frame
(the rect the page raster occupies on screen); `extent` is
[`super::page_extent_pts`] for the current page (the rotated device
width/height); `zoom` is [`super::ViewState::zoom`]. The page raster is drawn
at `image_rect.min` scaled by `zoom`, so undoing that — subtract the
origin, divide by the zoom — is the whole of the arithmetic.

**No rotation branch lives here on purpose.** Rotation-correctness comes
entirely from `extent` already carrying the rotated width/height (see
[`super::page_extent_pts`]); adding a rotation-aware branch here as well would
double-apply it. The `extent` argument is consulted only to reject a
degenerate page (per the contract below) — the mapping itself is a pure
affine undo of the draw.

Returns [`Pos2::ZERO`] for a degenerate page or zoom (zero/negative/
non-finite `extent` or `zoom`), mirroring [`super::fit_scale`]/[`super::clamp_zoom`]'s
"fail to a finite, harmless value, never a NaN/panic" discipline: there
is no sensible canvas coordinate for a page with no area.

### `fn page_to_screen`

Needed every frame by any live-preview overlay (a stored canvas-space
geometry must be projected back to the screen to be drawn) and, from
stage S4, to draw a hit-tested object's selection outline. Same
degenerate-input contract as [`screen_to_page`].

### `fn canvas_to_pdf_space`

Implemented by inverting the SAME transform
[`pdfcer_render::page_device_geometry`] computes to rasterize this page
at scale 1.0 (its third tuple element, a
[`pdfcer_render::tiny_skia::Transform`]). Canvas space *is* that
transform's output space at scale 1.0, so its inverse is exactly the
canvas→user map, rotation and Y-flip included, with no second formula to
keep in sync (the geometry analogue of "reuse the renderer's own walk so
they agree by construction").

Returns `None` only for a genuinely non-invertible page transform (a
degenerate page). Callers decline the commit rather than author garbage
geometry.

### `fn pdf_space_to_canvas`

Needed by any consumer that receives geometry already in PDF space — the
primary case being the object-model provider handing back a hit-tested
object's bounds in PDF space, which the selection overlay must project to
the screen via `page_to_screen(pdf_space_to_canvas(bounds, page), ..)`.
Returns `None` under the same non-invertible-page condition as
[`canvas_to_pdf_space`], so the two bridges decline together.
