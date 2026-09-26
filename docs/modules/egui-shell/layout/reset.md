# `egui-shell/layout/reset`

## Item notes

### `fn resetting_re_docks_a_floated_panel`

`crate::dock::float::honour_position` is a *heuristic* about a
window whose monitor has been unplugged, and its own docs say so.
This is the part that is not a guess: whatever the desktop looks
like, a reset puts the panel back in the dock, where it is on the
same monitor as the application window by construction.

### `fn a_scoped_reset_leaves_a_float_from_the_other_side_alone`

The module header promises that a side outside the scope is *"not
read, not written"*. A float has no side, so without this the
promise would quietly stop covering half the layout: resetting the
left dock would yank back a panel the operator had floated out of
the right one.

### `fn resetting_an_already_default_layout_with_no_floats_reports_no_change`

The float clause must not make an already-default layout claim it
changed, or an application that saves on `reset`'s return value
writes a file on every press of a command that did nothing.

### `fn resetting_one_side_leaves_the_other_bit_identical`

*"An operator who only wanted the right dock back must not lose
their left one."* Equality on the whole `SideLayout` — its
columns, its shares, its width and its visibility — because a
reset that preserved the panels and lost the widths would satisfy
a weaker assertion and still be the defect.

### `fn resetting_one_side_cannot_leave_a_panel_in_both`

The operator dragged `pages` — which the default mounts on the
left — over to the right, then reset the left. Without the
normalization pass the panel would now be in both docks, which is
the state-drift bug [`DockLayout::normalize`] exists to forbid.
The freshly reset side keeps it, which is the side the operator
just asked to have back.

### `fn a_reset_never_touches_a_saved_workspace`

The judgement call in this module's header, asserted so that a
later "make reset thorough" edit fails a test instead of costing
an operator work they deliberately kept.
