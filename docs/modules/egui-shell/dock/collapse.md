# `egui-shell/dock/collapse`

## Item notes

### `const RAIL_WIDTH_PTS`

Narrow enough that it costs the document almost nothing, wide enough that it
is unmistakably a control rather than a border. Every program in the class
lands in the same range.

### `const RAIL_HIT_PTS`

Larger than the glyph it contains, deliberately. A live target may exceed
the drawn affordance and must never be smaller than it. A chevron is a few
points across and would be a miserable thing to hit.

### `const RAIL_TOP_PAD_PTS`

Aligned with the top of where the panel's own tab bar would be, so
collapsing and expanding do not make the control jump. An operator who
clicks to collapse should find the way back under the pointer they still
have there.

### `fn draw_collapsed_rail`

The minimising half is [`draw_collapse`]; this is the half that makes it
reversible. Without it the only route back is a ribbon command the
operator has to know exists — see the module header on why a panel with
no visible handle is lost rather than minimised.

# It is not drawn for an EMPTY side

A side with no panels in it has nothing to bring back, and a control
that opened an empty compartment would be an affordance for something
that cannot happen — the no-placeholders rule, which this crate holds to
as strictly as its host does. The caller checks `is_empty` first.

# The chevron points where the panel will go

Inward on a collapsed side, because that is the direction the panel
arrives from. The mirror of the collapse control, which points outward.
Getting this backwards is a small thing that makes a control feel wrong
without the operator being able to say why.

### `fn draw_collapse`

Drawn at the top of the side, at the **trailing end of the tab row** —
the right-hand end on both sides, which is the inner edge for the left
dock and the outer edge for the right one. The inline note at the
placement says why the tab row, not the canvas edge, is the constraint
that binds.

# It raises an intent rather than writing the layout

Everything in this crate that changes the layout does, and here it is
load-bearing rather than ceremonial: flipping `visible` mid-frame would
change the width of a panel that has **already laid out inside it**, so
the frame would draw a body at one width inside a container at another.
The apply phase runs once, after every side has drawn.
