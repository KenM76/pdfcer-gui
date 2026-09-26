# `egui-shell/ribbon/height_tests`

## Item notes

### `fn context`

# Why the theme has to be applied here and is not next door

`Theme::apply` writes `spacing.interact_size.y = control_height`, which
is what makes a band control exactly as tall as the metric
[`super::rhythm::rows_height`] budgets for it. Without it `egui`'s default
`interact_size.y` is 18 pt against a 24 pt `control_height`, and **every
row carries 6 pt of slack that a spacing error can hide in**.

One did. A first cut of the two-row band padded its rows against
`GROUP_ROWS × height + (GROUP_ROWS − 1) × spacing` — correct for the ink
and one gap short for the cursor, which `egui` advances past *every* rect
including the last. Under the un-themed context the slack absorbed it and
these tests passed; the running binary's own trace showed a two-row group
at 68 pt beside a one-row group at 64.

**A fixture can flatter the thing it measures, and the numbers will still
look fine.** The theme is applied so this one cannot.
[`the_fixture_is_themed_like_the_running_application`] asserts it took.

### `fn the_fixture_is_themed_like_the_running_application`

Asserted rather than assumed, because the failure it prevents is silent —
see [`context`]. A slack row makes a band-height test pass against a band
whose groups are different heights.

### `fn wide_registry`

Deliberately **unequal** label lengths, for the reason
[`super::strip_width_tests`]'s `strip_shell`'s
labels are unequal: an even split of eight identical widths is
arithmetic anyone could get right by accident, and it would not exercise
[`super::plan::wrap_group`]'s search over contiguous runs at all.

### `fn two_row_shell`

The `narrow` tab holds two controls; the `wide` tab holds eight. Under
the one-row band these two tabs were **different heights**, and the
difference moved the canvas underneath on every tab click.

### `fn row_tops`

A row is a set of controls whose tops agree to within [`SLACK`]; `egui`
rounds widget rects to whole physical pixels, so an exact equality would
make this brittle for a reason that has nothing to do with layout.

### `fn the_band_is_the_same_height_on_every_tab`

`PROJECT_PLAN.md`'s rule and this project's most-repeated bug — a
content-driven height beside a fit-to-viewport zoom is a feedback loop,
measured at 230 % → 224 % → 215 % of drift. The ribbon sits directly
above the canvas, so a band that is one row tall on one tab and two on
another re-fits the page every time the operator changes tab, and the
zoom walks.

The fixture is built so the *content* really does differ: `narrow` holds
one two-control group and `wide` holds one eight-control group that
[`super::plan::wrap_group`] splits over two rows. A band sized to its
content would report two different numbers here; a band sized from the
theme reports one.

# Why the vacuity guards are as long as the assertion

**`cargo test -p egui-shell` and `cargo test --workspace` compile with
different `egui` features** — Cargo unifies features across the graph, so
the workspace run gets capabilities the single-crate run does not — and a
layout test can be entirely vacuous under one of them. Two things could go
quiet
here — a `None` ribbon height (the measuring closure never ran) and a
band that never wrapped (both tabs the SAME row count, so "the same
height" is true and says nothing). Both are asserted as facts before the
equality is asserted at all.

The second guard asserts the two row counts **differ**, not that the
narrow tab holds one row. One row is a fact about the fixture; differing
row counts is the actual precondition, and a guard written the other way
accuses any build that learns to wrap more eagerly while still holding the
band's height fixed.

### `fn the_band_keeps_its_height_at_widths_where_every_group_overflows`

The case a height derived from drawn content gets silently wrong: below
the width at which one group fits beside the overflow reservation, the
band draws *nothing but the affordance*. A band measured from what it
drew would then be one control tall on that tab and two rows tall on the
next one — R128 arriving through the overflow machinery rather than
through the manifest.

Swept rather than spot-checked, because "at *some* width the answer is
wrong" is the shape of every defect this file exists for.

### `fn a_group_wider_than_the_cap_is_drawn_on_two_rows`

[`super::plan`]'s own tests prove the arithmetic; this proves the renderer
obeys it, which is a different claim. A `captioned_group` that emitted a
single `ui.horizontal` would satisfy the plan's tests and never produce a
second row however wide the manifest got.

The width claim is stated against the **items**, not against a constant:
the group's rect must be narrower than laying its eight controls end to
end would be. That is checkable without knowing a single measurement,
and it is the property that lets a fifth group fit on a width that would
otherwise push it behind the overflow affordance.

### `fn every_caption_in_a_band_shares_one_baseline`

The mockup's `justify-content: space-between`, and the half of "staged
nicer" that has nothing to do with wrapping: with each group as tall as
its own content, a two-row group's caption hangs a whole control-row
below its neighbours' and the band reads as ragged. Pinning the captions
is what makes the row of group names scan as a row.

### `fn two_padded_groups`

Both fixture properties are load-bearing for
[`a_group_is_inset_by_the_padding_the_plan_budgets_for_it`]:

- **Two groups**, so the *gap between* them can be measured as well as the
  inset within each. The inset is invisible in a one-group band — a group
  at the band's left edge with padding looks exactly like a group without
  it drawn 6 pt further right.
- **Controls wider than captions.** [`super::band::captioned_group`]
  centres the caption on the widest control row, so in a group whose
  caption is the wider half the caption overhangs the rows on both sides
  and the group's box is caption-driven rather than row-driven. That is
  correct behaviour and it makes the inset unmeasurable from the control
  rects, which is the only thing published. Eight- and four-control groups
  under the synthetic face are comfortably row-driven.

### `fn a_group_is_inset_by_the_padding_the_plan_budgets_for_it`

# The failure this refuses, and why nothing else catches it

[`super::plan::GROUP_PADDING`] adds 2 × 6 pt to every group's planned
width. A renderer that lays the group out as a bare `ui.vertical` with no
horizontal inset spends that budget as an accidental margin on the OUTSIDE
of the box instead of as padding on the inside, and **nothing fails**: the
plan is right, the width fits, every other test passes. Measured in the
running application at 1,100 pt, a group box and its first control both
began at **x = 322.5** — a zero-point inset, controls flush against the
group boundary and against the rule dividing them from the next group.
That is most of what the operator meant by *"cluttered"*.

# What is asserted, and why the second assertion is the interesting one

1. **The inset**, on both sides of both groups, exactly
   [`super::plan::GROUP_PADDING`]. This is what fails if either
   `add_space` is deleted.
2. **The gap between the two groups** — group edge to group edge — is
   `2 × GROUP_PADDING + `[`super::measure::separator_width`]. That is the
   number the mockup actually specifies: its `.group` padding is 13 px each
   side and its divider is a zero-width `border-right`, giving 26 px, and
   this build reaches the same 26 pt as 6 + 14 + 6 because its divider is a
   real `ui.separator()` with real width. Asserting the *sum* is what makes
   this test a statement about the mockup rather than about a constant it
   could have copied. See [`super::plan::GROUP_PADDING`] for the full
   reconciliation.

### `fn separator_width`

[`super::measure::separator_width`] takes a `&Ui` because it reads
`item_spacing` from the live style, and a test that spelled the sum out
again would pass while disagreeing with the renderer — which is the exact
failure `super::plan`'s header warns about for the *group* width.

### `fn the_band_leaves_clear_space_beneath_its_captions`

# The failure this refuses

A band whose reserved height is exactly the rows, the gap and one line of
caption ends on the caption's own baseline. Measured in the running
application at 1,100 pt in that state, the captions ended at y = 103 and
the dock's tab bar began at y = 105.3: a 10 pt line of `weak()`, `small()`
text separated from the panel header beneath it by less than its own
leading. The caption is the one piece of text that says what a block of
controls is *for*, and a caption sitting on a seam reads as a label for
whatever is on the other side of it.

# Why this is asserted against the ribbon's bottom edge

Because that is the edge the operator sees and the edge the application
puts a panel against. It is deliberately **not** asserted as
`band_height() == rows + gap + caption + padding`, which would be the
derivation restated — true by construction, and equally true of a build in
which the reservation is made and then not honoured. The reservation is
made by `ui.set_min_height`, whose whole job is to be larger than the
content; a test that never measures the content cannot tell whether it took
effect.

R128 is *not* re-asserted here — [`the_band_is_the_same_height_on_every_tab`]
and [`the_band_keeps_its_height_at_widths_where_every_group_overflows`] own
that claim, and both still hold with the padding folded into the
derivation, which is the point of folding it in rather than emitting it
after the last group.

### `fn every_preset_reserves_room_for_its_own_rows`

The one invariant that makes [`crate::theme::Metrics::ribbon_rows`] safe as
a stated number rather than a derived one, and the reason the ribbon
rhythm is a *metric* at all rather than a constant in `band`.

The band's row area is now a **budget** — the mockup's 68 px, into which
[`super::plan::GROUP_ROWS`] rows are laid, with the caption hanging off the
bottom of it (`.grp .cap { margin-top: auto }`). A budget smaller than what
two rows cost does not fail loudly: the second row simply draws over the
caption, in one preset, which is the class of defect
`MODES_AND_PANELS.md` says has exactly one oracle and this project would
rather not need it for.

`Airy` is the preset that makes this real, and it is why transcribing the
mockup's `68` into all three would have been wrong: its `control_height` is
28 pt and its `gutter` 8, so two of its rows cost 72 — **more than the
mockup's whole area.**

Asserted over `Preset::ALL` rather than over the three by name, so a
preset added later cannot ship unmeasured. That is the same discipline
`Preset::ALL`'s own doc comment asks for.

### `fn every_preset_can_still_re_wrap_a_group`

`RIBBON_SCALING.md`'s ladder is re-wrap → collapse → scroll, and rung one
divides the *same* row area into [`super::plan::MAX_GROUP_ROWS`] instead of
[`super::plan::GROUP_ROWS`]. It has a self-disabling guard —
`band::rewrap_is_legible` — that turns the rung off rather than clipping
icons when the arithmetic does not clear, and **a rung that has switched
itself off looks exactly like a rung that was never needed.** So a change
to the row area could disable the first rung of the ladder in one preset
and no other test in this crate would say a word.

The margin is stated, not just the sign: with `Quiet` the compressed row is
`68/3 − 2 = 20.67` pt against a 16 pt icon. Before the mockup pass it was
`56/3 − 2 = 16.67` against the same 16 — a margin of two thirds of a point,
i.e. the rung was one theme tweak from vanishing.

### `fn the_band_draws_clear_space_above_its_first_control`

The exact mirror of [`the_band_leaves_clear_space_beneath_its_captions`],
and it exists for the same reason: a padding that is *budgeted* and not
*drawn* is invisible to every test that measures a total height, because
the total is right and the ink is in the wrong place. That is the shipped
defect `super::plan::GROUP_PADDING` produced horizontally, and this is the
vertical version of the tripwire.

Measured from the **group's own published rectangle** rather than from the
ribbon's, because the ribbon rect includes the tab strip above the band and
would answer a different question. A group's rect starts at the band's top
edge; its first control starts `ribbon_pad_top` below that, and nowhere
else in `group_body` is any space emitted before the rows.

### `fn render_auto_hidden`

Two frames, like `render_shell_with`, and for one more reason besides its:
[`crate::peek::Peek`] answers from the pointer AND from last frame's state,
so the frame that reveals and the frame that draws the revealed band are
different frames.

### `fn an_auto_hidden_ribbon_takes_the_same_room_whether_its_band_shows_or_not`

This is the property that decides whether the setting is usable. The band is
drawn into an `egui::Area` when it is revealed, which allocates nothing, so
the `Ui` the application handed the ribbon is exactly as tall as the tab
strip in both states. The plausible wrong implementation draws the band
inline and simply skips it when hidden — every test in this file would still
pass, and the operator's drawing would jump by ninety points every time the
pointer crossed the tab row.

Both states are DRIVEN rather than assumed. An absence test that never
reached the revealed state would be satisfied by a build that never reveals
anything, which is the vacuous shape this project has shipped twice; so the
second reading asserts `Show::Overlay` before comparing, and says so if the
plant did not land.

### `fn the_tab_strip_is_the_same_height_with_the_band_shown_and_hidden`

`peek`'s direction bound rests on the trigger being independent of the
surface it reveals. Here that is a claim about *this* ribbon rather than
about `peek`: the strip is drawn by [`super::strip::render`] before the band
exists, so nothing the band does can reach it. Asserted across a width
series rather than at one width, because the strip's own overflow machinery
changes what it contains and a single width could agree by luck.
