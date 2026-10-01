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

### `const ROW_GAP`

**Scaled by the zoom**, like everything else here, which is what keeps the
strip's geometry exactly linear in the zoom and therefore makes the zoom
anchor's "measure, then re-place" solve exact rather than approximate. A
constant *screen* gap would be more conventional and would make
[`crate::canvas::geometry::zoom_anchor_offset`] wrong by the gap on every
step, in a way that accumulates over a long scroll.

12 points is about a sixth of an inch: wide enough that two white pages do
not read as one tall page, narrow enough that scrolling does not feel like
it is mostly gap.

### `const SPREAD_GAP`

Deliberately **smaller** than [`ROW_GAP`], and the asymmetry is the whole
point of a spread: the two halves are one opening and must read as a pair,
while the next spread is a separate thing. Equal gaps would produce a grid
of pages rather than a run of spreads.

### `struct RowMetrics`

The two facts about the **current row** that are needed *before* the strip
can be laid out, because both are inputs to
[`crate::viewer::ViewState::apply_fit`] and the strip's geometry depends on
the zoom that produces.

### `fn row_metrics`

**This exists to keep the frame to one O(n) pass rather than two.** The
fit modes need the row's extent and ceiling in order to decide the zoom, and
the strip needs the zoom in order to lay out — so a naive frame would build
a strip, read two numbers off it, apply the fit and build it again. On a
continuous strip over a large document that doubles the only per-frame cost
this feature has.

A row is at most two pages, so this is O(1) whatever the document's size,
and it produces the *same* two numbers [`Strip::row_extent`] and
[`Strip::row_max_zoom`] do — asserted by
[`tests::row_metrics_agrees_with_the_laid_out_strip`], because two
derivations of a fit scale is exactly how "Fit page" and the page it fits
come to disagree.

### `struct Placement`

`Copy` and two fields, for the same reason
[`crate::canvas::mapping::PageMapping`] is: it is a fact about one frame,
and a borrow that outlived the frame would describe a layout that has since
moved.

### `struct Strip`

Built once per frame in [`crate::canvas::show`], immediately from the page
vector and the view state, and then asked rather than re-derived. Every
question the canvas has about layout — how big is the scroll content, which
pages are visible, which page is the pointer over, which page is "current"
now that the operator has scrolled — is a method here, so that no two of
them can answer from different arithmetic.

### `fn new`

`current` is [`crate::viewer::ViewState::page_index`], which decides
*which* row a non-continuous mode lays out and, in every mode, which row
[`Self::row_extent`] and [`Self::row_max_zoom`] describe. It is clamped
into the document rather than trusted, because a page count that shrank
under a stale index is the same hazard
[`crate::viewer::clamp_page_index`] exists for.

An empty page vector yields an empty strip: `size()` is zero and every
query answers `None`. That is a legal document (`/Count 0`) and the
canvas already has a sentence for it, so failing here would replace a
message with a panic.

### `fn size`

For [`PageDisplay::Single`] this is exactly `extent × zoom` for the
current page, which is the expression `canvas::show` used before Phase 4
and the one every scroll, pan and anchor solve is written against.

### `fn visible`

`view` is the scroll viewport expressed in strip coordinates — i.e.
`Rect::from_min_size(offset, viewport)`. **This is the set that gets
rasterized**, and it is deliberately intersection rather than
containment: a page one pixel of which is on screen is a page the
operator can see, and drawing nothing there would be a visible hole at
every row boundary.

### `fn rect_of`

`None` is the ordinary answer under [`PageDisplay::Single`] for every
page but the current one, and callers rely on that: it is what makes
"draw the find highlights for this page" a no-op on a page that is not
being shown, without a mode check at the call site.

### `fn row_rect_of`

`OPERATOR_REQUESTS.md` O177, second half: *"fit page when in 2 pages
side by side views should fit the two side by side pages onto the canvas
- right now it snaps to fitting one."*

The defect this exists to close was an asymmetry, not an omission. The
fit's **scale** was already row-aware — [`Self::row_extent`] says so in
its own doc, *"fitting one page of a spread would leave the other half
off screen"* — while the fit's **placement** went on asking
[`Self::rect_of`] for the acting page. A spread was therefore scaled to
fit two pages and then centred as though it were one, which the operator
experiences as the fit being wrong rather than the centring being wrong.
The general lesson, worth carrying: **when a feature's scale rule learns
about a new layout unit and its placement rule does not, the symptom
presents as the scale being wrong.**

Returns `None` on exactly the same condition [`Self::rect_of`] does —
this strip does not lay `page` out — so a caller can fall back to the
page rect with a plain `unwrap_or` and get the pre-O177 behaviour rather
than a panic.

The union of the row's placements rather than a rect rebuilt from
`Row::{top, height, width}`: those are strip-space-at-zoom-1.0 and the
horizontal centring lives in [`Self::place_row`], so rebuilding here
would be a second copy of the placement arithmetic that could disagree
with the first. `reduce` over the placements cannot.

### `fn page_at`

`None` in the gaps between rows and in the centring margin either side
of a narrow page — which is the truthful answer, and the reason the
canvas asks this before deciding a click landed on anything.

### `fn page_at_view`

`GUI_ROADMAP.md` Phase 4.3's *"scroll-driven current-page tracking"*.
The rule is **the page most looked at wins**: each visible page scores
the larger of the share of itself that is visible and the share of the
view it fills, with the lowest page index breaking a tie. Each part matters:

* *Share, not raw area*: a postcard page seen whole above a Letter page
  is the page being looked at, though the Letter page covers more of the
  view. Raw area would make page commands act on the Letter page.
* *Visible share*, not "the page under the viewport centre": on a
  drawing sheet zoomed in past the viewport, the centre is always on
  some page and the two rules agree; on a document zoomed out far enough
  to show four pages, the centre rule flips the reported page as soon as
  the boundary crosses the middle, which reads as the page number
  twitching. Area is stable.
* *Lowest index on a tie*, so a spread reports its left-hand page and a
  boundary sitting exactly on the viewport edge does not oscillate
  between two answers on alternate frames.

`None` when nothing is visible at all, which happens for one frame after
a mode change before the scroll area has settled. The caller leaves the
current page where it was rather than guessing.

### `fn row_extent`

**A row, not a page**, and that is the only thing Phase 4 changes about
fitting. Under `Single` and `Continuous` a row is one page, so this is
[`crate::app::state::OpenDoc::current_extent`] and "Fit page" behaves
exactly as it did. Under a facing mode a row is the spread, and fitting
one page of a spread would leave the other half off screen — which is
not what a control called "Fit page" promises in a mode that shows two.

Falls back to a US Letter shape for an empty strip, for the same reason
`current_extent` does: the fit arithmetic needs something finite to
divide by, and nothing is drawn in that state anyway.

### `fn row_max_zoom`

The per-page raster ceiling, generalised to a row. It is the
**minimum** of [`crate::viewer::max_zoom_for_page`] over the row's
pages, and it is a minimum over *pages* rather than the ceiling of the
row's combined extent for a concrete reason: a spread is two pixmaps,
not one. Computing `max_zoom_for_page(row_extent)` would halve the
available zoom on a facing spread to guard an allocation nobody makes.

It deliberately does **not** consider the pages a continuous strip is
scrolled past. Those are separate pixmaps too, each guarded by its own
ceiling when it is rendered, and taking a minimum over a whole document
would let one Annex-C-sized sheet cap the zoom on the other 399 pages —
a control that stops working because of a page the operator cannot see.
The consequence, stated rather than hidden: scrolling a continuous strip
onto an enormous page while zoomed in past *its* ceiling shows that page
undrawn with the reason given, rather than silently zooming the whole
document out. See [`crate::render::strip`] for what the page says.

`pixels_per_point` and `quality` are both threaded through because the
ceiling is about **device pixels**, and the raster is made at the whole
of [`crate::viewer::raster_density`] — see `max_zoom_for_page`'s own docs
on why omitting either is how a guard passes its tests and fails on the
machine that matters.
