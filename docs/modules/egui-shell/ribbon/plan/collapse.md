# `egui-shell/ribbon/plan/collapse`

## Item notes

### `fn gains_from`

Measured, not assumed. A group whose three-row layout is no narrower
than its natural one — anything with few enough items — would otherwise
consume a rung of the ladder, change nothing, and make the band appear
to stall at one width and jump at the next.

### `fn width_of`

`n` groups carry `n − 1` separators, and a compacted group still occupies a
slot — it is narrower, not absent. Conflating "compacted" with "gone" is the
mistake that would break the conservation half of
`the_visible_groups_are_a_prefix_and_nothing_is_lost`.

### `fn every_group_rewraps_before_any_group_collapses`

The rung order is the whole of the ladder, and it is the property most
likely to be broken by an "optimisation" that collapses the widest group
first because that converges faster. It does converge faster and it is
wrong: re-wrapping keeps every control on the band and collapsing hides
them all, so five re-wraps beat one collapse however the arithmetic
comes out.

### `fn widening_the_band_never_compacts_a_group_further`

For every pair of adjacent widths, no group's state may move DOWN the
ladder as the band grows. A hysteresis bug — the classic consequence of
planning from the previous frame's answer instead of from scratch —
shows up here as a width at which some group compacts again.

The sweep is one point at a time, deliberately. A claim about what
happens across a range that is checked at two widths is a claim about
two widths: endpoints agreeing is not evidence about what happens
between them.

### `struct Candidate`

Deliberately not the manifest's `Group`: the ladder needs three widths and a
priority, and keeping it that way is what makes it testable without building
a manifest, a registry and a font.

### `fn fit`

Returns a state per group, parallel to `groups`. A pure function of its
inputs, which is the whole of the monotonicity argument in this module's
header.

# The ladder

1. Everything at [`State::Natural`]. If it fits, stop — a band that
   compacted a group it had room for would be making itself harder to read
   for no gain.
2. **Re-wrap** every group that gets narrower by it, in manifest order.
   This rung is exhausted before the next one begins.
3. **Collapse** in authored priority order, skipping any group that
   declines.
4. Stop when it fits, or when the ladder is exhausted — at which point what
   is left over goes to [`super::plan_band`] and its overflow affordance,
   which was always going to be the last resort.

Each step re-measures rather than subtracting a precomputed saving,
because the two are not the same once separators are involved, and the
difference is exactly the kind of one-group-too-many error that shows up
only at a single window width.

### `fn widths_after`

A convenience so the caller does not re-derive the same `match` in a third
place — the states and the widths must agree, and the cheapest way to
guarantee that is to produce them together.
