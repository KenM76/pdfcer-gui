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

### `const MIN_COLUMN_WIDTH`

A constant, not a measurement — see the module header on failure mode
#3. At this width a tab bar still holds the overflow affordance plus
at least one truncated tab, which is checked by
`the_minimum_column_width_still_admits_the_overflow_affordance`.

### `const MIN_STACK_HEIGHT`

Enough for a tab bar plus two rows of content. Below that a stack is
a header with nothing under it, which reads as a rendering fault
rather than as a small panel.

### `const MAX_SIDE_FRACTION`

This is a **presentation clamp, and it never writes back to the
model** — that is the whole point, and it is failure mode #6's design
rule applied to the one dimension the operator sets in absolute
points:

> Store proportional sizes with pinned minimums. **Restore**, do not
> recompute.

A layout saved on a 3840-point-wide monitor may carry a 900-point
dock. Restoring that on a 1280-point window would hand 70 % of the
screen to one dock. Clamping at draw time keeps the window usable;
*not* writing the clamped value back means re-maximising restores the
operator's 900 exactly, rather than leaving them permanently with the
576 that the small window happened to allow. A dock that loses its
width every time you un-maximise is failure mode #6; this is the
two-line answer to it.

### `const SPLITTER_THICKNESS`

Also the hit target. `egui` grows a drag target's *interaction* rect
beyond its visual one only if asked; [`super::splitter`] asks, so the
visual line can stay thin without the grab becoming a game of skill.

### `const MIN_TAB_WIDTH`

A tab narrower than this shows no useful part of its label, and an
unreadable tab is worse than a menu entry: it occupies the space that
would have let a readable one fit, and it tells the operator nothing.

### `const MAX_TAB_WIDTH`

Without a cap, one panel with a long descriptive name consumes an
entire tab bar and pushes every sibling into the overflow menu — a
stack's worth of reachable tabs traded for one unabbreviated title.
Beyond this width the label truncates with an ellipsis and the full
text stays available as the tab's tooltip and accessible name.

### `const TAB_BAR_HEIGHT`

Fixed rather than content-derived, and that is not laziness: a tab bar
whose height depended on its content would make the body rectangle
depend on the tab list, which is failure mode #3 in the vertical
direction.

### `fn sane_length`

A layout pass can hand out `f32::INFINITY` for available space inside
an unbounded container, and `INFINITY − overflow_width` is still
infinity — which would silently disable overflow rather than show
everything. A `NaN` propagates through every comparison as `false`,
which turns "does it fit?" into "no" for the rest of the frame. Both
become zero here, so the degenerate case is the *safe* one.

### `fn resolve_spans`

This is the function that makes failure mode #6 impossible, and the
property that does it is that **it is pure**. It reads `shares`, it
reads `total`, and it returns spans. It has no memory of previous
calls, it writes nothing back into the model, and no renderer is
permitted to store its output anywhere the model can see. Therefore
resolving at 400 points and then again at 800 gives exactly what
resolving at 800 would have given in the first place, and
un-maximising a window cannot cost the operator their proportions.

`the_layout_survives_a_round_trip_through_a_narrow_window` asserts
exactly that, because "we simply never write it back" is a claim about
the whole crate that no local reading can verify.

# Arguments

- `shares` — relative weights, one per child. Non-finite or
  non-positive weights are treated as [`MIN_SHARE`], because a zero
  weight would mean "this child gets no space", which is a state the
  operator cannot undo by dragging (a zero-width column has no
  splitter to grab).
- `total` — the space to divide, **including** the gaps between
  children.
- `min` — the pinned minimum for every child.
- `gap` — the space between two adjacent children, applied `n − 1`
  times.

# The algorithm

1. Subtract the gaps. What remains is the *content* budget.
2. If the content budget cannot even give every child its minimum,
   split it **equally** and return. This is a deliberate,
   disclosed degradation: at that size no assignment satisfies the
   minimums, and an equal split is the only answer that does not pick
   a winner. The alternative — honour the minimums and overflow the
   container — puts children off the edge of the dock, which is the
   same class of defect as failure mode #8 (a control drawn where
   nobody can reach it).
3. Otherwise assign proportionally; pin any child that came out below
   `min` at exactly `min`; redistribute the remainder among the
   unpinned children in proportion to their shares; repeat until no
   new child needs pinning. The loop terminates because each pass
   pins at least one child and there are finitely many children.

### `fn drag_boundary`

**This is failure mode #7's design rule as code**: *a splitter affects
its two neighbours only*. The function takes a mutable slice and
writes to exactly two indices. There is no renormalisation pass, no
"redistribute the remainder", and no total to preserve by adjusting
everyone — because the sum of the two changed spans is invariant, the
total is preserved for free and every other child is untouched by
construction rather than by care.

Returns the delta that was actually applied, which differs from the
requested one when a neighbour hit `min`. The caller uses it for
nothing today; it is returned because a caller that wanted to show
resistance at the limit would otherwise have to re-derive it, and
re-derived numbers drift.

A `boundary` index out of range is a no-op returning `0.0`, not a
panic: the index comes from a hit test on a rectangle, and a
rectangle can outlive the model it was computed from by exactly one
frame when a panel is closed while its splitter is being dragged.

### `fn spans_to_shares`

Called **only** after [`drag_boundary`], never after a plain resolve.
That restriction is the other half of failure mode #6: if a renderer
wrote spans back into shares every frame, then a frame drawn in a
narrow window would bake the narrow window's pinned minimums into the
model permanently, and the proportions would be gone before the
operator even noticed the window had been resized.

The result is normalised to sum to `1.0` so that stored shares are
comparable between sides and across sessions, and so a diff of two
saved layouts is readable.

### `fn tab_width`

Floored at [`MIN_TAB_WIDTH`] so the arithmetic stays meaningful with
no font installed, and capped at [`MAX_TAB_WIDTH`] so one long label
cannot evict every sibling into the overflow menu.

### `fn overflow_label`

The chevron is part of the label rather than a separate glyph so the
control is one measurable string, and so a build with no icon set
still shows an affordance rather than an empty button. Identical in
spirit and in wording to the ribbon's, because an operator should not
have to learn two overflow idioms in one window.

The chevron is `⏷` U+23F7 and **must stay in step with the ribbon's**
— see `crate::ribbon::plan::overflow_label`, which carries the account
of why `⌄` U+2304 renders as tofu in the pinned font and which near
misses are also missing from it. "Identical in wording" is the promise
above, and identical *codepoints* is what keeps it.

### `fn overflow_width`

# Why every reachable label is measured, not just the largest count

The hidden count is not known until [`plan_tabs`] has run, and
[`plan_tabs`] needs this number as an input. The circularity has to be
broken by reserving for a label that has not been chosen yet, and the
only safe direction is the worst case.

Reserving for `"⏷ N more"` at `N = total` alone is the intuitive
choice and it is **wrong**: that is true of the *character count* and
false of the *width*. With no font installed every label measures zero
and the two agree; with real metrics `"⏷ 8 more"` is wider than
`"⏷ 9 more"` in any face whose digits are not tabular, and a stack of
nine tabs showing one would then draw a control wider than the space
reserved for it — the affordance overhanging the tab bar's right edge,
which is failure mode #8 with the control present but partly
unclickable.

[`crate::ribbon::plan::overflow_width`] reserves on the same terms for
the same reason. The argument is written out here rather than left as a
cross-reference, because the next person to write a third overflow
control will read this file, not that one.

So the reservation is `max` over every label the control can ever
display: `1..=total`. That makes *"the drawn control never exceeds its
reservation"* a proof rather than an argument about digit shapes.

### `struct TabPlan`

Returned by [`plan_tabs`]. Every field is a decision the renderer then
obeys without re-deriving anything, so the arithmetic exists in
exactly one place and can be tested without a window.

### `fn plan_tabs`

# Why the visible set is a *window* and not a *prefix*

[`crate::ribbon::plan::plan_band`] shows a **prefix** of its groups,
and states why: the manifest's order is the operator's order, and a
plan that dropped a group from the middle would make the visible
band's order depend on the window width.

A tab bar cannot use that rule, because of an additional constraint a
ribbon band does not have: **the active tab's body is being drawn
underneath.** A prefix plan hides the active tab whenever the operator
selects a later one and then narrows the dock — leaving a panel body
on screen with no tab anywhere naming it. That is failure mode #11
(*selecting a tab must invalidate what is painted*) arriving from the
other direction: the paint is right and the tab is missing.

So the visible set is the widest **contiguous window** that contains
the active tab, preferring the window that starts at zero. Contiguity
preserves order — the property the prefix rule was protecting — and
containing the active tab keeps the bar honest about what is on
screen. It is the behaviour of every editor tab strip that scrolls.

# Arguments

- `widths` — each tab's planned width, in model order, from
  [`tab_width`].
- `active` — the index of the active tab. Out of range is clamped to
  the last tab rather than panicking; the index arrives from a
  deserialized layout, and a file that says `active: 9` on a
  three-tab stack is a fail-soft input, not a crash.
- `available` — the tab bar's usable width in points.
- `gap` — the space between two adjacent tabs, and between the last
  tab and the overflow control.
- `overflow_width` — from [`overflow_width`].

# The algorithm

1. If everything fits, show everything and reserve nothing. An
   overflow control that took space when there was nothing to overflow
   into it would be a permanent tax on every dock in the application.
2. Otherwise `tab_budget = available − overflow_width − gap`, **clamped
   at zero**. This is the line the whole module exists for.
3. Fill `tab_budget` greedily from index 0. If that window contains
   the active tab, use it — this is the stable, no-jitter case and it
   covers every stack the operator has not scrolled.
4. Otherwise build the window backwards from the active tab, then
   extend it forwards with whatever is left. The active tab is
   therefore in the window whenever the window is non-empty.
