# `widgetread` — a widget's free turn

`EditSession::turn_widget` takes the free angle absolutely, so a rotate drag
must know the current angle before it commits, and its ghost must know the
unturned box it turns. Both are the engine's: `forms::Widget::free_angle` and
`forms::Widget::free_size`. Opacity is `forms::Widget::opacity`, read by the
panel directly.

# Contract

- `free_turn(widget) -> Option<FreeTurn>`: `None` when `Widget::free_angle`
  or `Widget::free_size` is `None` (the engine sets both or neither).
- `FreeTurn{degrees, w, h}`: the engine's counterclockwise degrees, and
  `free_size` swapped when `/MK /R` is 90 or 270, since the engine gives it
  before that swap and the ghost is drawn in the `/Rect` frame.
- `normalise(deg)` and `is_quarter(deg)` are the rules `canvas::widgetturn`
  commits with: (−180, 180], and a multiple of 90 within 1e-6.
