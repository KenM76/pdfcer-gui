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

### `fn ribbon_band`

# The one custom item, and why it is not a command

`Item::Custom` is `egui-shell`'s extension point for a control that is
not a button — its own doc names *"a split button with a gallery"* —
and the Recent menu is one: a `Command` item can only render as a
button, and a button cannot ask *which* of ten documents. The renderer
therefore draws and reports, nothing else: the path is parked in
[`Self::recent_choice`] and the `file.recent` token is returned, so the
command goes through [`Self::dispatch_command`] exactly as a ribbon
click does. See [`crate::app::recent::menu`] for the control itself and
[`crate::shell::manifest::CUSTOM_BACKED`] for why the command is on no
tab. An unknown `kind` draws **nothing** and returns `None`, which is
why the manifest's unbuilt `colour_swatch` leaves a gap rather than a
mystery widget.

### `fn docks`

The dock knows nothing about PDFs — it is handed opaque
[`egui_shell::dock::PanelId`]s and hands them back, and this closure
is the single place a `PanelId` becomes a `crate::panels::Panel`.
One dispatcher, exactly as the ribbon has one: an id that does not
resolve draws its own explanation rather than an empty pane, because
an empty pane is indistinguishable from a panel that had nothing to
say.

### `fn central`

Each non-open state renders **one sentence and nothing else**. There
is deliberately no "Open…" button, no retry, and no password field:
S0 opens the file named on the command line and has no other way to
open anything, so a control here would either not exist or not work.
Saying plainly what happened, and what the operator can do about it
outside the application, is the honest version of that.
