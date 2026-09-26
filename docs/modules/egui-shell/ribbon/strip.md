# `egui-shell/ribbon/strip`

## Item notes

### `fn island`

The `id_salt` is fixed per region rather than derived from the content,
so `egui`'s per-id state — focus, hover, an open popup — survives a
resize that moves a tab into or out of the menu. See
[`super::ctx::Ctx::id`] on why that matters: an id that shifts with the
layout produces a control that loses keyboard focus when the window is
dragged, which reads as a focus bug rather than as an id bug.

### `fn render_affordance`

Modelled on [`super::band`]'s, deliberately, down to the `min_size` and
the `truncate()`:

- `min_size(rect.size())` makes the control exactly as big as the
  arithmetic promised, so the reservation is not quietly under-spent.
- `truncate()` is the other half. Without it a label wider than the
  rect makes the *button* wider than the rect, and the affordance hangs
  into the mode selector in precisely the situation — a crowded row —
  where it most needs to be reachable. Truncating spends the shortfall
  on characters, which is recoverable: the tooltip states the count in
  full.

Returns the tab the operator picked, if any, and the affordance's
`egui::Id` so a harness can hit-test it.

### `fn disclose_tab_plan`

Kept out of [`render`] so the drawing code reads as drawing. Each event
is a separate fact — a strip that overflowed, a pinned tab that
truncated, an affordance that was crowded — and a harness that wants
only one of them should not have to parse the others out of a combined
line.
