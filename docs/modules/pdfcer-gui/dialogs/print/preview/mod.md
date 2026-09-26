# `dialogs::print::preview` — a zoomable, pannable picture of the real sheet

## Why a picture rather than a number

pdfcer diverges from Acrobat here on purpose: Acrobat clips silently when
content falls outside the printable area, and pdfcer says so. That
divergence is worth nothing if the GUI reduces it to a count an operator can
look past. The whole reason `pdfcer-print` reads real device geometry
instead of guessing a bounding box is so this picture can be exact — drawing
only the SHEET and not the PRINTABLE AREA would show a page fitting that
will not.

**And the geometry is only half of it.** A preview that draws the correct
rectangles and fills the placed one flat answers *"where will the page
sit"* and never *"what is on it"*, which is the half an operator checking a
margin actually needs. The page content at (4) below is not decoration.

## What is drawn, outermost first

Four rectangles and one hatch, in this order, because each is *inside* the
previous and the nesting is the information:

1. **The sheet** — `DeviceGeometry::physical_pt`, the whole piece of paper.
2. **The printable area** — inset by the driver's own unprintable margins.
   This, not the sheet, is what constrains the job. A preview that showed
   only the sheet would show pages fitting that the hardware will crop.
3. **The placed page** — `Placement`'s offset and scale, applied *within*
   the printable area.
4. **The page's real content**, rendered through the same options the
   spooler uses (see [`super::render_options`]) and drawn into (3).
5. **A hatch over what will be lost** — over the part of the overhang that
   actually carries ink, and over nothing else. Hatched rather than filled:
   a hatch means *"this will happen and has not happened yet"*, which is
   exactly a pre-print clip. A solid fill reads as something already done.

## (5) is ink-aware — operator request O113

`Placement::clipped` is a *geometric* verdict: the page box exceeds the
printable rectangle. Hatching the whole overhang on the strength of it
shouts about losing something on every 1:1 CAD drawing while nothing is
being lost — *"the area that isn't printed is just empty border."*

**A disclosure that is technically true and practically false is the worst
kind.** An operator who sees the same red band on every drawing learns to
ignore it, and then does not see it on the one sheet where the border really
does have a title block in it.

The hatch now asks [`super::ink::InkMask`] — a downsample of the very raster
drawn at (4) — what is in the band, and covers the ink extent within it. No
ink ⇒ no hatch. [`hatch_lost_content`] holds the geometry, `super::ink`
holds the pixel test and the measurement behind its threshold, and
[`Overhang`] is how the caption is kept from contradicting the picture.

## The preview owns NO scroll area, deliberately

Zoom is Ctrl+wheel and pan is a primary-button drag. Neither competes with
the dialog's own [`egui::ScrollArea`]: per
`D:\dev\rag\egui\egui_0.35_zoom_with_keyboard_vs_app_zoom_chords.md` egui
splits wheel input at the input-state level, so a wheel event carrying the
zoom modifier surfaces as `zoom_delta()` and contributes nothing to
`smooth_scroll_delta()` — the two cannot fire from one gesture. A plain
wheel over the preview therefore belongs unambiguously to the dialog, and
there is no nested consumer to race it. **Scroll-to-pan was rejected for
exactly that reason**: it would have made the preview a scroll consumer
and put the question back.

## Colours: chrome for the diagram, pass-through for the page

Everything this file paints *except the page bitmap* is chrome — a diagram
of a piece of paper — and takes its colour from [`egui::Visuals`], the
same discipline [`crate::canvas::overlay`] states. The page bitmap is
**document content** and is drawn with a white multiplier, which
`painter.image` treats as "draw the pixels as rendered". Any palette role
there would mean restyling the application restyled the operator's page.

## Item notes

### `const STRIP_HEIGHT_PTS`

# It reserves THREE rows, not two

The strip is two rows — the seven controls, then the zoom caption — but the
first is `horizontal_wrapped` (see [`strip`] for why), so on a narrow column
it becomes two rows and the strip becomes three.

Reserving for the wrapped case is what keeps the constant honest. Reserving
two rows would let the strip overflow the column vertically the moment it
wrapped, the column's content would exceed the body, and a **vertical**
scrollbar would appear — the scrollbar defect this layout exists to remove,
re-entering by the other axis. The cost of the third row is about 28 pt of
canvas height on a wide column where it is not needed, against a scrollbar
that cannot be dismissed on a narrow one.

# FIXED, so the canvas can never be shrunk by its own caption

The canvas height is computed as `available − this constant`. Reading the
strip's ACTUAL laid-out height instead would reproduce a measured feedback
loop exactly: the clip caption wraps to two lines on a narrow window, the
strip grows, the canvas shrinks, the sheet is refitted smaller — and the
operator watches the preview settle over several frames for no reason they
can see. Subtracting a constant means the strip's content cannot reach the
canvas at all.

The window-resize coupling is the DESIRED one and is unaffected: a taller
window still means a taller canvas.

### `const CANVAS_MAX_HEIGHT_PTS`

A ceiling rather than a preference. `ui.available_height()` inside a
scroll area is a value this code does not own; clamping both ends means a
surprising answer from egui produces a preview that is merely the wrong
size rather than one that allocates a screen-sized rect.

### `const FIT_MARGIN`

Slightly under 1 so the sheet's own outline is not flush against the
canvas edge — the outline is load-bearing here (it is the paper), and an
outline touching the boundary reads as content continuing off-screen.

### `const TARGET_DPI`

Chosen against what the preview is FOR — checking that fine print clears
the unprintable margin — rather than against the size it is first drawn
at. At fit the bitmap is heavily downsampled, and that headroom is what
lets the operator zoom in and still see type rather than a mosaic. It is
deliberately NOT the job's own render DPI: the job renders at up to 2400
DPI and a preview does not need a 500 MB pixmap to answer a margin
question.

### `const MAX_SIDE_PX`

The DPI figure alone is not a bound: an ISO A0 sheet is 3370 pt on its
long side, which at 150 DPI is 7020 px and 190 MB of RGBA. This clamp
holds the worst case near 20 MB regardless of page size, which matters
because large-format CAD sheets are exactly the document population this
project's operator prints.

**It must sit ABOVE the office page sizes, and that is the whole trick.** A
ceiling meant for exotic sheets that falls below an ordinary one stops being
a ceiling and becomes the scale for every document, costing sharpness on the
common case to bound the rare one — silently, because a slightly softer
preview still looks like a preview. At 150 DPI the long sides are A4
1754 px, Letter 1650 and Legal 2100, so 2200 leaves all three at the full
target DPI and binds only where it is meant to.
[`a_letter_page_previews_at_the_target_resolution`] asserts that, which is
what keeps this constant and [`TARGET_DPI`] in step.

### `const ZOOM_MIN`

Bounded on BOTH sides because zoom is driven by a wheel: an unbounded
multiplier reached by a flick leaves the operator staring at one white
pixel with no way back except the Fit button they may not have found.

### `const PREVIEW_TEXTURE_ID`

**Distinct from `crate::render::raster`'s `PAGE_TEXTURE_ID`, and it has
to be.** egui reuses the allocation when the same name is loaded again, so
sharing a name with the canvas's page texture would make each surface
silently overwrite the other's pixels — the canvas would show the preview's
page at the preview's resolution, and neither would look broken enough to
investigate. That module's own header names the hazard and says a second
live page texture is what forces a per-texture id. This is that second one.

### `fn raster_scale`

# Two bounds, and the second one is the load-bearing one

[`TARGET_DPI`] alone would be a scale, not a bound: it says how finely to
render a point and says nothing about how many points there are. An ANSI E
sheet is 2448 × 3168 pt, which at 150 DPI is 5100 × 6600 px and 134 MB of
RGBA for a picture drawn 300 pt wide. [`MAX_SIDE_PX`] holds that near
15 MB, and it binds on exactly the large-format documents this project's
operator prints while leaving every office page size at full resolution.

The result depends only on the page's own size, so it is fully determined
by [`PreviewKey::page`] and does not need to be a key field of its own.

### `fn paint`

Returns the texture that was drawn — `None` when the page would not render,
which is the degraded-but-honest state described on [`texture_for`] — and
**what the overhang turned out to hold**, which is what the caption is then
written from. See [`Overhang`] for why the second half is returned rather
than recomputed.

### `fn strip`

They share a row because they are both "what am I looking at" controls and
because two rows plus the clip caption would not fit the fixed strip — and
the strip's height is fixed for the feedback-loop reason on
[`STRIP_HEIGHT_PTS`], so the layout has to live inside it rather than the
other way round.

### `fn zoom_by`

A two-field assignment over [`zoomed_view`], which holds the arithmetic and
is where it is tested — `PrintDialog` carries a spooler's worth of device
state that a test of the anchor term has no business constructing.

### `fn texture_for`

Returns the id rather than the handle so the caller holds no borrow of
`dialog` past the call — the alternative is a `&TextureHandle` living
across the rest of a function that also wants `dialog` mutably, which
compiles only by accident of statement ordering.

# Never re-rendered on a frame where nothing changed

Same discipline as the printer enumeration in
[`crate::dialogs::print::PrintDialog::open`] and as `RenderKey`'s
staleness fields: a preview that re-rasterised sixty times a second would
make an open dialog cost more than the print. [`PreviewKey`] carries every
input that can change the pixels; see its docs for why orientation is not
one of them, and for the rule a new rendering input lands under.

A failed render clears the cache and returns `None`, which drops the
preview back to the flat fill. **It is not reported as an error**: the same
failure will be reported honestly, once, by the spool attempt, and a
preview that turns into an error banner while the operator is still
choosing a page range is noise in front of a decision they have not made
yet.

### `fn upload`

# This is a SECOND premultiplied-alpha call site, and that is a defect
# this module cannot fix from here

[`crate::render::raster`] states the convention and says it is enforced by
there being **one** function rather than by review: both `ColorImage`
constructors accept premultiplied bytes without complaint, and the wrong one
silently darkens every antialiased glyph edge. This is a second one, and it
exists only because that module's public helper (`texture_from_pixels`)
takes a `RenderedPixels` — a worker result carrying a `RenderKey` — and
uploads under a *single fixed texture name* shared with the canvas. Neither
suits a preview, which has a pixmap and its own texture name.

⇒ **The fix is a `texture_from_pixmap(ctx, name, &pixmap)` in
`render/raster.rs` and the deletion of this function**, which is a change to
that module rather than to this one.

Until then the convention is held by this doc comment and by the assertion
in [`the_preview_upload_reads_pixels_as_premultiplied`], which is the same
fixture `render::raster`'s own test uses — so the two cannot drift without
one of them failing.
