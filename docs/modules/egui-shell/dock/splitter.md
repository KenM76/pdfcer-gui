# `egui-shell/dock/splitter`

## Item notes

### `fn the_idle_splitter_is_distinguishable_from_the_panel`

A boundary drawn in the panel's own colour is invisible, and an
invisible boundary is never discovered to be draggable — the
silent half of failure mode #1. Checked for every shipped preset,
because a colour pair that holds in one theme and collapses in
another passes any check that looks at a single palette.

### `fn a_zero_area_splitter_does_nothing`

It happens for exactly one frame when a compartment closes while
its splitter is on screen, and registering a widget at an
impossible position would leave `egui` holding focus somewhere
unreachable.

### `fn splitter`

`rect` is the **interactive** rectangle — the full
[`super::plan::SPLITTER_THICKNESS`] — and the painted rule is a
centred sliver of it. See the module header on why those are
different sizes.

### `fn idle_colour`

A separator drawn in a colour indistinguishable from the panel it
sits on is invisible, and an invisible boundary is one the operator
never learns is draggable. The theme's own contrast gate covers text
pairs; this is the one non-text pair the dock adds, and it is checked
by `the_idle_splitter_is_distinguishable_from_the_panel`.
