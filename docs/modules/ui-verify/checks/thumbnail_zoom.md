# `thumbnails_zoom_without_blanking`

**Defect it guards.** The operator asked for page thumbnails that zoom, with a
quick picture first and a finer one after, and never a blank tile between
(`OPERATOR_REQUESTS.md` O289 item 18). Ways it goes wrong: the size cannot be
changed, the cache throws pictures away on a size change so tiles blank while
they re-render, or a wide tile only ever shows the small picture stretched.

**Fixture.** `fixtures/four-pages.pdf`, Edit mode, off the desktop. The dock's
default width gives one column of tiles about 208 pt wide, wider than the
140 pt draft.

**Steps.**

1. Wait for a `pages-tiles` census with visible tiles, `pending=0`, `blank=0`.
2. Page 1 must land `grade=draft` first and `grade=fine:<width>` after.
3. Press `panel-pages-zoom-out` twice: `panel-pages-tile.0` must become under
   0.7 of its width. Press `panel-pages-zoom-in` twice: it must grow past 1.3
   times that.
4. No `pages-tiles` census after step 3 began may count more blank tiles than
   the fixture has pages with no picture landed yet. Zooming out brings page 4
   into view for the first time; it is blank until its draft lands, which is a
   first fill, not a picture taken away.

**Falsified** in two ways:

- Clearing the held pictures in `ThumbnailCache::set_tile_width` when the
  width changes fails step 4: every visible tile counts blank.
- Making `wants_finer` return `false` fails step 2: page 1 lands only a draft.

**What it does not prove.** The fine picture's pixels: that is the renderer's
scale, set by `raster_scale_at`. Ctrl+wheel is the same `ThumbZoom::step`
and is not driven separately. At one column the largest sizes cannot widen the
tile past the dock; they matter in a wider dock.
