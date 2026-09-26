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

## Single page is a one-row strip, and that is load-bearing

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

## Item notes

### `fn place_row`

The single owner of the placement arithmetic — a row is centred
horizontally in the strip, and each page is centred vertically in its
row so a short page in a tall spread does not hang from the top edge.

### `fn single_page_reproduces_the_pre_phase_4_geometry_exactly`

The operator's constraint, as an equality rather than an intention:
continuous is *an option, not a replacement*, and the way that survives
a refactor is that the single-page strip's size **is** the expression
`canvas::show` used before this feature existed — `extent × zoom`, with
the page at the origin and no gap anywhere.

A change that gives `Single` a row gap, a margin or a scroll range it
did not have fails here.

### `fn strip_geometry_is_exactly_linear_in_the_zoom`

The property the zoom anchor's "measure, then re-place" solve rests on:
[`crate::canvas::geometry::zoom_anchor_offset`] holds a *fraction* of
the content still across a zoom step, which is only correct if doubling
the zoom doubles every coordinate. A constant screen gap would break it
by the gap, cumulatively, down a long strip.

### `fn the_current_page_follows_the_scroll_by_greatest_visible_area`

Phase 4.3. Asserted as the behaviour an operator sees — scroll down a
page and the reported page becomes the next one — rather than as the
arithmetic, and with the boundary case that would make a centre-based
rule twitch.

### `fn row_metrics_agrees_with_the_laid_out_strip`

[`row_metrics`] exists so a frame costs one O(n) pass rather than two,
and the price of that shortcut is a second derivation of two numbers a
fit mode depends on. Two derivations of a fit scale is exactly how "Fit
page" and the page it fits come to disagree, so the equality is
asserted over every mode, a mixed-size document and both spread
parities rather than left to review.

### `fn a_continuous_fit_does_not_depend_on_the_current_page`

A mixed-size document, asked for its fit metrics from two different
current pages. Under a continuous mode the answer must be the SAME —
because the current page is derived from the scroll, so an answer that
varies with it closes a loop between zoom and scroll position.

Written as an equality between two calls rather than as an assertion
about a particular scale, because the bug is not "the zoom is wrong".
Either zoom was individually defensible; the defect is that the two
disagreed, so the frame's answer depended on the previous frame's.

### `fn a_continuous_fit_frames_the_largest_extent_in_each_axis`

Scroll-independence alone would be satisfied by always fitting page 0,
which is stable and wrong: a later, larger sheet would overflow a
control called "Fit page". The per-axis maxima matter for the same
reason — with a portrait and a landscape sheet in one document neither
row is both the widest and the tallest, so fitting either whole row
would leave the other overflowing on one axis.

### `fn a_uniform_document_fits_exactly_as_it_did_before`

The property that makes this fix free in the common case: every row is
one page and every page is the same, so `fit_metrics` and `row_metrics`
agree exactly. Without it, the fix would be a silent behaviour change
for every ordinary document rather than a repair of a broken one.

### `fn facing_continuous_fits_the_spread_not_the_cover`

This assertion was written the other way round first — as "a uniform
document is unaffected in every continuous mode" — and it failed,
correctly. Under a facing mode **row 0 is a cover**: one page, while
every row after it is a two-page spread. So on eight identical Letter
pages the rows are genuinely 612 pt and 1,230 pt wide, and they are not
interchangeable.

Fitting the cover would therefore make every spread in the document
overflow — from a control called "Fit page", on the second row. The
old per-row behaviour did exactly that whenever the operator's scroll
happened to leave page 0 current, which is another face of the same
bug rather than a separate one.

### `fn a_paged_mode_still_fits_the_current_row`

They show one row at a time and the operator chose it, so there is no
loop — and fitting the document's largest sheet there would shrink
every other page for no reason. The fix must not leak into them.
