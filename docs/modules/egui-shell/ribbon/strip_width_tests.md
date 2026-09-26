# `egui-shell/ribbon/strip_width_tests`

## Item notes

### `const STRIP_TABS`

Seven, which is the count `MODES_AND_PANELS.md` failure mode #8 names
(*"past ~6 tabs the overflow button itself gets hidden"*), and enough
that no realistic test window fits them all.

### `fn strip_shell`

Seven ordinary tabs with deliberately unequal label widths, one
contextual tab, the same three modes and the same two-control QAT as
[`super::tests::shell`] — so the row has all four claimants on it
(QAT, tabs, tab affordance, selector) and the reservation order is
actually under test rather than assumed.

Each tab carries **one small group**, so the band never overflows at
the widths these tests use. That is deliberate: a frame in which both
the strip and the band have an affordance is a frame in which a test
asserting "the affordance is on screen" might be reading the wrong one,
and `report::tab_overflow()` and `report::overflow()` exist as separate
names precisely so it cannot.

### `fn nothing_on_the_tab_strip_row_is_ever_off_screen`

The single assertion the whole of [`super::strip`] exists to make true.
It covers all four claimants at once — the QAT, every tab, the strip's
affordance and the mode selector — because the failure is in none of
them individually: it is what happens when only *one* is reserved and the
rest are laid out into whatever is left, which at 180 pt is a negative
coordinate.

The sweep step is 7 pt, which is deliberately finer than a "few
samples" test and deliberately **not** relied on for the estimate
check — see `the_strip_that_claims_to_fit_really_does_fit`, which
binary-searches because a sweep at any step coarser than the estimation
error walks straight over it.

### `fn the_tab_strip_never_runs_under_the_mode_selector_or_the_qat`

The shape an unreserved row fails in, measured at 320 pt: tabs from 188
to 265 under a selector from 142 to 320 — a 77 pt overlap, with the tabs
underneath. Nothing is off screen and nothing looks wrong in the reported
rects; the tabs are simply unreachable, which is why a containment sweep
does not cover this on its own.

Asserted as a geometric relation (`tab.right ≤ selector.left`) rather
than as coordinates, so it survives a fourth mode, a reworded label and
a theme change — which is the whole reason the rects are published.

The QAT is included on the same principle: the row is ordered
`QAT → tabs → affordance → selector` and every adjacent pair must
respect it.

### `fn the_active_tab_is_on_screen_at_every_width_whichever_tab_it_is`

Requirement 2, asserted through the renderer rather than through the
plan. The interesting case is the **last** tab being active — the one a
prefix-filling planner drops first — so every tab in turn is made
active and swept.

What makes this a *rendered* claim rather than a repeat of
`the_active_tab_is_always_shown_and_never_hidden` is the second
assertion: the tab is not merely in the plan's `shown` list, its
published rectangle is inside the window and has area. A plan that
pinned a tab into a zero-width slot would satisfy the pure test and
fail here.

# The one exception, asserted rather than skipped

Below about 47 pt of tab area the strip **collapses**: it cannot hold a
tab and an affordance at sizes `egui` will draw, and #8 decides which
survives — see [`super::plan::plan_tab_strip`]'s collapse section. The
branch below does not quietly `continue` past that; it asserts the
collapsed contract instead, because "the active tab is not in the
strip" is only acceptable while *every* tab is reachable through the
affordance, and a bug that collapsed the strip at 900 pt would
otherwise slip through as a skipped iteration.

### `fn the_strip_collapses_only_at_widths_too_narrow_to_hold_a_tab`

The guard on the exception the test above carves out. A collapse is a
real loss — the strip stops showing which tab is current — so it must
be confined to widths where the alternative is worse, and it must be
**monotonic**: once the window is wide enough for a tab strip, widening
it further can never take the strip away again.

Without this, "the strip collapsed" would be an escape hatch that a
regression could widen indefinitely while every other test kept
passing by taking the collapsed branch.

### `fn the_tab_overflow_affordance_is_hit_testable_under_real_metrics`

A rectangle proves something was allocated; only `egui`'s own hit test
proves it can be reached, because that is what accounts for clipping,
for occlusion by a later widget and for a zero-area interact rect. The
band's affordance has the same test for the same reason
(`the_affordance_is_hit_testable_under_real_metrics`); this is its
counterpart one row up, and the row up is where an unreserved layout
puts controls at negative coordinates.

Three widths: one where several tabs still fit beside it, one where
almost none do, and one narrower than the affordance itself — the case
where [`super::plan::plan_tab_strip`] has to divide the shortfall
between the affordance and the pinned tab.

### `fn the_strip_that_claims_to_fit_really_does_fit`

The estimate-accuracy check for the tab strip, asked through the
renderer, and the one that catches an **under**-estimate — the
dangerous direction, because it means the plan believes nothing is
hidden while a tab is drawn off the edge with no affordance offering
it.

# Why this binary-searches rather than sweeps

The only widths at which an under-estimate is *visible* are those
between "the plan stopped overflowing" and "the content genuinely
fits". An estimate short by 8 pt makes that window 8 pt wide, and a
sweep at any step coarser than the error walks straight over it — which
was measured on the band, not assumed: with the item padding
deliberately cut to a fifth, an 11 pt sweep still reported success.

So the transition width is found exactly (the counts are monotonic in
width, which `the_active_tab_is_on_screen_at_every_width_whichever_tab_it_is`
and the pure `widening_the_strip_never_hides_a_tab_that_was_visible`
pin separately) and the assertion is made **there**, where any
shortfall at all is visible, plus at the two widths above it.

# What it reports

The transition width is printed on failure rather than hard-coded,
because it is a property of the synthetic face and the fixture and
would become a maintenance burden the moment either changed. What is
asserted is the *relationship* at that width, not the number.

### `fn a_contextual_tab_arriving_into_a_full_strip_is_announced_by_the_count`

The same frame twice, once with `selection.any` set and once without,
at a width where the strip is already full. What changes must be
exactly one thing: the affordance's count.

The `.count()` on the affordance is what "announced" means here, and it
is why [`super::plan::overflow_label`] puts the number in the label
rather than drawing a bare chevron — see [`super::a11y`] for what
`egui` 0.35 cannot express beyond that.

### `fn no_tab_is_lost_between_the_strip_and_its_menu`

The counting form of failure mode #8, and the cheapest possible
tripwire on it: a tab that is in neither place is a tab the operator
cannot reach at all.

Also asserts the biconditional — the affordance is drawn exactly when
something is behind it. Both directions are real defects: an affordance
with nothing behind it opens an empty menu, and something hidden with
no affordance is #8 itself.

### `fn the_qat_stays_inside_the_row_at_every_width`

The narrowest possible assertion on a measured symptom: at 180 pt an
unreserved row puts the first QAT control from −6 to a point past the
tabs. It is drawn, it is reported, and it cannot be clicked.

The QAT has no overflow menu of its own — it is a fixed cost, and
`RIBBON_IA.md` treats its contents as the handful of things an operator
uses constantly — so the answers available when it does not fit are
"truncate", "drop the ones that will not fit", and "draw off the edge".
This asserts that the third never happens.

# Two claims, and the second is the one with teeth

1. **Containment, at every width.** Whatever is drawn is inside the
   window.
2. **Presence, above a realistic width.** Containment alone is
   satisfied by a QAT that draws nothing at all, so the sweep also
   pins that the whole QAT really is there at ordinary widths. Without
   it, a regression that dropped every control would pass claim 1
   perfectly.

### `fn picking_a_hidden_tab_from_the_menu_brings_it_into_the_strip`

Opening the menu and clicking a hidden tab must make that tab active
**and** put it in the strip — the second half is what the pin
guarantees, and without it the operator would pick a tab out of the
menu and watch it stay in the menu.

Driven the way an operator would: hover the affordance, press, release,
let the popup render, then click the entry.
