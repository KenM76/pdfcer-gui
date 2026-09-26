# `egui-shell/menu/render`

## Item notes

### `struct RowPlan`

A struct rather than eight parameters, and not only for the lint: every
field here is a *decision already taken* — by [`plan::resolve`], by
[`measure`], by [`plan::icon_slot`] — and a positional list of
interchangeable flags is the shape where two of them get swapped and the
result still compiles.

### `fn measure`

Measured with [`crate::ribbon::measure::text_width`] and
[`crate::ribbon::measure::button_padding`] — **the ribbon's own
functions**, not copies. A menu row and a band control are both
`egui::Button`s, and two surfaces that measured the same text with
different constants would disagree about how wide the same command is
for no reason a reader could find.

### `fn close_if_open`

The second half of decision 1. `egui` tracks a popup's open state in
memory, not by whether anyone drew it, so a menu that stops being drawn
stays "open" and reappears the moment it is drawn again — at the old
pointer position, with no right-click behind it.

### `fn close_containing_menu`

Guarded rather than calling [`egui::Ui::close`] unconditionally,
because `close` logs a warning when there is no closable parent — and
[`ContextMenu::render`] is explicitly allowed to be called on a `Ui`
that is not a popup at all. A warning on a supported use is a warning
that teaches people to ignore warnings.

### `struct ContextMenu`

The plain entry points are [`Menu::attach`] (the one an application
wants) and [`Menu::show`] (for an application that owns its own popup).
This builder exists for the four optional capabilities, each of which
is a seam that keeps a domain concern out of the shell:

| Builder method | Supplies | Why the shell cannot do it itself |
|---|---|---|
| [`Self::with_icon_painter`] | how to draw an icon key | An icon set is a licensing and rasterization decision. |
| [`Self::with_custom_items`] | how to draw a non-button row | Otherwise the item vocabulary grows a variant per widget. |
| [`Self::reporting_rects_to`] | where to publish drawn rects | Only the harness knows what it wants to assert. |
| [`Self::with_shortcuts`] | chord hints from outside the manifest | An application may hold its accelerators in a platform table rather than in the keymap. |

All four are optional; without them a menu draws labels, no glyphs, the
manifest's own chords, and publishes nothing.

### `fn with_icon_painter`

The **same** callback type the ribbon takes, so an application
wires its icon set once. Without one, rows draw their labels and no
glyphs — a working menu, which is the point.

### `fn with_shortcuts`

For an application whose accelerators live outside the manifest —
a platform menu table, an inherited binding scheme. It is an
override rather than an addition, because two sources for one
chord is precisely the drift this crate refuses elsewhere: one of
them wins, and it should be the one the caller named.

### `fn attach`

**The entry point an application wants.** It owns the popup, which
is what lets it honour decision 1 in the module header: a menu with
nothing to offer is never opened, and an open menu whose offer
evaporates is closed.

`response` must come from a widget that senses clicks —
`egui::Label` does not by default; `Label::new(..).sense(Sense::click())`
does.

### `fn render`

For an application that owns its own popup, or that embeds a menu
in a panel. Prefer [`Self::attach`]: this entry point cannot decide
*not to open*, because by the time it runs the popup exists.

It does the next best thing — if the menu turns out to have nothing
on offer it draws nothing and asks the containing menu to close, so
the worst case is one frame of an empty popup rather than a
persistent one. A caller that wants the right-click to do nothing
at all must ask [`Menu::would_open`] first, which is exactly what
[`Self::attach`] does for it.

### `fn would_open`

Pure, cheap, and the same question [`ContextMenu::attach`] asks
itself. Public because an application may want to answer it for
another reason — deciding whether to draw a "⋯" affordance beside a
row, say, which should appear exactly when a right-click would do
something.

`false` when the context has no menu, when every command it names
is missing from this build, or when every command it names is
disabled. See [`plan`]'s rule 2.

### `fn show`

`catalog` is normally the [`crate::Shell`] — see
[`MenuLookup`] on why the parameter is a trait rather than that
type.
