# `egui-shell/ribbon/plan/mod`

## Item notes

### `const PACK_SLACK`

[`wrap_group`] builds its candidate widths by summing a slice and packs
by accumulating item by item. The two orders of addition are equal in
exact arithmetic and can differ by an ULP or two in `f32`, which would
make the intended candidate look infeasible by a hair and push the
answer onto the next-widest one. A thousandth of a point is far below a
physical pixel and far above the rounding.

### `fn pack`

An item wider than `target` on its own still gets placed — on a row by
itself, over budget — because refusing it would be refusing to draw a
control.

### `fn a_group_is_as_wide_as_its_caption_when_the_caption_is_wider`

`RIBBON_IA.md` §5 is full of two-word captions over one-glyph
controls — "Page display" over a single icon button — so this is
the common case, not the corner. Planning such a group at its
control's width would overflow the band by the difference, every
time, and the symptom would be a clipped caption rather than
anything that looks like a layout bug.

### `fn a_group_that_asks_for_rows_wraps_when_it_would_otherwise_fit`

`OPERATOR_REQUESTS.md` O97 — four square icon buttons that are a strip in
one row and a control in a 2 x 2 block. Without the hint the planner
short-circuits on *"it fits"* and never looks for a narrower shape.

The four widths are comfortably inside [`GROUP_WRAP_WIDTH`], so the
`None` case is the regression guard as much as the `Some` case is the
feature: **if the default ever starts wrapping, every group in every
application changes shape at once.**

### `fn the_row_hint_is_a_preference_and_the_bands_ceiling_still_wins`

Three properties, and each is a way the feature could have been built
wrong:

* asking for **one** row changes nothing — that is already the default,
  and a `Some(1)` that started wrapping would be a footgun in a manifest;
* the **band's ceiling wins** — a group asking for four rows in a
  two-row band gets two, because the band's height is fixed (R128) and a
  manifest must not be able to break it;
* asking does not FORCE the count — the planner still returns the
  narrowest packing, so two items whose one-row form is narrowest keep it.

### `fn a_group_that_fits_the_cap_stays_on_one_row`

The half of the mockup's rule that is easy to lose: `max-width` is a
*trigger*. Wrapping every group would turn a two-control group into
a column of one control per row, which is narrower and is not a
ribbon. Most groups in a real manifest are under the cap, so this is
the common path and not the corner.

### `fn a_group_over_the_cap_is_split_evenly_and_costs_less`

The evenness is asserted as a bound rather than as an exact split,
because the optimum depends on the item widths — seven items of 75
divide 4/3, seven of 20 with one of 300 do not divide at all. The
bound that always holds is *"the widest row is no wider than it has
to be"*, and the strongest cheap statement of that is: no row may be
wider than the widest single item plus the balanced share.

### `fn the_split_is_a_contiguous_partition_of_every_item`

The manifest's order is the operator's order — the same rule
[`BandPlan::shown`]'s prefix property exists for. A wrap that
reordered items to pack them better would rearrange the ribbon as
the theme's gutter changed, which is exactly the class of surprise
this module refuses elsewhere.

### `fn one_item_and_one_row_are_both_left_alone`

`max_rows == 1` is the old, one-row band expressed in the new
arithmetic. Keeping it reachable is what makes "two rows" a decision
this module records rather than a behaviour it merely has.

### `fn one_oversized_control_still_lets_the_rest_wrap`

The interesting shape, because the naive answer — "this cannot be
split, give up" — would leave a group at the sum of every item when
one oversized control is present, and oversized controls are exactly
what a long label produces.

### `fn a_band_that_fits_reserves_nothing`

The second half matters — a reservation that persisted when
nothing was hidden would be a permanent tax on every band that
fits.

### `fn the_overflow_affordance_survives_a_band_too_narrow_for_any_group`

This is the invariant the whole module exists for. The observed
defect it guards against is `MODES_AND_PANELS.md` Part 2, #8: past
a certain count the overflow button *itself* gets hidden, leaving
no route to what it was hiding. The plan must degrade to "no
groups, one working affordance", never to "some clipped groups, no
affordance".

Checked at a series of widths on the way down, because the
interesting failure is not at zero — it is at the width where a naive
implementation still has *just* enough room for a group and
therefore spends the overflow control's space on it.

### `fn the_overflow_affordance_is_reserved_exactly_when_it_is_needed`

A biconditional rather than one implication, because both
directions are real defects. Reserving with nothing hidden wastes
band width forever; hiding with nothing reserved is #8 itself.

### `fn widening_the_band_never_hides_a_group_that_was_visible`

Monotonicity is what makes a window resize feel like a resize.
A greedy fill has it by construction; a later "pack the widest
first" optimisation would not, and this test is what would refuse
that change.

### `fn the_visible_groups_are_a_prefix_and_nothing_is_lost`

The prefix property is what stops the visible ordering of the
ribbon from depending on the window width — the manifest's order
is the operator's order, and a plan that dropped a middle group to
fit a later narrow one would rearrange the ribbon as the window
moved.

### `fn a_non_finite_width_degrades_safely`

`egui` hands out `f32::INFINITY` for available width inside an
unbounded container. `INFINITY - overflow_width` is still
infinity, so an unguarded implementation would conclude that
everything fits and emit no affordance — the #8 defect arriving
through arithmetic rather than through ordering.

### `fn the_reservation_covers_the_widest_label_by_width_not_by_digit_count`

The trap this pins is the one real text springs and zero-width
text cannot: in a face whose digits are not tabular, `"⏷ 8 more"`
can be wider than `"⏷ 9 more"` even though the counts and the
lengths say otherwise. Reserving for `N = total_groups` alone —
which reads as obviously sufficient — then draws a control wider than
the space held for it, and the affordance overhangs the band's right
edge.

The `measure` below is deliberately perverse about exactly that:
every character costs 7 pt except `8`, which costs 40. A
reservation that consults only the largest count fails here; one
that takes the maximum over every reachable label passes.

### `fn the_overflow_label_uses_the_pinned_chevron`

Whether the character is *drawable* is asserted in the consuming
application (`pdfcer_gui::shell`'s
`the_ribbon_overflow_chevron_has_a_glyph`), because `cargo test -p
egui-shell` compiles without egui's `default_fonts` — a `has_glyph`
call here would answer about a font set no real build has, and would
pass for the whole life of a defect.

So this test does the half it *can* do honestly: pin the codepoint,
so that changing it is a deliberate act which fails a named test and
sends the next reader to the other half. Both docks and both rows
are covered, because [`crate::dock::plan::overflow_label`] promises
to stay identical and identical wording means identical codepoints.
