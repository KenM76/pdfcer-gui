# `egui-shell/dock/float`

## Item notes

### `fn float_then_dock_puts_the_panel_back_where_it_was`

The single most important property in this module, and the one an
operator will test within ten seconds of finding the command. A
round trip that returned the panel to a *different* stack would be
a command that quietly edited an arrangement nobody asked it to
touch.

### `fn docking_back_brings_the_panel_to_the_front_of_its_stack`

Without this, docking a window into a three-tab stack whose front
tab is something else is a command whose only visible effect is
that a window vanished.

### `fn a_home_that_no_longer_exists_still_docks_the_panel`

The operator floats a panel out of the second column, then closes
everything else in that column so it is pruned, then docks the
float back. The recorded address names a column that no longer
exists, and [`DockLayout::dock_back`] rebuilds it rather than
refusing — it inserts the missing column and stack and clamps only
what is genuinely past the end.

### `fn closing_a_floating_panel_removes_its_float_entry`

A leaked entry would draw a window every frame for a panel the
operator closed, and no command would offer to close it again
because every "is it open" query would say no.

### `fn a_floating_panel_cannot_be_mounted_a_second_time`

The View ▸ Panels group calls `mount` for a panel that is not
showing. If `contains` ignored floats, choosing a floating panel
there would put it in the dock *and* leave it in its window —
two surfaces drawing one panel from two `Ui`s with the same widget
ids.

### `fn collapsing_the_home_side_does_not_hide_a_float`

The float has no side, so collapsing the dock it came from cannot
hide it. Worth pinning because `is_on_screen`'s docked branch
consults `side.visible`, and an implementation that checked the
*home* side would get this wrong in a way nobody would notice until
they collapsed a dock.

### `fn normalize_drops_a_float_whose_panel_is_also_docked`

Not reachable through any verb here; reachable through a
hand-edited `layout.ron`, which is a supported way to configure
this application.

### `fn a_float_survives_serialization`

This is the "remembers, per mode, the way docked width already
does" claim, asserted against the actual serializer rather than
against the intent.

### `fn a_layout_with_no_floating_key_loads_with_no_floats`

The backward-compatibility claim, asserted rather than asserted
*about*. A `layout.ron` with no `floating` key is what every
existing installation has.
