# `egui-shell/dock/overflow_probe`

## Item notes

### `const OVERFLOW_TOLERANCE_PT`

The overflow this catches measures 0.3–0.4 pt, a 1/32-grid multiple;
`0.05` is well under it and well over f32 noise at coordinates near
1400.

### `fn a_body_that_overflows_its_pane_does_not_move_the_sides_frame`

Two assertions, and the second is not implied by the first: the frame
staying put says the guard held; no `overflow.*` region says the
tripwire agrees nothing got past it.

### `fn publish`

`parent` is the ui the side was shown in (its `max_rect` is the window's
content rect, so its outer edge is the edge a right dock must not pass);
`frame` is the response rect `Panel::show` returned.
