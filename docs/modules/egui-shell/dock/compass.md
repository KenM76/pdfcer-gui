# `egui-shell/dock/compass`

## Item notes

### `fn geometry_for`

Deliberately not [`super::super::plan`]: this is a plausible geometry,
not the real one, and the compass must not care which. What it must
share with the real one is the relationship the resolution depends on —
the strip along the top of the compartment — and that is asserted
directly by `over_the_tab_strip_is_a_boundary_between_tabs`.

### `struct Compass`

Cheap enough to build per frame and per candidate; it holds two rects and
derives everything else. See the module header for the mitre rule that
makes [`Compass::zone_at`] and [`Compass::outline`] one definition.

### `fn new`

The edge bands are a fraction of each dimension so that a small
compartment's edges stay reachable, capped so that a large one's centre
stays dominant — the centre is the common release, and an edge split is
the deliberate one.

### `fn resolve_drop`

`None` when the pointer is over no compartment this geometry recorded —
the canvas, a splitter, a side that drew nothing — which is the same
answer as *"a release here docks nothing"*.

Reads both the geometry and this layout because the two halves of the
question need different sources: *which compartment* is a rect
question, and *which boundary within it* is a count question. The
centre zone appends, so it needs the tab count and cannot be resolved
from rects alone.

### `fn body_of`

The strip spans the compartment's full width along its top, so subtracting
it leaves no band that belongs to neither. A stack with no strip recorded —
too narrow to draw one — is divided whole.

Visible to [`super::overlay`] because the rectangle the overlay fills must
be the rectangle the resolution divided. Two spellings of "the compartment
less its strip" would put the zones the operator sees a strip's height away
from the zones the release is read against.
