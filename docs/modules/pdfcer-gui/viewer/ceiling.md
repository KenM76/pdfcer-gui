# `pdfcer-gui/viewer/ceiling`

## Item notes

### `fn the_zoom_ladder_can_climb_to_a_configured_maximum`

**Moved here from [`super`] on 2026-09-12, and the move is the point.**
Every test below asks one question — *how far may this page be magnified?* —
which is the question this file exists to answer, and they were sitting in
`viewer/mod.rs` only because they predate the split that created this file.
`viewer/mod.rs` had reached 1,498 lines of R2's 1,500, so O186's fourth
parameter could not have been tested at all without finding the seam first.
R2's own wording: *"when a file approaches the limit, that is the signal to
find the seam, not to raise the limit."* The seam was already named in this
file's header.

What deliberately did NOT move: the tests of `super::max_zoom_for_page`,
`super::raster_scale` and `ViewState`'s clamping. Those answer questions
about a *pixmap* and about *where the view is*, and the note above this
file's `use super::...` line already records why that function stays in
[`super`] "with its own tests". Dragging them across would have made this
file the home of two subjects instead of one.

### `fn the_zoom_ladder_can_climb_to_a_configured_maximum`

`zoom_ceiling` answering a big number is necessary and not sufficient:
the `+` button walks `ZOOM_LADDER`, which ends at 8.0. If stepping
stopped there the setting would be honoured by every code path except
the one the operator actually uses, which is the same silently-inert
control in a subtler place.

This is the gap `OPERATOR_REQUESTS.md` O24 predicted in its own
words — *"the buttons stop working exactly where the setting starts
mattering"* — asserted rather than left to be discovered.

### `fn a_configured_maximum_is_honoured_past_the_whole_page_raster_limit`

`OPERATOR_REQUESTS.md` O24 warned in as many words that shipping the
setting without the mechanism would produce *"a control that is drawn,
accepted, persisted, and quietly overruled downstream"* — the operator
types 100,000 % and the zoom stops near a thousand with nothing said.

This is that failure, stated as an assertion. `zoom_ceiling` must
answer the operator's configured maximum wherever it is higher than
the whole-page raster limit, on a page large enough that the raster
limit really does bind.

### `fn the_default_reaches_the_maximum_on_every_page_and_display_scale`

The property is kept, not dropped: **what must not change is the
PANNING**, which is what he actually cares about. That is asserted by
`every_zoom_the_shell_offers_today_still_rasterizes_the_whole_page` in
`render::strategy`, which walks the whole ladder — the ceiling is
permission, and the strategy is behaviour.

### `fn a_low_setting_does_not_lift_the_whole_page_raster_limit`

The half that survives from the test this replaced, and it is the one
that stops the change being dangerous: below `MAX_ZOOM` the whole-page
raster limit is a real constraint — an A1 sheet at 1.5x tops out at
690 %, not 800 % — and asking past it would demand a raster the engine
refuses.

### `fn the_deep_position_model_takes_over_only_past_the_sub_pixel_extent`

One unit of content space is one screen pixel, so `2^24` content points
is the last extent at which the offset is exact. Below it the scroll
area is authoritative and the canvas is unchanged; above it
`viewer::deep::DeepAnchor` is.

Asserted on both sides of the threshold, because a predicate that
answered `true` everywhere would put the whole shell on the deep path —
and that path is the one that has never carried ordinary use.

### `fn with_regions_the_page_size_no_longer_caps_the_zoom`

This is the whole point of the region tier stated as an assertion. In
the whole-page tier an A0 sheet hits its ceiling far sooner than a
business card, because the ceiling is a pixmap size and the page is in
it. With regions the pixmap is the window, so both pages reach the same
limit — the operator's.

### `fn a_page_that_has_refused_nothing_keeps_its_derived_ceiling`

`learned_raster_scale` is `None` on every page of every document until a
render is actually refused for a raster limit, which for almost every
file the operator opens is never. `None` must therefore be the exact
identity, not approximately so.

Walked across three page sizes, three display densities and three
configured maxima — 27 combinations — rather than asserted at one point,
because the expression it guards is a `max` of a `min` and a regression
that broke only the middle rung would sail past a single-point check.
That is not hypothetical:
`the_default_reaches_the_maximum_on_every_page_and_display_scale` above
walks a grid for exactly this reason and its own comment records that one
case would have missed the defect it was written for.

The comparison is against the derived parts **recomputed here**, not
against another call to [`zoom_ceiling`]. There is no three-argument form
any more, so a test that compared the function to itself would pass under
every possible change to it — including deleting the whole body.

### `fn a_learned_ceiling_overrules_even_a_trillion_percent`

This is the clause that answers O186. `max_zoom_percent` is deliberately
unbounded — he asked for a trillion percent and
`a_configured_maximum_is_honoured_past_the_whole_page_raster_limit` above
asserts that he gets it — and above the region tier the page's own size
stops entering the arithmetic. So without this clause *nothing* in this
function can stop a page whose rasterizer has already been measured
giving out, and what he saw instead was an error sentence painted across
his drawing.

The numbers are a real measurement, not a round one: an E-size sheet
refused at raster scale 284,964, where a business card got to 8,053,069 —
a 28× spread on the same build, which is the whole reason this ceiling
has to be learned per page rather than derived once.

### `fn the_learned_ceiling_is_a_raster_scale_and_not_a_zoom`

Pinned as its own test because this is the one part of O186 that fails
*silently* rather than visibly if it is got backwards. A ceiling stored
as a zoom would be too high on a high-DPI monitor and too low on a
standard one, so the operator would meet the same wall again on exactly
one of his two screens while the shell looked correct on the other — and
a window dragged between them would change the answer with no event
anywhere to explain it.

**The quality rows are O218 and they are the ones that were false.**
The density row alone passed for a year while the conversion divided by
the density and dropped the quality multiplier, because the test that
measured the conversion only ever varied the half that was right. A
test's coverage of a product is the product of the axes it varies, and
this one varied one of two.

### `fn a_learned_ceiling_is_not_re_ordered_at_the_zoom_it_permits`

Every other test here compares one derivation against another, which
cannot catch a factor missing from both. This one closes the loop the
only way it can be closed without a renderer: take the zoom this
function offers under a learned ceiling, put it back through
[`crate::viewer::raster_scale`] — the function the canvas actually uses
to order a raster — and check that it does not re-order the scale that
was refused.

# Why the LEARNED clause, and why the derived ones cannot be asserted
this way

A ceiling from the derived clauses is not a claim about a whole-page
raster at all. Above [`max_zoom_for_page`] the canvas switches to
[`crate::render::strategy::Strategy::Region`], whose pixmap is the
*window* — so a derived ceiling well past the page's own limit is
correct, and an assertion that a derived ceiling fits `MAX_PIXMAP_EDGE`
would be testing a rule the shell does not have. The learned clause is
different in kind: it is a **measurement of what the engine refused**,
so re-ordering at or above it is by definition another refusal.

The operator's own build runs `render_quality = sharper`, so the Sharper
rows are not hypothetical. The old conversion divided the refused scale
by the display density alone, which returns a zoom that rasterizes at
exactly `scale × 1.5` — the clamp that exists to rescue him handed the
engine a *larger* raster than the one it had just refused, and he was
shown *"this zoom is further in than pdfcer can rasterize"* again.

### `fn a_learned_ceiling_is_bounded_below_and_never_raises_anything`

Three clauses in one test because each is a bound on the same `min`:

* floored at [`MIN_ZOOM`], so even an absurdly small learned scale leaves
  the document navigable — the failure mode that would otherwise present
  to the operator as a file that refuses to be magnified at all, with no
  sentence anywhere that could explain it;
* a learned value ABOVE the derived ceiling changes nothing, because this
  clause is a narrowing and not a replacement;
* a degenerate value changes nothing. [`crate::render::ceiling`] refuses
  to learn one, so it cannot arrive from there — the guard exists because
  this function is `pub`, and this test is what stops the guard being
  deleted as unreachable by someone who checked only the one caller.
