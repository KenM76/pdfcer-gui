# `rasterregion` — turning "what is on screen" into "what to rasterize"


> *"I got a requested raster size 14580x18868 is empty or exceeds
> MAX_PIXMAP_EDGE when I got to 2382% zoom."*

A US Letter page at 2382 % is 18,868 device pixels tall against a 16,384
cap. The whole-page raster cannot be made, and the answer he proposed is the
right one: *"reducing the raster sized area to around the cursor zoomed area
to what will fit"*.

## The two conversions, and why they are here rather than in the canvas

| | |
|---|---|
| [`page_region`] | the visible part of a page, in the **PDF's** coordinates, ready for `render_page_region` |
| [`region_on_screen`] | where that rectangle's raster belongs on screen |

They are exact inverses of each other and are the only place this shell
crosses between screen space and PDF space for a *raster*. Keeping them
together is what lets the round trip be a unit test — and a round trip is
precisely the property that matters, because getting one of the two slightly
wrong produces a page that is drawn *almost* in the right place, which reads
as a rendering bug rather than as a coordinate one.

## The y flip, which is the part that goes wrong

Canvas space is y-**down** from the page's top-left; PDF user space is
y-**up** from its bottom-left. `render_page_region` documents its rectangle
as *"page space, pre-scale — the same coordinate system as `Page::crop_box`,
y-up"*, so the flip happens here, once, in both directions.

A flip that is applied twice is the identity, and a flip that is missed
shows the operator the *opposite end* of the page from the one they are
pointing at — which at 2382 % is a uniform field of whatever happens to be
there, and looks exactly like a blank raster.

## `/Rotate` — O174, and why a y flip alone was never enough


> *"this pdf causes problems zooming past about 1600% — the view appears to
> jump to another location and when I pan back to where something is visible
> it appears to be distorted."*

That file is a single 792 × 1224 pt page carrying **`/Rotate 270`** (set by
an incremental update; the original object says `/Rotate 0`, which is why a
naive grep of the file finds both). Every fixture in this repository and
every page in the engine's own region tests is `/Rotate 0`, so until his
file arrived nothing had ever exercised this path on a turned page.

### The two coordinate systems this module bridges, precisely

| space | origin | y | `/Rotate` |
|---|---|---|---|
| **canvas** — what the canvas lays out, hit-tests and draws in | page's top-left **after** turning | down | already **resolved**; a 270°-turned 792 × 1224 page is 1224 × 792 here |
| **PDF user** — what `render_page_region` takes | `CropBox` lower-left | up | **not applied**; the rect is still 792 × 1224, and the engine turns it itself |

The original implementation converted between them with one subtraction —
`height − y` — which is exactly right for `/Rotate 0` **on a crop box whose
origin is (0, 0)**, and silently wrong for every other page. On his sheet
the shell asked the engine for a rectangle whose axes were swapped and
mirrored relative to the one the operator was looking at:

- the engine rasterized **a different part of the drawing** → *"the view
  appears to jump to another location"*;
- and that raster's width and height were **swapped**, while
  [`region_on_screen`] computed a destination rect from the un-swapped
  canvas rect, so the texture was stretched into the wrong aspect →
  *"it appears to be distorted"*.

Both symptoms, from one missing transform, appearing exactly at the zoom
where `render::strategy` hands over from the whole-page tier to this one —
`MAX_PIXMAP_EDGE / 1224 pt ≈ 13.4×`, i.e. about **1,340 %** on his page at a
100 % display, which is the *"about 1600%"* he reported.

### Why the existing round-trip test could not see it

`a_region_maps_to_screen_and_back_to_itself` asserts that [`page_region`]
and [`region_on_screen`] are inverses **of each other**. They were — both
made the same wrong assumption, so the round trip closed perfectly while
the rectangle handed to the engine pointed somewhere else entirely. An
oracle built out of both halves of the system under test agrees with itself
by construction.

The calibration that *can* see it is
[`tests::the_engine_rasterizes_the_rectangle_the_canvas_asked_for`]: it
pushes the region this module produces through the **engine's own**
`region_base_geometry_of` and asserts the device rectangle that comes back
is the canvas rectangle we started from. That is a measurement against the
other side of the boundary, and it fails on `/Rotate 90`, `180` and `270`
before this fix.

## Item notes

### `fn canvas_to_user`

The exact inverse of the coefficient table in
`pdfcer_render::region_base_geometry_of`, which at `scale = 1` maps user
space onto the same canvas space this shell lays pages out in:

| `/Rotate` | user → canvas | canvas → user (this function) |
|---|---|---|
| 0 (and any other value) | `cx = x − llx`, `cy = ury − y` | `x = llx + cx`, `y = ury − cy` |
| 90 | `cx = y − lly`, `cy = x − llx` | `x = llx + cy`, `y = lly + cx` |
| 180 | `cx = urx − x`, `cy = y − lly` | `x = urx − cx`, `y = lly + cy` |
| 270 | `cx = ury − y`, `cy = urx − x` | `x = urx − cy`, `y = ury − cx` |

Note that 90 and 270 **swap the axes**: the canvas's x comes from the
PDF's y. That is the whole of O174 — a conversion that only subtracted
in y could never produce it, however carefully the subtraction was
written.

### `fn the_pages_own_box_fills_canvas_space_exactly`

# Why this is the test that matters

Canvas space is defined twice over, and the two definitions have to be
the same rectangle:

* by [`PageFrame::user_to_canvas`], which says where a *point* of the
  page lands; and
* by [`PageFrame::extent_pts`], which says how *big* the page is, and
  therefore how big the rect the shell lays out for it is.


So the assertion is not "the extent is 2383.937". It is that the two
definitions agree, on every rotation, on an offset crop box, and on a
box with a fraction — which is a property, not a number, and would have
caught the defect on the day it was written.

### `fn a_fractional_sheet_reports_its_fraction_not_the_pixmaps_ceiling`

The literal regression pin for the defect above. Separate from the
property test because a future change that broke the property in the
*other* direction — making the conversion agree with a ceiled extent —
would satisfy the property and still be wrong: the engine's region
origin is in page-device space, which is canvas space times the scale
with no rounding at all, so the exact crop extent is the one both sides
of the boundary already use.

### `fn an_inverted_crop_box_measures_zero_rather_than_negative`

Built by struct literal, **not** by `Rect::from_corners`, which
normalises: a crop box arrives here as `pdfcer_core` parsed it, and a
file whose `/CropBox` has its corners the wrong way round is exactly
the case worth pinning. Writing the test through the normalising
constructor would have asserted that `from_corners` works.

### `fn the_engine_rasterizes_the_rectangle_the_canvas_asked_for`

# Why this test and not another round trip

`a_region_maps_to_screen_and_back_to_itself` below asserts that
[`page_region`] and [`region_on_screen`] are inverses of each other, and
it passed for the whole life of the region tier while his page was being
rasterized in the wrong place. Both halves shared one wrong assumption,
so they agreed perfectly. **An oracle assembled from two pieces of the
system under test measures their agreement, not their correctness.**

The independent oracle is `pdfcer_render::region_base_geometry_of`: the
engine's own user-space → device-space mapping, the very function
`render_page_region` uses to decide which pixels to make. Push a canvas
rectangle out through [`page_region`] and back in through that, and the
device rectangle that comes out must be the canvas rectangle we started
from. Anything else means the operator is being shown a different part
of his drawing from the one he is pointing at.

Asserted at `scale = 1`, where device space **is** canvas space. The
engine's `x0`/`y0` are the region's left and top edges in page-device
space, and `width`/`height` its size there, so the comparison needs no
arithmetic of its own — which is the point, since arithmetic in a test
is one more place to make the same mistake twice.

### `fn the_region_and_its_screen_rect_are_inverses_on_every_rotation`

Kept alongside the calibration above rather than replaced by it: they
answer different questions, and the pair of them is what says both that
the right pixels are made and that they are put in the right place.

### `fn a_turned_pages_region_lands_inside_its_crop_box`

The cheapest statement of O174 and the one that needs no engine call: a
window in the middle of the canvas must map to a rectangle inside the
**crop box**, which on his sheet is 792 wide and 1224 tall. The pre-fix
arithmetic produced `llx` up to 1224 on a page only 792 wide — a
rectangle off the side of the sheet, which is why he saw blank paper.

### `fn looking_at_the_top_of_the_page_asks_for_the_pdf_top`

Looking at the TOP of the page must ask for the page's HIGH y in PDF
space. A missed flip shows the opposite end of the sheet, which at deep
zoom is a uniform field and reads as a blank raster rather than as a
coordinate error.

### `fn the_sharp_raster_covers_the_window_on_every_side`

> *"the canvas does a fading around the edges on stuff shown at the
> edges of the view. I don't want this. it should render true."*

## What this asserts, and why it is the composed chain rather than one
function

[`super::strategy::region_for`] has its own test of this property in
page points, and it is the tighter one. This is the same claim made
**where the operator makes it — in screen pixels, about the rectangle
the texture is actually painted at** — and it therefore has to go
through every conversion `canvas::present` goes through:

| step | what it produces |
|---|---|
| the viewport, in the page's own points | what `present` derives from `visible_rect ∩ place` |
| [`page_region`] (which calls `region_for`) | the PDF-space rect that will be rasterized |
| [`region_on_screen`] | `paint_rect` — where that raster lands |

The y flip lives in the middle of that chain and is the reason this
test is worth writing separately. `region_for` snaps in canvas space,
y-**down**; `page_region` then flips to PDF space, y-**up**; and
`region_on_screen` flips back. A margin that is generous on the snapped
low side and starved on the high side comes out of that pair of flips
attached to a *different screen edge* than the page-space test names, and
only a test that composes all three can say which edge of the operator's
window is the starved one.

Since O174 the chain also crosses a rotation, so this runs on **every**
frame rather than on an upright Letter page alone: a starved edge that
depended on the axis swap would otherwise be invisible here.

## What a failure looks like on his screen

`paint_rect` is where `canvas::present` draws the sharp texture;
`canvas::backdrop` has already painted the low-resolution whole-page
texture underneath, across the page's *whole* rect. So every screen pixel
inside the window but outside `paint_rect` is showing **the blurry
stand-in instead of the page**. A margin of zero on a side means that
band opens along that edge of the window on the first pixel of a pan and
stays open for the ~1.6 s a region raster takes on a CAD sheet.

### `fn the_deep_placement_is_exact_at_zooms_where_f32_is_not`

At four billion percent the page's own screen rect has a magnitude of
~10^12 px, where `f32`'s spacing is 131,072 px — coarser than the whole
window. The anchor-based path never forms that number, so the rect it
returns is correct to a fraction of a pixel.

Asserted by placing the anchor ON the region's own canvas-space corner:
the answer must then be the viewport origin exactly, at any zoom. The
anchor is seeded through [`PageFrame::canvas_box_of`] rather than by
hand, because after O174 "the region's corner in canvas space" is a
rotation away from its `llx`/`ury` and a hand-written seed would only be
right for the upright case — which is the whole class of mistake this
module was just corrected for.

### `struct PageFrame`

# Why a struct rather than two arguments

Because they are only meaningful together, and because the pair is what
makes a *page* — the same reason `pdfcer_render::RegionGeometry` is a struct
rather than five positional values. A caller that passed a crop box and
forgot the rotation would get the pre-O174 behaviour back, compiling
cleanly, on the exact pages where it is wrong.

# The crop box is narrowed through `f32` on the way in

`pdfcer_render::region_base_geometry_of` does this — deliberately, with a
comment saying why: the whole-page path truncates the crop box to `f32`
before it multiplies, and computing from the `f64` box instead lands on a
different pixel for some pages, which broke the engine's own poster-tiling
reassembly test. This module's job is to be the **exact inverse** of that
function, so it must start from the same numbers. Narrowing here rather
than at each use keeps that a property of the type instead of a rule
someone has to remember four times.

### `fn crop`

Exposed so [`super::halo::region`] — and `canvas::present`, which
calls it — union the content box against the *same* numbers
[`Self::canvas_box_of`] maps against. Reading `page.crop_box` there
instead would be a second source for one value, and the two differ by
up to an `f32` ulp — enough for a halo that is exactly the crop box to
come back as a one-ulp overhang and flip a whole document into a bigger
raster for nothing.

### `fn canvas_box_of`

A bounding box of the two mapped corners, not a corner-by-corner
copy: under 90° and 270° the axes swap and under 180° both mirror, so
the mapped "lower-left" is not the canvas's top-left. Every rotation
here is a multiple of 90°, so the box of the two opposite corners is
the exact image of the rectangle — no rotation-of-a-rotated-rect
inflation is possible.

Visible to the rest of `render` rather than private, because
[`super::halo::reach`] needs the same mapping for the content bounding
box and O174 is exactly the class of defect that a second hand-written
copy reproduces. Deliberately **not** `pub`: PDF-user-space geometry is
this module's subject, and a caller outside `render` that wants canvas
coordinates wants [`region_on_screen`] instead.

### `fn region_on_screen`

`page_screen` is the rect the page occupies on screen — what the whole-page
texture would have filled. The returned rect is the sub-rectangle of it that
`region` covers, and it is routinely **larger than the screen and partly
negative**, because the region carries overscan beyond the viewport. That is
correct and must not be clamped: the texture covers that area, and clamping
the destination without cropping the source would stretch the image.

`page_pts` is the page's **canvas** extent as
[`crate::viewer::page_extent_pts`] reports it — rotation already applied,
and rounded the way the engine rounds its pixmap. It is what `page_screen`
was laid out from, so the two scales are derived from it and not from
`frame`'s crop box; a page whose extent rounded is then still placed exactly
on itself.
