# `pdfcer-gui/app/surfaces`

## Item notes

### `fn stack_key`

Returns whether the region was published — `false` means the dock laid it
out somewhere the operator cannot see it, and the trace stays silent about
it on purpose.

A stack's address as one grep-able token: `left.0.1`.

The same shape [`egui_shell::dock::DockSide::key`] is built for — an
identifier a harness matches on, never a label an operator reads — extended
with the two indices, because a side alone does not name a compartment and a
check that could only say *"the right dock"* could not tell the two stacks of
a column apart.

### `fn target_key`

Three shapes for the three variants rather than one flattened set of
fields, because the variants do not share an address space: a `Tab`'s `gap`
counts tabs, a `Stack`'s counts stacks and a `Column`'s counts columns, and
printing all three as `gap=` would invite a check to compare two numbers
that are not in the same units.

### `fn rows`

Deliberately the real [`Dock`], not a hand-built list of rectangles.
The rectangles that matter here are the ones `egui` and the dock's own
geometry produce together at a window size nobody laid out for, and a
fixture written by hand could only contain the numbers its author
already expected.

### `fn at_an_ordinary_window_size_the_visibility_gate_drops_nothing`

The companion to the test above, and the one that catches
over-application. Every region the dock publishes at 1280 × 800 is
fully inside its clip, so the gate must be a no-op there. If this ever
fails, the filter has begun eating regions that driven checks
legitimately need — in silence, because that is what the gate does.
