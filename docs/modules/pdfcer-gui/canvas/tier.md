# `pdfcer-gui/canvas/tier`

**Which picture this page needs** — the raster-tier decision, and nothing
else.

# Why this is its own module

Split out of `canvas::present` on 2026-09-10, when adding O23's halo tier
put that file at 1,503 lines and the R2 gate — *no source file over 1,500
lines* — refused it. The gate's own text says what to do about that: *"split
the module along its seams — one subject per file — rather than raising the
limit"*. This is the seam, and it was a seam before the gate said so.

Everything else in `present` is about **drawing and handling** what the
frame already has. This is the one block that decides what to **ask the
renderer for**, and it answers a question with a single, checkable shape:

> *Given where the operator is looking and where this page's ink reaches,
> is the whole page the right thing to rasterize?*

It has exactly one output — [`OpenDoc::raster_region`] — and
`OpenDoc::region_for` feeds that same value into **both** the cache key and
the engine request. So setting it here is sufficient to change what is asked
for and what counts as stale, and nothing downstream has to be told.

# The three tiers, in the order they are tried

| tier | when | what is rasterized |
|---|---|---|
| **region** | the whole-page pixmap would exceed the engine's ceiling, or would blend too much ink | the visible rectangle, whose device size is a multiple of the WINDOW and so constant at every zoom |
| **halo** | the page fits, but its ink reaches past the sheet | the union of the crop box and the content box |
| **whole** | otherwise | the crop box, as it always was |

The order is a safety argument, not a preference: a halo is never smaller
than the crop box, so a page that cannot fit a whole-page raster certainly
cannot fit a halo. Reaching the second row at all means the whole page fits.

# What this module deliberately does NOT do

It does not build anything, and it does not touch a texture. In particular
it reads the page's content bounds through
[`OpenDoc::content_bounds_if_known`] and **never** builds them: a
decomposition measured **469 ms** on the operator's own CAD sheet and this
runs every frame. The build is `OpenDoc::ensure_content_bounds`, called from
`app::settle` after the picture has been asked for — so the halo appears
a frame after the page rather than every edit stalling for half a second.

# Its trace

One line per change, through `canvas::trace::halo`:

```text
pdfcer-diag canvas-halo tier=halo known=true offpage=on box=-160.000,0.000,200.000,200.000
```

`known=` is the part that makes a failure readable: `tier=whole known=false`
is *"nobody has decomposed this page yet"*, which is normal on the first
frame; `tier=whole known=true` is *"the ink was measured and no halo was
asked for"*, which on a page with off-page content is a defect.
`ui-verify`'s `off_page_visible` reads exactly this distinction.

`offpage=` is the third state and it was added because the other two
could not express it — see below.

# The operator's switch, and why BOTH gates are in this file


1. **See and reach** — [`decide`] substitutes `None` for the content
   bounds, which puts every page back on the whole/region tiers and makes
   [`crate::render::halo::reach`] a no-op. Nothing off the sheet is drawn,
   so nothing off the sheet can be clicked.
2. **Room** — [`overhang`] returns zero, so the pasteboard does not grow
   to hold material that is no longer drawn.

The second is the one the operator asked for by name: *"when not showing
the stuff that is off page there shouldn't be a gap between pages where
the stuff is"*. Gating only the first would leave the band of empty grey
exactly where it was and hide the thing that justified it — the worst of
the two states. **A layout cost with no visible cause is worse than the
feature it pays for.**

## Item notes

### `fn decide`

Set for the **current page only**. A region is expressed in one page's own
coordinate space, and `OpenDoc::region_for` refuses it for any other page
rather than rasterizing the wrong part of a neighbour.

# Arguments

* `layout` — where every page in this view sits, in strip space.
* `current` — the page being acted on. Passed rather than re-read from
  `doc.view.page_index` so this function and its caller cannot disagree
  about which page the frame is about.
* `raster_scale` — device pixels per point, as the draw loop will key on.
  Both ceiling questions below are asked at this scale.
* `deep` — whether the view is at tier 3, where the scroll offset is no
  longer the position and the visible rectangle must come from the anchor.
* `visible_rect` / `avail` — what of the strip is on screen, and the size of
  the viewport, in strip space.

### `fn overhang`

# Why this lives here and not at its call site


* `present.rs` had eight lines of R2 headroom and this is thirteen.
* **This is the same decision `decide` makes**, expressed against the
  layout instead of against the raster. Both answer *"does this page reach
  past its sheet, and is the operator asking to be shown it?"*, and a
  switch whose two halves live in two files is a switch that will one day
  be half-flipped. Adjacent functions in one module is the cheapest
  arrangement in which that cannot happen quietly.

# The contract

Written to [`crate::app::state::OpenDoc::pasteboard_overhang`] by the
caller, once per frame, and read from there by nine places. **Two spellings
of the pasteboard is precisely the defect O23 spent three attempts on** —
see that field's own documentation.

`content_bounds_if_known` PEEKS and never builds: a decomposition costs
469 ms on the operator's benchmark sheet, and this runs every frame. The
honest consequence is that on the first frame after opening a large drawing
the pasteboard is the plain one, and one frame later it is the wider one —
the same one-frame lag the halo raster has, for the same reason.

Multiplied by the zoom HERE, because the overhang is a fact about the
drawing (canvas points) while every `geometry` term is in screen points.
`halo::overhang` resolves `/Rotate` through `PageFrame::canvas_box_of`,
which is O174's single place for it. A non-finite or non-positive zoom
yields zero rather than a NaN that would propagate into every scroll bound.
