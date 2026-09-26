# `render::prefetch` — filling in the pages he has not scrolled to yet

`OPERATOR_REQUESTS.md` O201:

> *"on multipage documents I noticed with scanned pdf I have to wait for
> pages to load as a I scroll to them. As many pages as we can should be
> rendered and ready to be shown as I scroll. The ones on screen should
> always take precedence to be rendered first."*

## Contract

[`OpenDoc::prefetch_candidate`] answers *"which page would render-ahead
like next, if anything"*, and is consulted by `settle::fill_strip` **only
when every visible page already has a picture**. [`disclose`] emits the
off-canvas report the feature owes. Nothing here spawns anything: the
caller owns the worker, so the single-slot rule and the zoom-settle
suppression stay in one place.

## Precedence is structural

His second sentence is not implemented as a priority number. The band is
reached only down the `None` arm of the visible scan, so there is no
ordering in which a page he can see loses to a page he cannot. A priority
comparison would have to be correct at every call site; an unreachable
branch is correct by construction.

## What bounds it

Three things, and they are independent on purpose:

* **[`PREFETCH_BAND`]** bounds how far a wrong guess about his direction
  can run, in pages.
* **[`OpenDoc::prefetch_headroom`]** bounds the memory, in texels, against
  the operator's own page-cache preference.
* **`OpenDoc::strip_page_orderable`** is applied unchanged, so a sheet
  above the renderer's pixmap ceiling is no more orderable ahead of time
  than it is on arrival.

## Item notes

### `const PREFETCH_BAND`

> *"on multipage documents I noticed with scanned pdf I have to wait for
> pages to load as a I scroll to them. As many pages as we can should be
> rendered and ready to be shown as I scroll."*

Eight is a bound on how far a *wrong* guess can run, not a target. The
band is symmetric about the current page (see [`prefetch_ranking`]), so
half of it is behind him, and behind him is where the rasters he has
already scrolled past are still resident. What eight really buys is eight
pages of lead on the direction he is actually going.

A count of PAGES rather than of texels because his request is in pages.
The memory is bounded separately and absolutely, by
[`OpenDoc::prefetch_headroom`].

### `const PREFETCH_SLOT`

A prefetched page renders exactly as a page he scrolled to: same request,
same raster, no marking of any kind. Rule 4 is satisfied by that, and what
it asks for in return is that the inference be **reported somewhere else**.
This is that report. It carries the resident band count and the texels
against the budget, because the one surprise render-ahead can spring is a
page-cache preference that used to sit idle and now fills — the
connection between *"pages are ready when I get to them"* and *"this
program is holding a gigabyte"* has to be visible rather than inferred.

Its own slot, per the rule at [`BEYOND_RASTER_SLOT`]: `trace_changed`
de-duplicates per slot, so two lines sharing one appear only when they
happen to alternate. `band=0` is printed too, on the way out of the regime
as well as into it, so a driven run can tell *"never prefetched anything"*
from *"prefetched and then stopped"*.

### `fn prefetch_ranking`

Pure, and separated from the cache and the page tree on purpose: the order
is the one thing here that can be silently wrong, and a wrong order does not
fail — it grinds. The tests at the foot of this file pin it.

# Why nearest-first is not a preference

[`crate::render::strip::StripRasters::retain`] evicts the entry **furthest**
from the current page. A prefetch offered in any other order therefore asks
for the page that eviction most wants to drop, and the two mechanisms
alternate — render, evict, render — for as long as he keeps scrolling. The
prefetch order has to be the exact reverse of the eviction order or the
feature is a busy loop that also costs memory.

# Why forward wins a tie, when nothing here measures a direction

Because the pages behind him are the ones already rendered. He arrived at
the current page by scrolling through them, so `current - 1` is nearly
always resident and `current + 1` nearly always is not — which means the
symmetric band spends almost all of its budget forwards without being told
to, and the tie-break only decides the first page of a document opened in
the middle. Measuring the scroll direction to reach the same result would
be two fields of state answering a question the cache already answers.

`last` is the highest page index in the document, so the band is clamped to
it rather than offering a page [`OpenDoc::rasterize`] would discard.

### `fn is_nearest_first`

Stated as a property rather than as a literal expected vector, because
it is the property `retain` is the mirror of — a future change that
reorders the band for some other good reason has to break this, not a
hand-written list that could be updated to match.

### `fn the_band_does_not_wrap_past_the_first_page`

Wrapping would be defensible in the abstract and is wrong here: page 0
and the last page of a hundred-page scan have nothing to do with each
other, and eviction — which this order mirrors — does not wrap either.
