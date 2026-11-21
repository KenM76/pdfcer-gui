# `widgetread` — a widget's free turn

`EditSession::turn_widget` takes the free angle absolutely, so a rotate drag
must know the current angle before it commits, and its ghost must know the
unturned box it turns. The angle is the engine's `forms::Widget::free_angle`.
The unturned size is not in `forms::Widget` (request G316), so this module
reads it from the appearance until the engine reports it, and is deleted with
that answer. Opacity is `forms::Widget::opacity`, read by the panel directly.

# Contract

- `free_turn(session, widget) -> Option<FreeTurn>`: `None` when
  `Widget::free_angle` is `None`, or the normal appearance (`/AP /N`, a
  reference, a direct stream, or the first state of a state dictionary) has no
  four-number `/BBox`.
- `FreeTurn{degrees, w, h}`: the engine's counterclockwise degrees, and the
  `/BBox` size, swapped when `/MK /R` is 90 or 270.
- `normalise(deg)` and `is_quarter(deg)` are the rules `canvas::widgetturn`
  commits with: (−180, 180], and a multiple of 90 within 1e-6.
