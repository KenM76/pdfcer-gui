# `rasterstrategy` — whole page, or just the window?

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
making. `app::settle::fill_strip` declines to order a strip page whose
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

### `const OVERSCAN`

This is the dial the operator's constraint turns on, and its cost is
quadratic, so it is written down rather than tuned by feel:

| overscan | pixels | pans that cost nothing |
|---|---|---|
| `0.0` | 1× | none — every pixel of movement crosses the edge |
| `0.5` | **4×** | **at least a quarter screen in every direction**, up to three quarters |
| `1.0` | 9× | at least half a screen in every direction |

`0.5` is the shipped value. At the zooms where the region tier engages the
viewport is a few hundred thousand pixels, so 4× of it is small in absolute
terms — which is the entire point of the region tier: **the raster stops
scaling with the zoom**, so a constant multiple of the window is affordable
where a constant multiple of the page would not be.

# The middle column is the budget; the right column is what SURVIVES
the snap

This table said *"up to half a screen in any direction"* against `0.5` until
2026-09-04, and that sentence was **false in two of the four directions** —
not because the overscan was not bought, but because [`region_for`]'s snap
spent it all on one side. The measurement and the operator report that
exposed it are in that function's header; the correction is recorded here
because this is the number a future reader will reach for when they want a
bigger margin, and reaching for it would have been the wrong fix.

The right-hand column is now a **guarantee at the worst grid phase**
rather than a best case, which is the only form of it worth writing down:
what the operator experiences is the worst side of the worst phase, because
that is the edge the fade appears along. The spread exists because the snap
quantises the window to half a viewport, so the margin varies between
`OVERSCAN − 0.25` and `OVERSCAN + 0.25` viewports on each side as the view
moves through the grid.

# What a bigger number would cost, priced rather than guessed

`BENCHMARK.md` carries the engine's own measurement of the region path on
the benchmark CAD sheet: **691 ms of fixed cost** (a one-by-one-*point*
region costs 691 ms — ~99 % of render cost there is area-independent) plus
roughly **0.19 µs per pixel**. On a ~1.3-megapixel viewport that puts one
region raster at ~1.6 s today, and moving this constant costs:

| overscan | region | raster time | guaranteed margin |
|---|---|---|---|
| `0.5` (shipped) | 4× viewport | ~1.6 s | 0.25 screens |
| `0.75` | 6.25× | ~2.1 s (**+0.5 s**) | 0.5 screens |
| `1.0` | 9× | ~2.8 s (**+1.2 s**) | 0.75 screens |

Which is why this is an operator decision and not a tuning exercise. He
has already ruled on this trade once — *"I don't want the affect that other
readers have where you always have to wait for detail to render after
panning"* — and both columns of that ruling move together: a wider margin
means fewer waits but each one is longer.

### `enum Ink`

`page_pts` is the page's own size in PDF points, longest edge first or not —
both are examined. `raster_scale` is device pixels per PDF point, which is
the operator's zoom already multiplied by the display scale.

# Why the pixmap ceiling is the switch, and not a zoom percentage

A zoom threshold would be wrong on exactly the documents this shell is for.
The whole-page raster fails when `page × scale` exceeds
`MAX_PIXMAP_EDGE` — so a small page survives to a far higher zoom than a
large one, and an A0 sheet reaches the ceiling while an A5 is still
comfortable. Switching on the thing that actually fails means the operator
keeps free panning for as long as it is physically available, on every page
size, without anybody choosing a number per document class.

It also means the switch **moves with the display scale**, which is
correct and would be easy to get wrong: `raster_scale` already includes
`pixels_per_point`, so a 150 % display reaches the ceiling at two-thirds the
zoom, exactly as it should.
**Whether this page is blended in ink, and at what ceiling** — the
second thing that ends the whole-page tier.

# Why the tier has two ceilings now

The operator, 2026-08-26: *"seems I get different results depending on Zoom
level … up to 474 % they are mismatched, but at 579 % they match."*

A page whose group declares a subtractive blending space (§11.4.7) is
composited in a four-colorant buffer at 20 bytes a pixel. Above a ceiling
the engine refuses that buffer and composites in sRGB instead — correctly,
and it says so — and the colours move, measured at up to 16 levels of 255.

That ceiling is **much lower than [`pdfcer_render::MAX_PIXMAP_EDGE`]**: on A4
the default is reached at about 518 % zoom against the edge ceiling's
1946 %, a factor of 3.76. Every whole-page raster in between comes back with
approximate colours — and a **region** raster of the same view does not,
because the buffer is sized to the region. So ending the whole-page tier at
whichever ceiling bites first is the repair, and it needs no new tier.

# Why it is OBSERVED and not assumed, which is the whole design

The obvious implementation applies the ink ceiling to every page. It would
be a serious regression, and the numbers say so plainly:

* the engine measured **13 of 51** files in the print-conformance suite and
  **15 of 4,012** in its external corpus as declaring a subtractive page
  group — about 0.4 % of real documents;
* on the operator's own D-size drawing sheet (1584 × 1224 pt) the default
  ceiling is crossed at **263 % zoom**, which is well inside the range he
  works in every day;
* and that sheet is line work with **no transparency on it at all** — it
  never asks for the buffer, so nothing would have been gained.

Applying the ink ceiling unconditionally would therefore have taken free
panning away from the operator's normal working zoom, on his own documents,
to fix a problem those documents do not have.

So the shell **learns**: `pdfcer-render` reports `cmyk_buffer_engaged` and
`cmyk_buffer_refused` on every raster, and either being non-zero means *this
page asked to be blended in ink*. `OpenDoc::absorb_render` records it, and
only a page that has been seen doing so gets [`Ink::Subtractive`]. A
document opens at a fit zoom and renders once before any zoom is possible,
so the observation is in hand before it can matter.

**The engine CAN now be asked directly, and this shell asks
first.** `interpret::page_blend_space` is still private, but `Pass 296.4`
made `pdfcer_render::page_composites_in_ink` public, and it is that same
function with the policy taken out of the `RenderOptions` handed in.
`OpenDoc::learn_ink` calls it once per page, so the answer is in hand from
the page dictionary rather than one raster later. The paragraph this
replaces described the request that produced it as still open.

**The observation below is KEPT, as a second observer that can only
agree.** The engine ships a test asserting the two match on every fixture,
so this is not a union of two opinions and must not be read as one — it is
one answer reachable by two routes, and the render route survives for the
pages that are rastered before anyone thinks to ask. What the counters
cannot supply is *which* of Table 147, the output intent or the device
decided it; that is `OpenDoc::ink_source`, it has one writer, and an
observed-only page correctly has no entry in it.

### `fn whole_page_raster_fits`

`MAX_PIXMAP_EDGE` and nothing else: no opinion about colour, no opinion
about speed. `true` means `pdfcer_render::render_page` will get past its own
size guard; `false` means it will return
`RenderError::BadRasterSize { width, height }` and the caller will have a
refusal to explain instead of a picture.


Because two different callers need two different questions answered, and
until O186 they were both asking [`for_page`] — which is the *union* of this
hard limit and the soft ink one.

* The **canvas** asks *"which tier should I use?"* It wants the union:
  dropping to a region is the right answer both when the whole page cannot
  be allocated and when it can but would lose its ink.
* The **strip** asks *"can I order this page at all?"* A strip page is
  handed `region: None` by construction — `OpenDoc::region_for` refuses a
  region for any page but the current one, deliberately, because a region is
  in one page's coordinate space — so for a strip page `Region` is not an
  alternative tier, it is *"there is nothing I can order"*.


> *"I think this sometimes results in similar error to 'This page could not
> be drawn. requested raster size 50411508x32619210 is empty or exceeds
> MAX_PIXMAP_EDGE'."*

`50411508 × 32619210` is **1224 × 792 pt at scale 41185.87**, and
`SW41177.pdf` has exactly two pages that size against thirty-four at
1584 × 1224. The failing raster was a **neighbour** sheet in the continuous
strip, ordered whole-page at the current page's deep scale, because
`app::settle::fill_strip` asked for every visible page without ever
asking whether the order could be filled.

And the union would have been the *wrong* predicate for the strip even so:
an ink page above the CMYK buffer ceiling but below the pixmap one answers
`Region` from [`for_page`] while its whole-page raster allocates perfectly
well. Skipping it would have left a neighbour sheet undrawn at an ordinary
zoom to avoid a failure that was never going to happen — trading a real
regression for an imaginary one.

# Degenerate input answers `true`

The same rule [`for_page`] applies, and for the same reason: a zero or
non-finite extent cannot be reasoned about, the whole-page path refuses it
safely with its own sentence, and answering "it does not fit" here would
instead make the strip silently skip a page whose real problem is something
else entirely.

### `fn region_raster_fits`

[`whole_page_raster_fits`]' twin, asked of an explicit rectangle in the
page's own user space rather than of the sheet's extent. Same ceiling, same
arithmetic, same degenerate-input rule — one definition of where the wall
is, because two would eventually disagree about it.

# Why a region needs asking at all

Two different rectangles arrive here wearing one type, and they behave
oppositely as the operator zooms:

| region | device size as zoom rises |
|---|---|
| [`crate::canvas::tier`]'s visible-rect tier | **constant** — the box is a multiple of the WINDOW, so the raster is the same size at 800 % and at 8,000,000 % |
| [`crate::render::halo`]'s off-page box | **grows with the zoom**, exactly as the whole sheet's does — and it is the bigger rectangle, so it hits this ceiling FIRST |

A caller that assumes "a region was chosen, therefore the order is small"
is right about the first row and wrong about the second. On the operator's
own site plan the halo box is 2,384 × 1,684 pt against a 1,191 × 842 pt
sheet, so it crosses `MAX_PIXMAP_EDGE` at about **half** the zoom the sheet
does. `OpenDoc::raster_order_fillable` is where that assumption was made
and is the caller this exists for.

# Degenerate input answers `true`

[`whole_page_raster_fits`]' rule, for its reason: a non-finite or inverted
box cannot be reasoned about, the render path refuses it safely with its
own sentence, and answering "it does not fit" here would make a caller
silently withhold an order whose real problem is something else.
