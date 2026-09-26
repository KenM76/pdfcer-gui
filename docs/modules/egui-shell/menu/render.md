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
