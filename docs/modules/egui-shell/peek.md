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

### `struct Peek`

Cheap and `Copy`-free only because it holds a `Rect`; an application keeps
one per surface beside that surface's other presentation state (the ribbon's
in [`crate::ribbon::RibbonState`], the rail's in
[`crate::dock::DockState`]).

### `const MIN_TRIGGER_PTS`

Eight, which is the width Windows gives a window's resize border and
about what VS Code gives its collapsed sidebar edge — a strip a pointer
finds without being aimed. Below it [`Self::resolve`] reports
[`Show::Inline`] and the surface simply does not hide: see the module
header on why the floor fails **open**.

### `const GRACE_PTS`

Four. Without a grace margin the surface closes on the one-pixel seam
between the trigger and the overlay it anchors — a gap that exists
because two rectangles that share an edge do not both contain the points
on it — and the symptom is a band that flickers as the pointer crosses
from the tab strip into the group beneath it. It is added to a
**containment test**, never to a laid-out rectangle, so it changes no
geometry and cannot reach the layout.

### `fn set_mode`

Switching **on** hides the surface immediately unless the pointer is
already over its trigger, which the next [`Self::resolve`] decides.
Switching **off** shows it, and clears the reveal so that turning the
setting on again does not inherit a stale `true`.

### `fn resolve`

`trigger` is the always-present rectangle that reveals the surface —
the ribbon's tab strip, the rail's sliver. It **must not be derived
from the surface's own body**; see the module header, and see
[`Self::MIN_TRIGGER_PTS`] for what happens when it is degenerate.

`pointer` is the latest pointer position, or `None` when the pointer is
outside the window — which closes the surface, because a pointer that
has left cannot be hovering anything.

`holds_focus` is whether a widget inside the overlay currently has
keyboard focus. A **keep** term only.

### `fn record_overlay`

Called by the surface **after** it has drawn, with the rectangle the
body occupied. Until it is called the overlay term of the invariant is
false, which is the safe direction: a surface that drew and forgot to
report closes as soon as the pointer leaves the trigger, rather than
staying open over a rectangle nobody can see.

### `fn overlay`

Published for the region a harness asserts against — see
`crate::verify` — and for a check that wants to prove the overlay does
not move the surface beneath it.
