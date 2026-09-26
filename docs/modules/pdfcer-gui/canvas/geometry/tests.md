# `pdfcer-gui/canvas/geometry/tests`

## Item notes

### `fn panning_stops_a_whole_viewport_past_the_page_edge`

> *"I should also be able to move the view of the corner of the page to
> the center of the screen, or even all the way vertically to the
> opposite corner if I want to."*

With a one-viewport pasteboard the content is `1000 + 2×800 = 2600`
wide, so the last offset that still shows anything is `2600 − 800 =
1800`. The pan asks for `700 + 500 = 1200`, which is now inside the
range and is therefore granted in full.

### `fn any_page_corner_can_be_brought_to_the_centre_and_to_the_opposite_corner`

The pasteboard's size is not a taste; it is whatever makes these two
true. If `PASTEBOARD_FRACTION` is ever reduced, this fails and says
which sentence stopped holding.

### `fn pasteboard_rule`

Four tests below need to know how much slack a viewport gets, and every one
of them needs to know it *independently of [`pasteboard`]* — a test that
calls the function under test agrees with it by construction, including when
it is wrong. Before O186 they each spelled it `v * PASTEBOARD_FRACTION`
inline, which was right while the rule was one term.

It is now two, so the restatement lives here once: a whole viewport, less
the band [`MIN_SHEET_ON_SCREEN`] keeps on screen. Still a restatement — it
reads the two constants and does the arithmetic itself, so a change to
`pasteboard`'s *code* cannot be ratified by these tests, only a change to
its *constants*. That is the property worth having, and four copies of it
drifting apart is not.

### `fn viewport_half_less_sliver`

`pasteboard` offers the larger of `viewport × FRACTION − sliver` and
`overhang + viewport / 2`, so the second wins once the overhang passes
`viewport / 2 − sliver`. Worth naming because the number is not obvious and
two tests need it: it is **zero** on a viewport narrower than twice
[`MIN_SHEET_ON_SCREEN`], and a few hundred points on a real canvas.

### `fn anchored_screen_x`

Screen position is `margin + frac * display - offset`, which is the same
expression the doc comment solves — so this checks the solve, not merely
that the code agrees with itself about arithmetic (assert the outcome,
not the intent).

### `fn the_scroll_offset_saturates_at_the_pasteboard_edge_not_at_the_page_edge`

The behaviour the old test was protecting is real and still wanted —
an anchor near an edge must saturate rather than scroll into nothing.
It just belongs to the value that reaches the widget. So the assertion
moved to [`strip_offset`], which clamps against `content_extent`, the
pasteboard included.

### `fn placing_an_anchor_and_measuring_it_are_exact_inverses`

Framing a rect is "put this page point at the viewport centre", which
is the second function with a chosen target. If the pair ever stopped
being an exact inverse, a marquee zoom would land the region *near* the
centre — an error small enough to look like a rounding artefact and
large enough to be the wrong answer.

### `fn the_strip_bridge_is_a_pure_pasteboard_shift_for_a_single_page`

The mechanical form of "continuous is an option, not a replacement":
the default path must not merely *behave* the same, it must compute the
same number. With one page in the strip its origin is `(0,0)` and the
strip's size is the page's, so both terms cancel — at every zoom, and
with the page both larger and smaller than the viewport (the latter is
where the centring margin is non-zero and a sloppy conversion would
show).

### `fn the_strip_bridge_preserves_where_a_page_point_lands_on_screen`

The property the whole pair exists for, asserted as an *outcome*: take
a fraction of the current page, work out where it appears on screen
from the real strip geometry, then work it out again through the
page-local view the zoom and reveal solves are handed — and require the
two to agree. A conversion that dropped either margin term would pass
every algebraic check and fail this one at exactly the zoom an operator
starts from.

### `fn measuring_the_offset_from_the_drawn_rect_matches_the_solved_one`

This is the claim O26e's fix rests on, and it is the claim that makes
the change safe: `canvas::show` swapped one for the other on **every**
frame, not only deep ones, so if they disagreed anywhere below the
threshold the fix would have traded a rare catastrophe for a constant
small one.

The shallow tier's geometry is reconstructed here exactly as `show`
builds it — content origin, strip centring margin, pasteboard, the
page's place inside the strip, the scroll offset — and then the page's
screen rect and the viewport's screen rect are handed to
`offset_from_drawn` the way `show` hands it `image_rect.min` and
`inner_rect.min`. Spelled out rather than calling `strip_margin`,
for the reason the sibling test above states: a test that reuses the
function under test agrees with it by construction, including when
both are wrong.

What this does **not** claim, deliberately: that they agree at the
deep tier. They do not, and that is the whole point — there the scroll
offset is forced to zero and `page_local_offset` describes a page
nobody is looking at, while `offset_from_drawn` describes the one on
screen. There is no assertion to write for "one of these is a lie",
only a driven check: `zooming_back_out_keeps_the_view`.

### `fn a_non_finite_drawn_rect_measures_as_centred_rather_than_as_nan`

Zero rather than the previous value, because this function has no
previous value to return — it is a measurement, not a step. "Centred"
is the safe fiction; a `NaN` propagates into `zoom_anchor_offset` and
blanks the canvas, which is the one outcome worse than a wrong offset.

### `fn the_strip_origin_is_the_plain_expression_wherever_that_expression_is_exact`

The magnitudes here are deliberately ordinary. The whole point of the
symbolic form is that it agrees with the plain one where the plain one
is trustworthy and continues to be right where it is not, and only the
first half of that is assertable in `f32` arithmetic — the second half
is what `zooming_back_out_keeps_the_view` drives.

### `fn a_pinned_axis_centres_a_page_that_fits_and_sits_flush_with_one_that_does_not`

Asserted through [`anchor_screen_pos`] rather than by re-stating the
arithmetic: what matters is not that the function returns zero, it is
**where the page's top-left ends up on screen** when it does. A test
that checked for zero would keep passing if [`margin`] were changed
underneath it, which is exactly the coupling this pins.

### `fn an_unpinned_axis_is_kept_but_clamped_to_the_page`

Both halves in one test, because they are one rule. Keeping the
position is what stops "Fit width" throwing the operator back to the
top of a long sheet; clamping it is what stops "kept" meaning "still
looking at nothing".

### `fn an_unpinned_axis_on_a_page_smaller_than_the_viewport_still_centres`

This is the landscape-sheet-under-fit-width case, and the one where a
`max(0.0)` on the wrong side would leave the page pinned to the top of
the window with a gap underneath.

### `fn centring_agrees_with_the_pinned_fit_answer`

On an axis a fit **pins**, holding the page's own centre at the viewport
centre gives exactly the offset [`fit_placement_offset`] gives. So
preserving the centred point across a resize **subsumes** the fit's
re-placement, and `canvas::fit` can stop having a separate resize path.

If this ever fails, deleting that path changed what Fit page does — which
is why it is asserted over a range of sizes rather than at one, and why it
asserts the two functions agree rather than asserting each is zero.

### `fn measuring_the_centred_point_and_placing_it_are_exact_inverses`

The sibling of `placing_an_anchor_and_measuring_it_are_exact_inverses`, and
written in the same shape: at an unchanged viewport the round trip must
return the offset it started from, or a frame with no resize would still
move the view.

### `fn a_resize_keeps_the_same_page_point_in_the_middle`

> *"when I change the size of the canvas window, whatever area was centered
> in the current canvas should stay centered."*

Asserted by measuring where the preserved fraction lands **after** the
resize, rather than by comparing offsets — an offset that happened to be
preserved for the wrong reason would pass a comparison of offsets and fail
this.

### `fn a_degenerate_extent_centres_rather_than_producing_a_nan`

`offset_holding_anchor_at` fails to `0.0` because that is a legal *offset*;
this is a *fraction*, so the harmless value is `0.5`. Asserted because the
two guards look alike and choosing the wrong constant would put a NaN into
a scroll offset one call later.

### `fn the_opening_seed_centres_a_large_page_and_is_a_no_op_for_a_small_one`

`OPERATOR_REQUESTS.md` O78: *"when starting the view should be centered on
the canvas when a pdf is first opened."*

The second half is what makes the change safe to ship: under the shipped
default (Fit page, so the page always fits) the new seed expression is
**exactly** the old literal `(0.0, 0.0)`, so nothing about the common path
moves.

### `fn a_fixed_pasteboard_stops_reaching_off_page_content_at_a_calculable_zoom`

With a fixed one-viewport pasteboard, the slack is a count of **screen**
pixels, so the slice of the **drawing** it covers is `viewport / zoom` and
shrinks with every notch. Centring a point 100 pt off the sheet needs
`pasteboard ≥ 100 × zoom + viewport / 2`; with `pasteboard = viewport`
that is `zoom ≤ 235 / 100`, i.e. **235 %**. Above it the object walks off
the screen while the operator zooms toward it.

O186 moved the crossover to **203 %** — `(470 − 32) − 235`, over 100 —
because [`MIN_SHEET_ON_SCREEN`] comes out of the fixed slack. The table
below is therefore re-measured rather than re-tuned, and the ceiling is
derived from the rule instead of written as a literal, so the next change to
either constant moves it without a second edit. The number itself does not
matter to anybody: this test pins the *shape* of the old defect — a ceiling
that exists at all — and the whole point of the overhang term is that the
ceiling does not apply when there is content out there to reach.

### `fn an_object_off_the_page_can_be_centred_at_every_zoom`

`strip_offset` is the one function whose answer actually reaches the
`ScrollArea`, and it is the one that clamps — so a solve that is thrown
away by the clamp is indistinguishable, to the operator, from a solve that
was never made. This drives the whole chain: the offset that would put the
off-page point at the viewport's centre must survive the clamp unchanged.

Single page, so `page_origin` is `(0,0)` and `strip == page_display`; that
is the geometry the driven check `an_object_off_the_page_is_actually_drawn`
exercises, and the same shape as `off-page-object.pdf`.

### `fn a_page_with_nothing_off_it_takes_the_fixed_pasteboard_branch`

It used to assert that the value was `v * PASTEBOARD_FRACTION` exactly,
and was titled *"keeps exactly the old pasteboard"*. O186 made that false on
purpose — see [`MIN_SHEET_ON_SCREEN`] — so what it asserts now is the thing
it was always *for*: that the overhang term does not bite until the content
genuinely reaches further than the fixed slack. [`pasteboard_rule`] is the
independent restatement, so this still cannot be satisfied by
[`pasteboard`] agreeing with itself.

### `fn the_ends_of_the_range_are_the_two_scroll_offsets_the_area_can_reach`

`OPERATOR_REQUESTS.md` O186 stage one. Stated in the direction that is
*exact* in `f32` — take the two reachable scroll offsets, `0` and
`content_extent − viewport`, convert each into strip space with
[`scroll_to_strip`], and you get `lo` and `hi` on the nose. Both sides are
`x − strip_margin(...)` evaluated on the same arguments, so there is no
rounding to argue about.

The reverse direction is asserted as *behaviour* rather than as an
identity, and the nudge is `64.0` rather than `1.0` on purpose: a round trip
through `hi + pad` is two roundings, and at the strip magnitudes this tier
reaches — nine hundred thousand points and up, where an `f32`'s step is a
sixteenth of a point — `1.0` is close enough to the boundary to be arguing
with the last bit rather than with the clamp. 64 points is unambiguous at
every magnitude tested and is still a fraction of a viewport.

### `fn without_an_overhang_the_top_of_the_range_is_the_strip_itself`

A corollary rather than a separate rule, and worth pinning because it is the
shape every number in O186's trace block is read against:
`pasteboard = viewport × PASTEBOARD_FRACTION − sliver`, so
`hi = strip + pasteboard − viewport` collapses to `strip − sliver`. If
someone later changes either constant this test fails immediately and says
which identity the trace block's arithmetic was relying on, rather than
leaving a reader to wonder why a measured `hi` of 1684.27 nearly equalled a
measured sheet height.

**The `− sliver` is the whole of O186's second half.** Without it `hi`
is `strip` exactly, which places the viewport's top-left on the strip's last
point: zero overlap, `canvas-unavailable reason=nothing-visible`, and a
clamp to that endpoint is a clamp onto a blank frame. Both ends are
asserted, because both ends were blank.

### `fn the_range_is_never_inverted_however_short_the_strip`

`hi − lo` is `content_extent − viewport` by construction, which is
`2 × pasteboard` plus `display.max(viewport) − viewport ≥ 0`. The grid below
covers the cases a formula written out longhand gets wrong: a strip shorter
than the viewport, a strip of exactly zero, a strip equal to its viewport, a
degenerate viewport, and an overhang large enough to hit the pasteboard cap.

### `fn a_short_strips_range_is_centred_on_the_centring_margin`

Unreachable at the deep tier, where a strip is `pages × page × zoom` and the
tier engages only once `longest_page_pt × zoom` exceeds
[`crate::viewer::ceiling::SUB_PIXEL_CONTENT_EXTENT`], and asserted anyway
because the function is ordinary geometry and a reader of the shallow tier
will hit it. The answer must be the *shallow* tier's answer: centring a display
smaller than its viewport is what [`margin`] and [`fit_placement_offset`]
already do, and a range whose midpoint was anything else would mean the two
tiers disagree about where a small document rests.

### `fn the_o186_anchor_is_pulled_back_to_the_end_of_the_sheet_from_either_side`

The two things this pins that a one-sided test would not:

* **the magnitude is the whole defect.** 0.05 pt is nothing; `0.05 × 539.7`
  is 27 logical points of strip, which against a 700-point viewport is
  enough to carry the entire strip off it. The assertion on the recovered
  page coordinate is what keeps that factor honest — a future change that
  clamped to the right side of the wrong bound would still satisfy
  `clamped == hi`.
* **both signs.** A suite that only tries one sign is not testing the value;
  the anchor in the trace had *also* been seen at `-0.54` pt, above the
  sheet rather than below it, which is the same defect reached from the other
  end of the pasteboard.

### `fn a_nonsense_frame_collapses_the_range_to_the_strips_origin`

Not a tolerance and not a fallback to the last good value: a degenerate pair
makes [`crate::canvas::deep::confine`] pin the anchor at the strip's origin
for that one frame, which is the one placement that needs no history and
cannot be wrong about anything.

### `fn the_draft_longhand_range_was_wrong_in_the_two_ways_measuring_it_found`

The draft was `(-pb, (strip + pb - viewport).max(-pb))`. See
[`visible_origin_range`]'s doc for the argument; this is the measurement
behind it, and it exists because the doc originally claimed the `.max` was
*needed*. It is not. Writing the test is what said so.

⇒ Two lessons, both general: **a guard whose precondition is decided by a
constant elsewhere in the module cannot be read locally** — `.max(-pb)`
looks essential and is dead — and **a fresh formula for a range some
existing function already clamps to re-derives the range and invents its own
faults**, which here meant a window narrower than the scroll area's by
exactly the centring margin.
