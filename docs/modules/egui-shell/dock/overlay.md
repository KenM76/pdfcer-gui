# `egui-shell/dock/overlay`

## Item notes

### `struct DropPreview`

Published on [`super::DockFrameReport`] because the visible form of this
affordance is a wash of colour over a rectangle: precise to look at, and
nothing a harness can assert on. Same reason as
[`super::drag::TabDragPreview`], which is its sibling for the other half of
the gesture.

### `fn draw`

Draws nothing unless a drag is in flight over a compartment that is not the
strip the drag began in — [`drag::preview`] owns that one and has already
drawn its caret, so this stands down whenever it published.

### `fn offer`

The grammar half of [`draw`], shared with [`super::floatdrag`]. The two
gestures differ only in where the point comes from — `egui`'s pointer, or a
float window's own drag reported by the application — and sharing this is
what stops them disagreeing about which compartment a point is in, which
zone of it, or what the release produces.

### `fn wash`

Not `gamma_multiply`, which scales an opaque colour's channels and so
*darkens* it against the panel instead of letting the panel through. The
zones sit over a panel body the operator must still be able to read.

### `const ARMED_A`

Far enough above [`RESTING_A`] to be unmistakable at a glance, and still
short of hiding the panel underneath — the operator is choosing between five
places in a compartment whose contents are how they recognise it.

### `const OUTCOME_PTS`

Heavier than the zone edges: the zones are the question and this is the
answer, and the answer is frequently somewhere else on the screen entirely —
a column that does not exist yet, on the far side of the dock.
