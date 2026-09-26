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
