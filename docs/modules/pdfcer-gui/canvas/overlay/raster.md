# `pdfcer-gui/canvas/overlay/raster`

## Item notes

### `const RASTER_GHOST_ALPHA`

High enough that the lettering reads as lettering rather than as a smudge,
low enough that whatever it is passing over stays visible — which is the
whole reason the copy is translucent: the operator is choosing where to put
the chunk by looking at what is already there.

### `fn draw_raster_ghost`

`OPERATOR_REQUESTS.md` O215 ask 5 — *"the chunk follows the pointer, not a
rectangle."* [`super::draw_move_ghost`] is the floor and says WHERE the set will
land; this says WHAT will land there, which for a line of text is the only
answer the operator can read off the screen.

# Where the pixels come from, and why not from the engine

The page's raster is already on the GPU, already at this zoom, and already
a picture of the chunk — so the travelling copy is a blit of a
sub-rectangle of it. No re-render, no decomposition and no engine call on
any frame of the gesture, which is the same affordability argument
[`super::draw_move_ghost`] makes and has to hold for the same reason: on the CAD
sheets pdfcer exists for, one raster is tens of milliseconds and a drag is
sixty frames a second.

⚠ **The UV is taken relative to [`crate::canvas::strip::PageView::paint_rect`], never to
the page's own rect.** See that field: the two differ for as long as a new
region's raster is in flight, and deriving the UV from the page rect
samples the wrong part of the picture at exactly the zooms the region tier
exists for.

A source that reaches outside the rastered part is **cropped**, and the
destination is cropped by the same amount so the copy keeps its scale.
Clamping the destination alone would stretch the image, which is a subtler
wrong than a missing corner — the same choice `canvas::present` makes about
the paint rect itself.

# Rule 4

A pre-commit affordance on exactly the same footing as the outline ghost:
it is the cursor, it exists only while the button is down, and nothing that
has already been applied is styled. The content underneath renders exactly
as it will render saved and reopened; the copy is drawn over it and both
are gone on release. The translucency is what says *copy* rather than
*content*, and it is how Word, PowerPoint and Acrobat each draw a drag in
flight.

### `fn blit_of`

Returns `None` when nothing of `from` lies over the raster, which is a
chunk scrolled off the painted region rather than an error.

⚠ **`source` is the rectangle the texture is a picture OF, and the UV is
taken relative to it, never to the page rect.** The two differ for as long
as a new region's raster is in flight, so deriving the UV from the page
rect samples the wrong pixels at exactly the zooms the region tier exists
for.

⚠ **Both rectangles are cropped by the same amount.** Clamping the
destination alone would stretch the picture to fill it, which is a subtler
wrong than a missing corner: the lettering would travel at the wrong size
and nothing about it would look like clipping.

### `fn raster_ghost_is_owed`

One condition, and it is the second half of [`super::ghost_is_owed`]: nothing
better is already on screen. Where a shape preview carries the real anchors
travelling, the operator can already watch the geometry move, and a blitted
copy of the same thing doubles it.

It is deliberately NOT `outline`-sensitive, and that is the whole
difference from [`super::ghost_is_owed`]. The outline ghost is owed at the object
rung even while the geometry travels, because the box states the SET; a
second picture of content that is already visibly moving states nothing.

⚠ The empty-preview trap applies here identically — see
[`super::ghost_is_owed`].
