# `panels::pages::zoom` — thumbnail size

The Pages panel's tiles zoom in and out (`OPERATOR_REQUESTS.md` O289 item 18),
the way Acrobat's page thumbnails do.

## Contract

- `ThumbZoom` is a factor on the narrowest tile the grid lays out
  (`MIN_TILE_WIDTH_PTS`), stepped through `STEPS` (0.5× to 4×, default 1×).
  The grid fits as many columns of at least that width as the dock holds
  (`columns_at`), then shares the width among them, so a larger factor means
  fewer, wider tiles. At 4× a default-width dock holds one column.
- Two buttons beside the previews row (regions `panel-pages-zoom-out`,
  `panel-pages-zoom-in`), greyed at either end with the reason on hover, and
  Ctrl+wheel while the pointer is over the grid. The grid takes the gesture
  only when hovered, so it never reaches the canvas as well.
- A change traces `pages-zoom factor=`.

## Never blank: the two passes

The size is a layout change, and pictures are cached per page, not per size.
`thumbnails::ThumbnailCache` holds each page's picture with its `Grade`:

1. **Draft**: `THUMBNAIL_WIDTH_PTS` wide, rendered first for every visible
   page that has no picture or a stale one. Quick on any page.
2. **Fine**: once no visible tile is waiting, each visible draft is
   re-rendered at the width the tile is drawn (`set_tile_width`, rounded up to
   `FINE_STEP_PTS` so a splitter drag does not request a render per frame).
   It is only wanted when the tile is wider than the draft.

Throughout, a tile draws whatever picture it holds stretched to its rect, so
the old picture stays until the new one replaces it. A fine render that fails
or runs past the per-page time limit leaves the picture there and records the
width as refused for that page's revision (`wants_finer`); it is not retried
at that width or wider until the page changes or the time limit is raised.
A page edited after its fine picture was drawn is redrawn directly at the fine
grade, so a tile does not drop to the draft and climb back.

Fine pictures cost memory in the square of the width. Those off screen are
capped at `MAX_OFFSCREEN_FINE`, furthest first; a visible one is never
evicted for it. An evicted page comes back as a draft when scrolled to.

## Trace

- `pages-thumbnail ... grade=draft|fine:<width>` on each landing.
- `pages-tiles ... fine=<n> ... blank=<n>`: the visible census. `blank` is the
  never-blank invariant's oracle: a zoom must not raise it from zero.
