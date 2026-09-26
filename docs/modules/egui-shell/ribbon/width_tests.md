# `egui-shell/ribbon/width_tests`

## Item notes

### `fn render_view_tab`

Two frames because `egui` resolves some geometry a frame late; the
second is the honest one, exactly as the harness in [`super::tests`]
does it.

### `fn the_band_measures_real_text_and_not_a_floor`

The guard on every other test in this file. With no font data the
three View groups would each collapse to their item count times
[`super::plan::MIN_ITEM_WIDTH`] and would come out *identical* in
width; with a proportional face they differ, because "Page display"
over three buttons is not the same width as "Render" over two.

Asserted through the **published rects** rather than through
`layout_no_wrap` directly, because what needs proving is not that the
font loaded — [`testfont::install`] proves that — but that the font
reaches the ribbon's own measurement path.

### `fn the_overflow_affordance_is_on_screen_at_every_width`

Correct reservation arithmetic is not enough on its own. The band asks a
`Ui` for its width, and if the tab-strip row above it has overflowed and
grown that `Ui`'s `max_rect`, the reservation is taken from a right edge
off screen — measured at 78 pt past it. The affordance then exists, has
area, is reported, and cannot be clicked, which is a state no arithmetic
test can see.

The sweep matters. A single narrow width would have caught this one,
but the family of bugs it belongs to is "at *some* width the answer is
wrong", and the interesting widths are the ones where a naive
implementation still has *just* enough room for one more group and
spends the affordance's space on it.

### `fn no_visible_group_overlaps_the_overflow_affordance`

The other half of the reservation: it is worth nothing if a group
drawn under real metrics overruns its budget and paints on top of the
affordance. `plan_band` subtracts the reservation before measuring, and
`render_band` hands the groups a `Ui` whose `max_rect` stops there —
this asserts that the two together actually hold once the widths are
real numbers rather than zeros.

### `fn a_band_that_claims_to_fit_really_does_fit`

This is the estimate-accuracy check, asked through the renderer rather
than of the estimator, and it is the one that would catch an
**under**-estimate — the dangerous direction.

[`super::plan`] budgets groups analytically, from item labels plus
padding constants, because immediate mode cannot measure a group before
deciding whether to draw it. If those constants disagree with what
`egui` actually applies (the icon/label gap being `icon_spacing` rather
than the theme's `gutter` is exactly such a disagreement, and was one),
the plan concludes that three groups fit in a band that holds two and a
half, and the third is drawn off the right-hand edge with no affordance
offering it — because the plan does not think anything is hidden.

The property is checkable without knowing the estimate: on any frame
with **no** overflow, every group the band drew must lie inside the
window.

# Why this binary-searches rather than sweeps

The only widths at which an under-estimate is *visible* are the ones
between "the plan stopped overflowing" and "the content genuinely
fits". An estimate that is short by 8 pt makes that window 8 pt wide,
and a sweep at any step coarser than the error walks straight over it —
which was measured, not assumed: with the item padding deliberately
cut to a fifth, an 11 pt sweep still reported success.

So the test finds the transition width exactly (the counts are
monotonic in width, which
`widening_the_band_never_hides_a_group_under_real_metrics` pins
separately) and asserts **there**, where any shortfall at all is
visible, plus at the two widths above it.

# Its sensitivity floor

[`super::plan::GROUP_PADDING`] contributes 2 × 6 pt to every group's
planned width, and [`super::band::captioned_group`] insets the group by
that same constant — so the reservation is spent on ink rather than
standing as slack. That coupling is what gives this test its sensitivity.
A band that planned the padding without drawing it would carry 12 pt per
group of accidental safety margin, and any under-estimate smaller than
the margin would be absorbed and invisible here: measured against such a
band, cutting the item padding by 20 % changed nothing, and only removing
it outright failed at the transition width.

The test itself asserts only that a band claiming to fit really fits —
a claim about the renderer — so it needs no adjustment when the padding
moves. What moves with the padding is the smallest under-estimate it can
still see.

### `fn a_band_narrower_than_the_affordance_still_shows_it`

The degenerate case the reservation exists for, and the one the
arithmetic alone cannot answer: at 40 pt the band cannot fit the
control it is obliged to show. `MODES_AND_PANELS.md` #8 dictates what
gives — not the affordance. So the control is clamped into the band
rather than positioned by subtraction from the right edge, its label
truncates, and the shortfall is disclosed through the verification
channel as `ribbon-overflow-affordance-clamped`.

What is asserted is the part that matters to an operator: the control
is fully on screen, has area, and every group is reachable through it.

### `fn the_affordance_is_hit_testable_under_real_metrics`

A rectangle proves something was allocated; only `egui`'s own hit test
proves it can be reached, because that is what accounts for clipping,
for occlusion by a later widget and for a zero-area interact rect.

Checked at three widths: one where a group still fits beside the
affordance, one where none does, and one narrower than the affordance
itself.

Those widths are a property of the fixture manifest and the synthetic
face, not constants with meaning of their own: `band::measure_group_rows`
asks every group for the band's row ceiling, so the View tab's groups
stack into columns and fit in less width than a single-row arrangement
needs. If the `overflow_visible` precondition goes red, the question is
*"do the groups still not fit?"* and the answer is a measurement, not a
smaller literal. That precondition is why a change to the band's
arrangement surfaces here as a red test naming its own vacuity, rather
than as a green test asserting nothing.

### `fn widening_the_band_never_hides_a_group_under_real_metrics`

[`super::plan`] asserts this of the arithmetic. This asserts it of the
whole renderer with real text, where the group widths are not the
uniform 100 pt of the unit test and where a mis-measured group would
show up as a band that flickers a group in and out as the window is
dragged.

### `fn the_mode_selector_stays_within_the_row_at_every_width`

Two things on this ribbon must never be squeezed out by content: the
mode selector and the overflow affordance ([`super`]'s header owns that
rule). Laying the selector out first, from the right edge, delivers it
against content. It delivers nothing when the selector alone is wider
than the row: `egui` answers an over-wide
`allocate_exact_size` in a right-to-left layout by extending past the
container's left edge, silently.

With the synthetic face a three-position *Read · Review · Edit*
selector wants ~150 pt, so the sweep starts well below that and the
clamp in [`super::mode_selector::fit_track`] is what is under test. Its
failure mode without the clamp is a first position at a negative x —
drawn, reported, and unclickable.

### `fn the_reservation_covers_every_label_the_control_could_show`

The circularity in [`super::plan::overflow_width`] — the reservation is
needed before the hidden count is known — is broken by reserving for
the worst case. With no fonts, *every* label measures zero and the
worst case is a tautology; with a proportional face it is a real claim,
and the naive version of it ("reserve for the largest count, since it
has the most characters") is false in any face whose digits are not
tabular.

This asserts it directly: for a band of `n` groups, the reservation is
at least the width of every label the control could ever show.
