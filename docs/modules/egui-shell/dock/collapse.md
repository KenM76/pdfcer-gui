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
