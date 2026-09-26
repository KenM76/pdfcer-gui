# `egui-shell/dock/plan`

## Item notes

### `fn fill_backward`

Returns `active` itself when even the active tab alone does not fit —
the caller's [`fill_forward`] then yields `shown == 0`, and the bar
becomes the lone overflow control. That is the intended degradation,
not an error case.

### `fn a_child_below_the_minimum_is_pinned_and_the_rest_redistribute`

This is the "pinned minimums" half of failure mode #6's rule. A
proportional split alone would let a 0.02-share column render two
points wide, which has no grabbable splitter and is therefore a
state the operator cannot leave.

### `fn a_container_too_small_for_its_minimums_splits_equally_and_still_fits`

The alternative (honour the minimums, overflow the container) puts
a child where nobody can reach it, which is the same class of
defect as failure mode #8.

### `fn resolving_is_idempotent_under_a_round_trip_through_a_narrow_window`

The defect it names is that un-maximising and re-maximising loses
the panel proportions. That can only happen if some pass writes a
*computed* size back into the model. This test states the property
that forbids it: the shares are the input, they are never an
output, and therefore the spans at 900 are the same whether or not
the window visited 200 first.

### `fn a_splitter_moves_exactly_two_neighbours_and_no_others`

The defect it names is that dragging one divider resizes every
column. Four columns, drag the first boundary, and columns three
and four must be **bit-identical** — not "close", identical, because
the function is required not to touch them at all.

### `fn the_visible_tabs_never_encroach_on_the_reservation`

This is the invariant that stops the overflow control from being
drawn past the right edge — present, but partly or wholly
unclickable, which is exactly what "the overflow button itself
gets hidden" means.

### `fn the_active_tab_is_never_the_one_that_gets_hidden`

A prefix plan would fail this the moment the operator selects a
late tab and narrows the dock, leaving a panel body on screen with
no tab naming it anywhere.

### `fn a_bar_narrower_than_its_reservation_keeps_the_affordance_and_drops_the_tabs`

Failure mode #8 is the overflow button itself getting hidden,
leaving no route to the hidden tabs. Here the route survives and
the tabs are what give way, which is the reverse of it and the
whole reason the reservation is the first subtraction.

### `fn an_infinite_available_width_is_treated_as_none_at_all`

`INFINITY − overflow_width` is still infinity, so a naive
implementation shows every tab in a container that will then clip
them, with no affordance.

### `fn the_reservation_covers_the_widest_reachable_label_not_the_longest`

Exercised here with a deliberately non-monotonic measure — the
shape a real proportional face has, where `"⏷ 8 more"` is wider
than `"⏷ 9 more"`. A `max` over `1..=total` is right; measuring
`total` alone is wrong, and the difference is invisible with no
font installed. [`super::width_tests`] repeats this against real
metrics.

### `fn the_minimum_column_width_ignores_tab_labels_entirely`

The defect it names is an invisible, inactive tab whose width
holds the whole dock open — you cannot see it and you cannot
narrow the dock until you close it. It can only arise if a
minimum-size computation walks the tab list. This test states the
property mechanically: give a stack a preposterous label and every
minimum is unchanged, because they are constants.

### `fn both_docks_at_their_minimum_leave_most_of_a_1280_point_window`

Failure mode #4 is a single dock whose minimum consumes a third of
the screen. Both of this shell's docks at their minimum must leave
the application the majority of a 1280-point window — the width the
design rule names.

### `fn the_side_clamp_leaves_the_application_the_majority_of_the_window`

The clamp itself lives in [`super::mod`]'s renderer; this asserts
the constant it is built from is a sane fraction, so a change to it
is a deliberate one.
