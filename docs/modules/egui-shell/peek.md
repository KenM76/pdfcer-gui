# `egui-shell/peek`

## Item notes

### `fn a_remembered_overlay_cannot_start_a_reveal`

Planted state rather than a default one: `overlay` is written by hand to
last frame's band while `revealed` is false, which is the exact
configuration a "remember where it was" implementation reaches after the
pointer leaves and comes back. A pointer standing in the middle of that
remembered rectangle must NOT bring the band back, or the band's own
area becomes a second trigger — and a trigger whose position depends on
the thing it triggers is R128's loop.

### `fn a_still_pointer_settles_and_stays_settled`

Swept over a grid of pointer positions covering the strip, the band and
the document, forty frames each. Two consecutive frames with the same
input must give the same answer — which is what "monotone decreasing
while the pointer is still" means operationally, and is the property a
"don't ask twice" guard would only appear to have.

### `fn a_trigger_too_small_to_hit_makes_the_surface_stop_hiding`

Walked across the whole width series rather than at the two endpoints,
because a floor asserted only at 0 and at 8 would pass for an
implementation that used `<= 0.0`.
