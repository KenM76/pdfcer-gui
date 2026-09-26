# `egui-shell/dock/drop`

## Item notes

### `fn insert_at`

Total rather than panicking on a bad index, but not a second gate:
[`DockLayout::move_panel`] has already asked
[`DockLayout::accepts_drop`], and a `false` from here means those two
answers disagree.
