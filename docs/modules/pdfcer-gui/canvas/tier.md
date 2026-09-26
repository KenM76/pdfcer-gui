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

★ The order is a safety argument, not a preference: a halo is never smaller
than the crop box, so a page that cannot fit a whole-page raster certainly
cannot fit a halo. Reaching the second row at all means the whole page fits.

# What this module deliberately does NOT do

It does not build anything, and it does not touch a texture. In particular
it reads the page's content bounds through
[`OpenDoc::content_bounds_if_known`] and **never** builds them: a
decomposition measured **469 ms** on the operator's own CAD sheet and this
runs every frame. The build is `OpenDoc::ensure_content_bounds`, called from
`render::settle` after the picture has been asked for — so the halo appears
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

# ★★★ The operator's switch, and why BOTH gates are in this file


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
