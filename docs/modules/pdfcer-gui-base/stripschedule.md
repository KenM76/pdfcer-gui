# `pdfcer-gui-base/stripschedule`

Which pages the continuous strip may order at a raster scale, what it holds
for each, and how long a zoom must settle. `app::settle` schedules with these;
render-ahead is in [`prefetch`](stripschedule/prefetch.md).

## Item notes

### `fn zoom_settle`

**The operator's, as of 2026-08-17.** [`ZOOM_SETTLE`] was the whole
answer and is now only the *default* — `manifest::DIRECTED` carried this
as *"partial G — `ZOOM_SETTLE` is a compiled-in constant today"*, and
that was accurate: the control was missing, not the value.

Read from the document's preferences **snapshot** rather than from the
application, for the same reason its settings snapshot exists: this is a
per-frame read inside a `&mut doc` borrow, and reaching back to
`PdfcerApp` would be a second borrow of the whole struct.

The snapshot cannot be meaningfully stale here — `adopt_settings` writes
it and drops every raster in the same statement, so a settle read after
a change is a settle for a cache that no longer exists.

### `fn rehome_current_page`

Called when the scroll position has made a different page current. See
the module header, step 2: without this, every page of a continuous
strip would re-render at the moment it passed the middle of the
viewport — visibly flashing undrawn on the way through, which is the
opposite of what a continuous mode exists for.

`wanted` is the key the *current* page needs this frame. The incoming
page is taken out of the cache only if its raster matches that key,
because a raster at a stale zoom is not a raster the current page can
use — leaving it in the cache costs nothing and the ordinary staleness
path re-renders it.

The outgoing texture is filed at `self.edit_epoch`, and that is exact
rather than approximate: the current page's slot is cleared outright by
every edit (`crate::app::actions`' `vector_edit` and
`crate::panels::forms::edit` both assign `page_texture = None`), so a
texture that is still here has not survived an edit.

### `fn strip_page_orderable`

A strip page is always handed `region: None`: `OpenDoc::region_for`
refuses a region for any page but the current one, deliberately, because
a region is expressed in one page's own coordinate space and applying
page 4's rectangle to page 5 would rasterize the wrong part of the
neighbour with nothing reporting an error. So for a strip page the
renderer's whole-sheet pixmap ceiling is not a *tier boundary* — it is a
wall, and above it there is nothing to ask for.

# Why this exists as one function rather than two conditions

Three callers need the same answer and they must never disagree:

* [`Self::fill_strip`] uses it to **not place an order** it knows cannot
  be filled — the fix for the operator's
  `requested raster size 50411508x32619210` (see
  [`crate::render::strategy::whole_page_raster_fits`] for the full
  measurement, including why the failing sheet was never the one he was
  looking at);
* [`Self::strip_page_state`] uses it to say the **true** thing about the
  resulting empty page. If only the first caller existed, the page would
  report itself as `Waiting` — *"not drawn yet"* — for a picture that is
  never coming at this zoom. That is the wrong-refusal-sentence class of
  defect: the sentence is read as an answered question and nobody
  investigates.
* `render::prefetch` applies it to every band candidate, so a sheet
  that cannot be ordered on arrival is not ordered ahead of time
  either. Without it render-ahead would spend its whole budget
  re-offering the same unorderable A1 every frame.

# What it deliberately does NOT ask

`strategy::for_page`. That is the union of this hard limit and the soft
ink one, and an ink page above the CMYK buffer ceiling answers `Region`
from it while its whole-page raster allocates perfectly well — so asking
the union here would leave a neighbour sheet blank at an ordinary zoom
to avoid a failure that was never going to happen.

A page index past the end answers `false`: there is no sheet to order,
and [`Self::rasterize`] would discard the request anyway.

### `fn strip_page_state`

`None` means "there is a current raster for it" — the caller draws the
texture. Asked by the canvas while drawing, which is why it takes the
key rather than deriving one: the canvas already knows this frame's
raster scale and deriving a second one here is how the drawn page and
the requested page come to disagree.
