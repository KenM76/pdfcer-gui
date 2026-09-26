# `egui-shell/ribbon/collapsed`

## Item notes

### `const CHEVRON`

The same glyph the overflow affordance uses, deliberately: an operator who
has learned what `⏷` means at the end of the band should not have to learn
a second symbol for the same promise three inches to the left.

### `const SIDE_PADDING`

Matches `sizing`'s `LARGE_SIDE_PADDING`, because a collapsed group sits in
the band beside Large controls and a different inset would read as a
misalignment rather than as a different kind of thing.

### `fn a_collapsed_group_is_at_least_its_padding_wide`

The width feeds the ladder, and a zero would make the ladder believe
collapsing is free, which is how every group ends up collapsed at a
width where two would have fitted.

### `fn width`

Measured from the caption, since that is the only thing whose width can
vary. The ladder needs this before it can decide anything, which is why it
is a free function taking a `&Ui` rather than something the renderer
returns — a width that were only known after drawing would be a
measurement fed back into a layout, which is the feedback loop R128
forbids.

### `fn render`

`rows` is the split the group *would* have had expanded, passed through
untouched so the popup is identical to the band's rendering. `box_` is the
band's box, used for the button's height only — the popup gets
[`GroupBox::NATURAL`], so it is as tall as its own content.
