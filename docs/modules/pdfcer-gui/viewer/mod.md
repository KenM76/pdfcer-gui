# viewer — the page-view state machine and its geometry

**Salvaged whole from the old GUI's `viewer.rs`** — its documentation and
its entire test suite carried across rather than lifted as snippets,
because a snippet leaves the reasoning behind and the next engineer
re-derives a decision that was already paid for. What came across is the
zoom ladder with provable reversibility, the fit modes re-derived per
frame, and the per-page raster ceiling that accounts for
`pixels_per_point`; each is argued below.

Two of the geometry functions here carry `#[allow(dead_code, reason = …)]`
because their first consumer has not landed yet. They are kept rather than
deleted because each is the *pair* of a live function, and **a bridge with
only one direction implemented is how the two ends drift apart.**

**A page *range* is not a field here, and its absence is the design.** A
range is not something a view *holds*: which pages are on screen falls out
of where the pages are laid out and where the viewport is. So [`strip`]
computes it and [`ViewState`] keeps exactly one index, meaning *"the page
the operator is looking at"* — read off the scroll position under a
continuous mode, set by navigation under a paged one. See
[`strip::Strip::page_at_view`].

The view's fourth axis is [`ViewState::display`]: which of the four
arrangements is active. It is orthogonal to the three below and is
documented in [`display`].

**Anchoring a zoom is not this module's question.** It is a question about
the *scroll offset* rather than about the ladder, so the rule and the solve
live in [`crate::canvas::zoom`] and [`crate::canvas::geometry`]. Two things
here are reused by them rather than reimplemented, and that reuse is the
point:

* [`fit_scale`] under [`FitMode::Page`] computes the scale that frames a
  **region** for zoom-to-selection and marquee-zoom, exactly as it computes
  the scale that frames a page. One derivation, so a region zoom and a page
  fit cannot disagree about what "fits" means;
* [`max_zoom_for_page`] and [`clamp_zoom`] apply the per-page raster
  ceiling to a framing zoom. A marquee dragged around a bolt head asks for
  a scale no page-sized pixmap can supply, and the answer is the same
  answer the zoom buttons give — stop at the ceiling, and let the status
  bar's readout state the scale that was actually pinned.

---

Everything about "which page am I looking at, and how big is it on
screen" lives here, deliberately separated from the egui widget code.
The split exists for one concrete reason: **this module is unit-testable
and the widget code is not.** A windowed UI cannot be exercised
headlessly on a CI runner, but zoom-ladder arithmetic, fit-scale
derivation, page-index clamping and the raster-size ceiling are exactly
the parts where an off-by-one or a divide-by-zero would show up as a
user-visible bug — so they are pure functions with tests, and the widget
code is reduced to wiring.

## The view model

[`ViewState`] carries three things:

- `page_index` — 0-based into the flattened page vector from
  [`pdfcer_core::page_tree::pages`]. The UI displays it 1-based; the
  conversion happens once, in the string catalog.
- `zoom` — the **effective** scale in device pixels per PDF user-space
  unit, which is precisely the `scale` argument
  [`pdfcer_render::render_page`] takes. `1.0` is 72 DPI, i.e. "actual
  size" on a nominal 72-point-per-inch display.
- `fit` — whether `zoom` is a value the operator pinned
  ([`FitMode::None`]) or one derived from the viewport each frame
  ([`FitMode::Page`] / [`FitMode::Width`]). This is a *mode*, not a
  one-shot action: "Fit page" that stops fitting the moment the window
  is resized is the behaviour every viewer gets right and would be
  conspicuous to get wrong.

## Why the zoom ladder is a table, not a multiplier

Repeatedly multiplying by, say, √2 produces zoom levels like 141%,
199%, 281% — technically fine, but the operator can never get back to
a round number, and two different click sequences that "should" land
on 100% land on 99.6% and 100.4% instead. A fixed ladder of familiar
percentages ([`ZOOM_LADDER`]) makes zoom-in/zoom-out exactly
reversible and always lands somewhere nameable. Zoom values *off* the
ladder (from ctrl+scroll, or from a fit mode) are handled by taking
the next rung strictly above/below the current value, so the ladder
also acts as a "snap back to sanity" mechanism.

## The raster-size ceiling is a real constraint, not a formality

`pdfcer-render` refuses to allocate a pixmap with an edge over
[`pdfcer_render::MAX_PIXMAP_EDGE`] (16,384 px — the allocation guard). A
letter page never comes close, but ISO 32000-1 Annex C permits pages up
to 14,400 units on an edge, and such a page hits the ceiling at about
1.1× zoom. Rather than let the operator zoom into an error message,
[`max_zoom_for_page`] lowers the ceiling per page and [`ViewState`]
clamps against it — the zoom buttons simply stop, which is
self-explanatory in a way that "requested raster size 115200x86400 is
empty or exceeds MAX_PIXMAP_EDGE" is not.

## Item notes

### `fn default`

Fit-page rather than 100% is a deliberate choice. Opening at a
raw 100% produces a wildly different first impression depending
on the page size — a business card fills a thumb's worth of the
window, an A0 poster overflows it — and both read as a bug even
though nothing is wrong. Fit-page always shows the operator the
thing they just opened.

**Single page rather than continuous**, for the reason
[`display`]'s header states at length: continuous is an option, not a
replacement, and paging one sheet at a time is the right model for
drafting review. Read mode's continuous default is applied by the open
path (which knows the mode and the document), not by this `Default` —
so a `ViewState` built with no context is the conservative one.

**All three View ▸ Display toggles start off**, and that is not
timidity. A ruler, a grid and a set of guides are all chrome drawn over
or beside the drawing, and pdfcer's first duty on opening a sheet is to
show the sheet. Defaulting the rulers on would also take
`THICKNESS_PTS` off two edges of every canvas for every operator who
never wanted them, which is the one default that has a measurable cost
(see the field's own docs and rule R128).

`guides` is the one that is *overridden* at open — by
[`crate::app::state::OpenDoc::new`], when the document turns out to
have remembered guides. That override lives there rather than here for
the same reason Read mode's continuous default does: a `ViewState`
built with no context is the conservative one, and the path that knows
the document is the path that may know better.

### `fn the_readout_survives_the_whole_configured_range`

A `u32` return here saturates, and a saturated `as u32` reads as
**4294967295%** on the status bar: `u32::MAX` presented as a
measurement, at a zoom the ladder genuinely reaches.

Asserted against the FORMATTED string, because that is the artefact an
operator reads. A test of the numeric value passes on a build that
narrows the type further downstream, which is where such a defect
actually lives.

### `fn a_zoom_survives_a_round_trip_through_a_raster_scale`

The defect this pins is not that either function was wrong. Each was
right about what it claimed; they simply did not agree, because the
forward direction multiplied by the quality and all four backward
readings divided by the display density alone. On the operator's own
build — `render_quality = sharper` — every derived ceiling was therefore
half again too high, the engine refused the pixmap, and the clamp that
exists to rescue him landed by the same factor too high and refused
again.

The repair is structural rather than arithmetic: both directions now
run through [`raster_density`], so this test cannot be made to fail by
changing one of them. That is the point of it — it is here to fail if
somebody re-opens the two into separate expressions.

### `fn a_nonsense_density_is_the_identity_in_both_directions`

`sane_pixels_per_point` lives inside [`raster_density`], so the guard is
stated once and both directions inherit it. Were it applied in the
forward direction only, a `NaN` density would rasterize at `zoom` and
convert back through a division by `NaN` — a ceiling of `NaN`, which
compares false against everything and switches a clamp off silently.

### `fn the_page_ceiling_moves_with_the_render_quality`

Sharper rasterizes at 1.5×, so the zoom at which a page fills
[`pdfcer_render::MAX_PIXMAP_EDGE`] is two-thirds of what it is at Normal.
The old [`max_zoom_for_page`] returned the same number for all three
qualities, so on Sharper it offered a zoom whose raster the engine
refuses — which is the sentence the operator reported reading across his
drawing.
