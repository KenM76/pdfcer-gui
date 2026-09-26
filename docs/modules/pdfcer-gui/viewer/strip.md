# `viewer::strip` — where every page sits, in one coordinate space

`GUI_ROADMAP.md` Phase 4.1 asks for *"`ViewState` holds a page **range**
rather than one `page_index`"*. This module is that range, expressed as
geometry rather than as a pair of indices — because "which pages are on
screen" is not a fact the view *holds*, it is a fact that falls out of
where the pages are and where the viewport is.

## The one space, and its relationship to the three that already exist

[`crate::canvas::mapping`]'s header names three frames: **screen**,
**canvas** (page-device points, Y-down, `/Rotate` applied) and **PDF user**
(Y-up, un-rotated). This module introduces exactly one more, and names it
so it cannot be confused with canvas space:

| frame | origin | unit | who speaks it |
|---|---|---|---|
| **strip** | the top-left of the whole laid-out run of pages | logical points **at the current zoom** | this module, the scroll area's content box |
| **canvas** | one page's top-left | PDF user-space units (zoom 1.0) | `PageMapping`, the provider, the raster |

A [`Placement`] is the bridge: `rect` is where one page sits **in strip
space**, already multiplied by the zoom, which is exactly the rect
`PageMapping::new` wants as its `image_rect` once the strip's own screen
origin has been added. Nothing downstream ever converts between strip and
canvas space by hand; it asks for a `Placement` and builds a mapping from
it.

## ★ Single page is a one-row strip, and that is load-bearing

The operator's constraint is that continuous scroll is *an option, not a
replacement* — see [`super::display`]'s header. The way that constraint is
honoured mechanically rather than by care is that
[`PageDisplay::Single`] produces a strip whose row set is **the current
page alone**, so:

* `size()` is the page's drawn size — identical to the `display_size` the
  canvas computed before Phase 4;
* the page's rect is `(0,0)..size` — so the scroll range, the centring
  margin, the pan clamp and the zoom anchor are all the arithmetic they
  already were;
* there is no row gap, because there is one row.

[`tests::single_page_reproduces_the_pre_phase_4_geometry_exactly`] asserts
that as an equality against the old expression rather than as a comment. A
change that gives `Single` a gap, a margin or a scroll range it did not have
is a test failure, which is the only way a "do not degrade the default"
instruction survives contact with a refactor.

## Cost, and why there is no cache

[`Strip::new`] is one pass over the rows it lays out, with one `Vec`
allocation. For [`PageDisplay::Single`] and [`PageDisplay::Facing`] that is
one row — a two-element allocation, per frame, which is free. For the
continuous modes it is one row per page (or per spread) in the **whole
document**, because a scroll range is a property of the whole document and
cannot be computed from a window of it.

That is O(n) per frame, and it is deliberately not cached. The measured
shape of the work is a `page_device_geometry` call and about a dozen float
operations per row; a cache would need a key over (page vector, display
mode, zoom) held behind a `RefCell` on `OpenDoc`, invalidated on every zoom
notch — i.e. rebuilt on exactly the frames that are already the expensive
ones. See the module's own measurement note in `BENCHMARK.md` terms: the
rasterization this feature schedules is measured in **hundreds of
milliseconds**, and this is measured in **microseconds**. Optimising the
second while the first exists would be optimising the wrong thing, and the
cache would be a staleness hazard bought with nothing.

**The honest limit, stated rather than discovered:** strip coordinates are
`f32`, and a strip is as tall as the document. At the top of the zoom ladder
(8×) a 1,000-page US Letter document is ~6.4 million points tall, where
`f32` resolves to about 0.5 pt — sub-pixel, but no longer exact. Beyond
roughly 2,000 pages at maximum zoom the accumulated row tops begin to
quantise visibly. The fix, if it is ever wanted, is `f64` row tops converted
to `f32` per placement, and this paragraph is where that decision would
attach. It is not made now because the error at any realistic combination is
far below one pixel and because carrying two float widths through the
geometry has its own cost in confusion.
