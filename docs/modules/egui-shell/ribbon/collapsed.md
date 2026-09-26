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
