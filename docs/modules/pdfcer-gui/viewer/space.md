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

## ★★ Why the second bridge inverts the renderer instead of deriving

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
