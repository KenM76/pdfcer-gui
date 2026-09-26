# `render::region` — turning "what is on screen" into "what to rasterize"


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
