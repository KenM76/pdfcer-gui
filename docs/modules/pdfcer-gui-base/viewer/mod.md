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

### `mod ladder`

A scroll offset is `f32` into a content space of `page × zoom`, and one
unit of that space is one screen pixel — so at 1,000,000 % the offset can
only address every other pixel, and at 10,000,000 % it moves in
**sixteen-pixel jumps**. Its header carries the measured table.

`DeepAnchor` replaces it with a page point in `f64` plus where on screen
that point sits, which is a statement whose precision does not decay with
the zoom.
**How far this page can actually be zoomed** — the three limits that
bind at three different depths, reconciled in one place.

Its header carries which is which: the raster ceiling stops mattering
once the region tier engages, the `f32` scroll offset's is what the shell
can honestly offer today, and the operator's setting is the third.
**The zoom levels the `+` and `−` buttons step through**, and the rule
for what happens past the last named rung.

Split out under R2. Its header carries the one property that matters: the
two steps must be exact inverses, **above** the ladder as well as on it —
and above it is the half that is easy to get wrong (O24g).

### `struct ZoomAnchor`

Lives here rather than on `crate::app::state` — where it was declared
until the rulers landed — because it is a fact about **zoom**, and this
module already owns [`ViewState::zoom`], [`FitMode`], [`ZOOM_LADDER`],
[`MAX_ZOOM`] and [`raster_scale`]. `app::state` re-exports it, so
`canvas::zoom` still names it by its old path and the move cost that
module nothing. See `app::state`'s re-export for the R2 argument that
prompted it.

Recorded on the frame the wheel is seen, consumed on the next one, so
the scroll offset can be moved to keep that point still. See
[`crate::canvas::geometry::zoom_anchor_offset`].

**It has to span two frames**, and that is not an implementation
detail: the new zoom is not known when the wheel is seen. The zoom is an
[`crate::app::actions::Action`] applied after the UI is built, and it
*clamps* — so the only honest source of "how big is the page now" is the
next frame's own display size. Recording the *inputs* and solving later
avoids predicting a clamp we do not control.

### `struct ViewState`

## `PartialEq` is here for one test, and it is the right one

Derived for [`crate::app::prefs::Prefs::seed_view`], whose contract is
*"seeding from the shipped preferences leaves a freshly opened view
untouched"*. That property is only assertable as **whole-struct
equality**: checking the fields the seeder writes would pass while a fifth
field was silently clobbered, and checking the fields it does not write
requires listing them, which is the same restatement drifting in a second
place.

Deriving it over an `f32` is deliberate rather than overlooked. This struct
is a *record of choices* — a zoom that was set, not a zoom that was computed
— so two states that arrived at 1.0 by different routes genuinely are the
same state. The float-comparison caution applies to accumulated arithmetic,
and there is none here.

### `const OCR_OVERLAY_DEFAULT`

Not `1.0` and not `0.0`. Either stop shows exactly one of the two things
the mode exists to let an operator compare, so arriving at one would make
the first gesture *find the slider* rather than *read the page*. Two thirds
puts the text clearly on top with the scan still legible beneath it, which
is the position an operator checking a recognition against the paper
actually wants.

### `fn normalise_ocr_overlay`

The guard is `is_finite` **before** the clamp, not after: `f32::clamp`
propagates a NaN rather than rejecting it, so a NaN that reached the veil's
alpha would paint an undefined rectangle over the page. The same ordering
`crate::app::prefs::normalise_ui_scale` uses, for the same reason.

### `fn stroke_display`

# Why the conversion is a named function and not an `if` at the call
site

There are two call sites and they must not be able to disagree: the
**render key** ([`crate::app::state::OpenDoc::render_key_for`]) says
*what picture I want*, and the **render request** (built next to it, read
by `crate::render::worker::render_on_worker`) says *what picture this
is*. Two hand-written `if`s is exactly how a cache comes to serve a
raster drawn under the opposite answer — the failure mode that makes a
toggle look inert, which is the defect O137 reports about the button
this replaces.

# Why the return type is the engine's ENUM and not a `bool`

`StrokeDisplay` is `#[non_exhaustive]` with two variants today —
`Actual` and `Hairline` — and the engine made it an enum deliberately so
that Acrobat's *enhance thin lines* (the **opposite** convention: thin
things made thick) can arrive as a third variant. A `hairline: bool`
anywhere in this shell would, that day, come to mean *"one of the two"*.
So the boolean stops here and the engine's vocabulary starts here.

`Hairline` is the **off** position. `true` means faithful widths; see
the field.

### `fn go_to_page`

Clamping rather than erroring is right for a *view*: the only
ways to get an out-of-range index are a keyboard repeat past the
end and a page count that shrank, and in both cases the operator
wants the nearest valid page, not a message.

### `fn next_page`

Saturating rather than wrapping: wrap-around page navigation
silently teleports an operator from page 400 to page 1, which is
disorienting and is not what any document reader does.

### `fn set_zoom`

`max` is the per-page ceiling from [`max_zoom_for_page`], passed
in rather than recomputed so this stays a pure state transition
with no page argument.

### `fn clamp_page_index`

Returning `0` for an empty document rather than panicking keeps the
"no pages" condition a *presentation* decision (the canvas shows
[`crate::text::canvas_no_pages`]) instead of a crash, which matters
because a valid PDF really can have `/Count 0`.

### `fn clamp_zoom`

NaN is reachable in practice: a degenerate page whose CropBox has
zero width makes `viewport_width / page_width` infinite or NaN, and
an unclamped NaN would propagate into the render scale and then into
a pixmap size, where it becomes a much less obvious failure. Mapping
it to actual size fails visibly and harmlessly.

### `fn sane_pixels_per_point`

Returns `pixels_per_point` when it is a usable density and `1.0` otherwise.

# Why this is a named function rather than a `.max()` at each site

Four places divide or multiply by the display density — [`raster_scale`],
[`ceiling::zoom_ceiling`]'s learned clause, `app::settle`'s
`learn_raster_ceiling`, and `app::status::rasterstop` — and they are not free
to guard it differently, because they are three readings of *one* number
(`crate::render::ceiling::RasterCeiling`'s stored raster scale) and a
disagreement between them is a shell that clamps at one zoom and explains
itself at another.

The tempting spelling is `pixels_per_point.max(f32::MIN_POSITIVE)`, and it
is **wrong in the one case that matters**. `f32::max` returns the *other*
operand when one is `NaN`, so a `NaN` density becomes `f32::MIN_POSITIVE` —
and a division by it produces infinity, which is the most destructive
possible answer rather than a conservative one. The consequence is not
abstract: the sentence that explains a zoom limit would be switched off
permanently and silently, in exactly the state it exists for. The `NaN` row
of this function's unit test is what holds the guard to it.

`1.0` is the right fallback because it is the *identity*: a scale and a zoom
are the same number at unit density, so a caller that cannot learn the
density falls back to treating the two as interchangeable, which is what the
shell did for its whole life before HiDPI was handled at all.

### `fn raster_scale`

`zoom` is points per PDF user-space unit — what the operator sees as
a percentage and what fit modes compute. The raster has to be made in
*pixels*, so it is multiplied by the display's `pixels_per_point`.
Getting this wrong is not a crash; it is a viewer that looks
permanently slightly blurry on every HiDPI laptop and perfectly sharp
on the developer's external monitor.

### `fn raster_density`

# Why this is a function and not two multiplications

A raster scale is `zoom × pixels_per_point × quality.multiplier()`, and four
places in the shell need to run that conversion **backwards**:
[`max_zoom_for_page`], [`ceiling::zoom_ceiling`]'s learned clause,
`app::settle::absorb`'s `learn_raster_ceiling`, and `app::status::rasterstop`.
Every one of them divided by the density alone, and the quality factor was
simply absent — so on View ▸ Render ▸ Quality ≥ Normal the derived ceiling
asked the engine for a pixmap over [`pdfcer_render::MAX_PIXMAP_EDGE`], the
engine refused, and the clamp that exists to rescue the operator landed by
the same factor too high and refused again. O218.

Routing both directions through this one function is what makes
[`zoom_for_raster_scale`] the *exact* inverse of [`raster_scale`] rather than
a second reading of the same rule that has to be kept in step by hand.

# What is in it

* `pixels_per_point`, through [`sane_pixels_per_point`], so the raster stays
  sharp on a HiDPI display.
* `quality.multiplier()`. `Normal` is `1.0`: one raster pixel per device
  pixel, which is exactly `zoom × ppp` and is therefore what a build whose
  operator never opens the Settings window gets, byte for byte. The knob
  multiplies that, so the setting can only ever be a deliberate departure
  from the default — there is no compiled-in quality constant anywhere else
  for it to disagree with.

The result is finite and strictly positive without a guard, because
`sane_pixels_per_point` guarantees that of its half and
[`crate::app::prefs::RenderQuality::multiplier`] is a `const fn` over a
closed enum whose three values are 0.75, 1.0 and 1.5.

### `fn zoom_for_raster_scale`

[`crate::render::ceiling::RasterCeiling`] stores what the engine refused as a
**raster scale**, deliberately: a ceiling kept as a zoom would be wrong by the
density ratio on a window dragged between two monitors, silently, and only on
the machine it was not measured on. Every reader therefore has to convert,
and this is the conversion — the whole of [`raster_density`], not the display
density alone.

### `fn page_extent_pts`

# One definition of how big a page is

Delegates to [`crate::render::region::PageFrame::extent_pts`] rather than
reading `page.crop_box` directly. That is the point, and it has not
changed: a fit-page computed from an un-rotated `CropBox` against a rotated
raster is the classic version of this bug, so the rotation table lives in
exactly one place — the same place that holds the canvas↔user conversion
this extent has to agree with.

# Why NOT [`pdfcer_render::page_device_geometry`]'s pixmap dimensions

Those are `u32` and therefore **ceiled**, which makes them the wrong
measure of a page for a layout that translates in points. A page measuring
2383.937 × 1683.78 pt lays out as 2384 × 1684 — a canvas space 0.22 pt
taller than the page whose coordinates it carries, because
`PageFrame::user_to_canvas` puts that page's bottom edge at 1683.78.

**A rounding error in a layout is multiplied by the zoom.** At 100 % that
gap is a fifth of a pixel and invisible; at 1040 %, against a page 17,509 pt
tall on screen, it puts `render::region::region_on_screen`'s region raster
**2.3 pt** away from where the page's own rect says it belongs. Measured by
`ui-verify`'s `panning_at_deep_zoom_stays_where_it_was_put`, the only
instrument in the project that compares a raster's *painted* rect against a
rect recomputed independently from the page.

The full argument, including why the pixmap still being a fraction of a
pixel larger than the page is harmless and why the ceiled extent's version
of the same error was not, is on
[`crate::render::region::PageFrame::extent_pts`].
