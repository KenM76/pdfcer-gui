# `egui-shell/ribbon/plan/row`

## Item notes

### `fn grant`

The three-way answer is the whole point — see this module's header on
why a width between zero and `floor` is not a smaller region but a
control drawn outside it.

### `fn sane`

`egui` hands out `f32::INFINITY` for available width inside an
unbounded container, and `INFINITY − anything` is still infinity — a
region granted an infinite width would be laid out somewhere no
coordinate system contains. `NaN` and negatives get the same answer for
the same reason: the degenerate case must be the *safe* one.

### `const FLOOR`

Measured, not invented: `button_floor` is
`button_padding (8) + "…" (11.6875)` against the synthetic face of
[`super::super::super::testfont`], and `gap` is `egui`'s default
`item_spacing.x`. See [`super::super::super::measure::min_button_width`].

### `fn no_reservation_may_leave_the_tabs_with_nothing`

Without the floors, measured against the synthetic face at a 180 pt
viewport, the QAT runs from x = −6 to x = 160 with both tabs
entirely off screen. The arithmetic form of "must be
impossible" is this: **whatever the QAT asks for, the tabs are left
something usable**, at every row width above zero.

Swept over the whole plausible range of both, because the
interesting case is not "the QAT is enormous" — that one is easy to
spot — but the widths where it is *nearly* the whole row and a
subtraction would leave a small positive number that looks fine and
is not.

### `fn a_row_with_room_grants_every_reservation_untouched`

The other half of the rule above: a floor that also applied when
there was plenty of space would be a permanent tax, and a
reservation that rounded up would push the tabs left for no reason.

### `fn a_wide_qat_cannot_squeeze_the_selector_below_one_control_per_position`

The QAT is reserved first, so without
[`RowDemand::selector_floor`] it takes everything the tabs do not
need and the three-position selector is left whatever remains. At a
180 pt row that was 19.7 pt — 6.6 pt per position, which
`MODES_AND_PANELS.md` Part 1 forbids in the plainest terms it uses
anywhere: *"all three labels visible"*.

Holding one button's width per position back from the QAT's share
is what makes the ordering a *priority* rather than a licence.

The claim is carefully **relative**, and the difference matters: it
is *"an enormous QAT leaves the selector no less than a
zero-width QAT would"*, not *"the selector always gets its floor"*.
The second is not true and must not be asserted, because when the
row itself is narrower than `tabs_floor + selector_floor` the
selector compresses further — the tabs' floor outranks it and there
is nothing left to take.

### `fn a_truncated_reservation_says_so_and_an_untouched_one_does_not`

The flags are what [`super::super::strip`] turns into
`ribbon-qat-truncated`, and they are what makes a silently shrunken
row a different fact from a row that fitted. A flag that merely
meant "the row is narrow" would fire on every narrow window and be
ignored within a day.

### `fn a_degenerate_row_is_answered_with_zeros`

`egui` hands out `f32::INFINITY` for available width inside an
unbounded container; `INFINITY − anything` is still infinity, and a
region granted an infinite width would be laid out somewhere no
coordinate system contains.

### `fn the_active_tab_is_always_shown_and_never_hidden`

The rule the whole pin exists for. A band may legitimately degrade
to "no groups, one working affordance" because everything it hid is
one click away; a strip cannot, because the tab the operator is
looking at is the one thing its menu is not a route to. A strip
that hid it would show a band whose owner is invisible.

Swept across every width **and** every choice of active tab,
because the interesting case is the *last* tab being active — the
one a prefix-filling planner would drop first — and a test that
only pinned tab 0 would pass with the pin removed entirely.

Above the collapse width only. Below it the strip has no tab slot
at all and the menu holds everything, which
`a_strip_too_narrow_for_both_collapses_to_the_affordance` covers
and this test must not contradict.

### `fn every_tab_is_either_shown_or_reachable_through_the_menu`

Every tab is either drawn in the strip or reachable through the
menu, and the menu exists exactly when it has something in it.

The biconditional is asserted from one button's width upward, which
is a real boundary rather than a fudge: below it nothing can be
drawn in the area at all, and "the affordance is missing" describes
a strip that does not exist. See [`plan_tab_strip`]'s header.

### `fn the_strip_and_the_menu_both_keep_the_manifest_order`

The visible set is *not* a prefix — the pin makes sure of that —
but it is still ascending, and so is the menu. A strip whose
left-to-right order depended on the window width would move every
target under the operator's cursor as they resized.

### `fn a_contextual_tab_arriving_into_a_full_strip_goes_to_the_menu`

[`super::super::tabs::visible_tabs`] appends contextual tabs last,
so this is stated as "the tab that appeared at the end". Three
claims, and the middle one is the rule:

1. Adding it does not change which *other* tabs are shown when the
   strip was already full — the appearance of a Format tab must not
   reshuffle the strip under the operator's cursor.
2. The active tab is still shown afterwards.
3. The menu's count went up by one, which is how the new tab is
   announced — [`super::overflow_label`] puts that count in the
   affordance.

### `fn widening_the_strip_never_hides_a_tab_that_was_visible`

Monotonicity is what makes a window resize feel like a resize
rather than like a reshuffle. It is inherited from [`plan_band`]'s
greedy fill, but the pin, the collapse and the shared affordance
floor all sit on top of it and any of them could break it.

The collapse is the interesting boundary: crossing it upward must
*gain* the pinned tab, never lose one.

### `fn a_crowded_strip_keeps_both_the_pinned_tab_and_the_affordance`

The place this planner deliberately differs from [`plan_band`]. In
the band the affordance takes absolute priority and the groups may
get zero, because every group is still reachable. Here both must
survive: the affordance is the only route to the hidden tabs, and
the pinned tab is not reachable through it.

### `fn a_strip_too_narrow_for_both_collapses_to_the_affordance`

The one place the pin is deliberately given up, and the reasoning is
in [`plan_tab_strip`]'s header: the alternative is one visible tab
and no route at all to any of the others, which is failure
mode #8 in its original form. Reachability wins over pinning
because an unreachable tab is a lost capability and a hidden active
tab is a confusing one.

### `fn a_strip_that_fits_reserves_nothing_and_an_empty_one_plans_nothing`

The first is the "no permanent tax" half of the biconditional; the
second is the first frame of an application that has not built its
manifest yet.

### `fn one_over_wide_tab_truncates_rather_than_growing_an_empty_menu`

The corner where the pin and the biconditional could contradict
each other. There is exactly one tab, it does not fit, and it is
active. Nothing is hidden, so nothing may be reserved: a
"⏷ 0 more" button would be a control that opens an empty menu.

### `fn with_no_active_tab_the_strip_fills_from_the_front`

An empty manifest, or the frame before `resolve_active` has run.
The pin is the only thing that makes the visible set non-prefix, so
without one the strip must behave exactly like a band.

### `fn a_non_finite_strip_width_degrades_safely`

The same guard [`plan_band`] carries, for the same reason, and it
has to be re-asserted here because this function does its own
arithmetic before delegating. At zero width the area collapses, so
the honest answer is that everything is in the menu and there is no
room to draw even the affordance.

### `fn a_row_with_no_trailing_region_divides_exactly_as_it_did_before`

The first thing a claimant on a shared budget owes the others: it
costs nothing when it is not there. Every other assertion in this
file is about the three-region row, so if the fourth region's
arithmetic moves any of them, the fourth is the one that is wrong.

### `fn a_trailing_region_never_consumes_the_row`

`no_reservation_may_leave_the_tabs_with_nothing` restated for the
fourth claimant, and the assertion that makes reserving one safe.
A trailing control four times wider than the whole window
must still leave every load-bearing region what it had; the only thing
that may give is the trailing region itself.

### `fn a_trailing_region_below_its_floor_is_dropped_whole_and_disclosed`

The three-way `grant` rule, which this module's header measures: a
width between zero and the floor does not produce a smaller button, it
produces a button drawn *outside its own rectangle* — here, on top of
the mode selector, where a misplaced click changes the operator's mode
instead of opening their document elsewhere.

And the drop is announced. `trailing_dropped` is a separate flag from
`trailing == 0.0` precisely because the ordinary state of this region
is to be empty, and announcing that every frame would bury the one
case worth reading.

### `struct RowDemand`

A struct rather than five positional `f32`s because four of the five
are widths and a transposed pair would compile, run, and produce a row
that is subtly wrong at exactly the widths nobody checks by hand.

### `struct RowPlan`

Returned by [`plan_strip_row`]. Widths, not rectangles — this module
has no coordinate system; [`super::super::strip`] turns these into
rects.

### `struct StripPlan`

Returned by [`plan_tab_strip`]. `shown` and `hidden` are **indices**
rather than counts because, unlike a band, the visible set is not a
prefix — the active tab is pinned into it wherever it sits. Both are
ascending, so the strip and the menu each keep the manifest's order.
