# `render::strategy` — whole page, or just the window?

`OPERATOR_REQUESTS.md` **O24**. One decision, made in one place, from
numbers rather than from a mode flag: **at this zoom, on this page, do we
rasterize the whole sheet or only what is on screen?**

## The constraint that shaped this, in the operator's words

> *"I don't want to lose our capability to pan around a page and still see
> high detail as we pan. I don't want the affect that other readers have
> where you always have to wait for detail to render after panning to a new
> area."*

He is describing the cost of region rendering, and he is right to refuse it
as a general answer:

| | whole page | region |
|---|---|---|
| rasterized once per | **zoom** | **position** |
| what a pan costs | nothing — the texture exists and the view moves over it | a new raster, every time |
| what he sees while panning | full detail, immediately | blur or blank until it lands |

Panning at full detail is a *property of rasterizing the whole page*, and it
is free precisely because the raster does not depend on where he is looking.
So region rendering may not simply replace it.

## The tiers, and why nothing is taken away to pay for anything

| tier | when | panning |
|---|---|---|
| [`Strategy::WholePage`] | while the page's raster fits `MAX_PIXMAP_EDGE` — **and, on a page blended in ink, while it still composites in ink** ([`Ink`]) | **free, full detail** |
| [`Strategy::Region`] | only above that | free within the overscan; a re-raster on leaving it |

**The tier he works in does not change at all.** On an A1 sheet the
whole-page raster survives to about 1,034 %, and today `MAX_ZOOM` stops him
at 800 % first — so every zoom he has ever used keeps exactly the behaviour
he has, *by construction rather than by tuning*. There is no low-zoom
performance question to answer here, because at low zoom this module returns
[`Strategy::WholePage`] and nothing downstream is different.

And the region tier only ever engages where the zoom is currently
**unavailable**. It cannot regress anything, because there is nothing there
to regress.

## What this module is NOT

It does not render, does not touch a cache, and does not know what a texture
is. It is arithmetic over four numbers, which is what lets the interesting
question — *where exactly does the switch happen, and does it move when the
window resizes?* — be answered by a unit test rather than by watching a
canvas.

## Item notes

### `fn every_zoom_the_shell_offers_today_still_rasterizes_the_whole_page`

`viewer::MAX_ZOOM` is 8.0 — 800 % — and at one device pixel per point an
A1 sheet's whole-page raster is comfortably inside the ceiling there. If
this ever fails, the shipped zoom range has started taking the region
path, and the panning behaviour he asked to keep has silently changed.

### `fn the_hard_ceiling_is_the_same_wall_for_page_finds`

Both directions, at the exact boundary, because the boundary is what the
strip's order gate reads: one ulp on the wrong side of it is either a
neighbour sheet blanked for nothing or the operator's
`MAX_PIXMAP_EDGE` message back.

### `fn an_ink_page_pushed_to_the_region_tier_still_fits_whole`

The regression guard for the mistake O186's fix was one keystroke from
making. `render::settle::fill_strip` declines to order a strip page whose
whole-sheet raster cannot be allocated; had it asked [`for_page`] instead
— the union of this hard limit and the soft ink one — then a page observed
compositing in ink, above the CMYK buffer ceiling but comfortably below
the pixmap one, would have been skipped at an ordinary zoom. Its raster
allocates perfectly well; it would merely have been flattened in RGB,
which is a colour compromise and not a failure.

That would have traded a real regression — a neighbour sheet blank at
300 % — for a failure that was never going to happen.

### `fn most_small_pans_reuse_the_raster`

The other half of the bound above: three is a **ceiling**, and a test
that only checked a ceiling would pass on an implementation that
returned three different rects for every pan. This asserts the floor —
that the snap is doing the job it exists for.

## Why a proportion of phases, and not one hand-picked base

*No* snapping implementation reuses the raster for *every* small pan:
a pan that crosses a grid line must change the rect, and roughly a fifth
of phases sit within a tenth of a viewport of a line. Naming one base
that happens to avoid one would be pinning that base's position in one
implementation's grid — the exact mistake the sibling test's header
records, where a fixture at `1000, 1000` stood in for a property for
months and then failed the moment the grid's offset moved by a quarter
cell without its cadence changing at all.

⇒ So the claim is made over the whole grid: **at least three quarters of
all view positions reuse the raster across a tenth-of-a-viewport pan.**
Measured at 80 % both before and after the centring change, because the
cadence is a property of the grid step and the centring moved only the
grid's offset.

### `fn the_region_raster_stays_window_sized_at_any_zoom`

The region's page-space size is the visible extent, which shrinks as the
zoom rises — so its device size is a constant multiple of the viewport at
every magnification.

### `fn the_same_view_always_asks_for_the_same_rectangle`

That is what makes a raster cache possible at all. A caller that grew
the rect itself, by a slightly different amount, would produce a cache
that never hits — every pan a miss, which is exactly the "wait for
detail" the operator refused.

### `fn margin_fraction`

Returned as a fraction rather than in points so the answer is the
same number at every zoom, which is the whole claim being made: the
region tier's margin is supposed to be a constant multiple of the
window.

### `fn the_overscan_reaches_a_quarter_screen_in_every_direction`

That constant's table says of the shipped `0.5`:

> | `0.5` | **4×** | at least a quarter screen in every direction |


> *"the canvas does a fading around the edges on stuff shown at the
> edges of the view. I don't want this. it should render true."*

## Why a sweep, and why over exactly one grid step

The margin is not a constant: it is a function of **where in the snap
grid the view happens to sit**, and that phase repeats every
`step = w/2`. A test at one position would sample one phase and could
pass on an implementation that starves a side at every other phase —
which is exactly what happened here, because every existing test of
this function checks its *size* or its *origin* and none checks the
gap between it and the view.

So the sweep walks a full step in 200 increments and takes the worst
case. Nothing about the page, the zoom or the viewport shape is special
to it; the phase is the whole variable.

## What a failure means

A margin below the threshold is not a slow raster — it is the operator
looking at **the low-resolution backdrop instead of the page** in a band
along the edge of the window (`canvas::backdrop`), because the sharp
region raster stops before the view does. That is the fade he reported,
and it is a rule 4 defect rather than a performance one: the same
content is being shown at two different fidelities at once.

### `fn the_region_stays_proportional_to_the_view_at_every_depth`

The snap divides an absolute page coordinate by the grid step. At a
trillion percent the step is about 2 × 10⁻⁸ pt and the coordinate is an
ordinary ~540, so the quotient is 2 × 10¹⁰ — past `f32`'s last exact
integer of 2²⁴, which made `.floor()` meaningless and floored the
region at **fifty thousand times** the size the viewport showed.

The raster was still produced and still traced `drawn=1`, so every
existing check passed while the operator saw a fraction of one texel
stretched across the window — blank paper.

Asserted as a RATIO against the visible extent rather than against
absolute sizes: what matters is that the rect stays proportional to
what is on screen, at every depth, and a test of fixed numbers would
have to be rewritten the next time `OVERSCAN` moves.

### `fn the_snapped_origin_stays_next_to_the_view_at_every_depth`

The size test above would pass on an implementation that returned a
correctly-sized rect somewhere else entirely — which is close to what
the `f32` version did, since a meaningless `.floor()` corrupts the
origin rather than the extent. Measured before the fix: the raster was
placed 18,998,834 window points from the viewport at a trillion
percent.

### `const A4`

Written out rather than reused from a fixture because the *label* is
the thing that went wrong once: every "A4" percentage in this project's
request and in the engine's first reply was computed on a 596 × 791 pt
page, which is neither A4 (595 × 842) nor US Letter (612 × 792). The
mechanism was right and the label was not, and it propagated for a day
through both repositories. This constant is the label, pinned.

### `fn an_additive_page_ignores_the_colour_ceiling_entirely`

The regression guard, and the reason [`Ink`] exists as a two-state value
instead of the ceiling simply being applied. About 0.4 % of real files
declare a subtractive page group; the other 99.6 % must reach exactly
the same tier at exactly the same zoom as they did before this argument
was added.

Asserted at a scale that is *far* past the ink ceiling and comfortably
under the pixmap one — 12×, where A4 wants 68 megapixels and the default
colour ceiling admits 13.4 — so a build that applied the ceiling
unconditionally cannot pass it.

### `fn a_page_blended_in_ink_leaves_the_whole_page_tier_at_the_colour_ceiling`

This is the whole repair: between the two ceilings, a whole-page raster
comes back with approximate colours and a region raster of the same view
does not, because the buffer is sized to the region.

The two scales are found by search rather than written down, for the
reason the label constant above records — a hardcoded 5.18 would be a
second copy of a measured limit, which is exactly what this project's
request to the engine refused to accept and what
`will_composite_in_cmyk` exists to prevent.

### `fn a_larger_ceiling_keeps_the_whole_page_tier_for_longer`

Without this, the Colour group's control could be wired to the renderer
(which `app::settings` asserts) and still change nothing about *when the
colours go approximate*, because the tier would switch first and hand
the operator a region raster before their larger buffer was ever asked
for.

Four ceilings, each a real quantity of memory, asserted as an ordering
rather than as four thresholds: a bigger allowance must never move the
switch DOWN, and the exact numbers belong to the engine.

### `fn an_absurdly_small_ceiling_falls_to_the_region_tier_rather_than_failing`

`Some(0)` is reachable: the settings field is uncapped in both
directions and `0` parses. It must produce a region raster rather than a
panic or a whole-page one — the region path is the one that copes,
because its buffer is sized to the window rather than to the page.

### `fn degenerate_input_is_refused_before_the_ink_ceiling_is_consulted`

The existing guard answers `WholePage` for a non-finite page or scale,
and it must keep doing so on a subtractive page: casting a NaN scale to
`u32` for the predicate would be undefined-ish rather than merely wrong,
and the early return is what makes it unreachable.
