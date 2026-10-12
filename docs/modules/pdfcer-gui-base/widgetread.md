# `widgetread` — a widget's free angle and opacity, read from the file

`EditSession::turn_widget` takes the free angle absolutely, so a rotate drag
must know the current angle before it commits. The engine has no public reader
for either the angle or `/CA` (request G177); this module reads them from the
object graph until it does, and is deleted with that answer.

# Contract

- `free_turn(session, widget) -> Option<FreeTurn>`: `None` when the box is
  upright or its appearance cannot be read.
  - The normal appearance (`/AP /N`, a reference or a direct stream) must carry
    a `/BBox`; a missing or malformed `/Matrix` reads as identity.
  - The matrix counts as a rotation only when `|a−d|`, `|b+c|` and
    `|hypot(a,b)−1|` are all under 1e-6.
  - The free angle is `atan2(b,a)` in degrees, less the `/MK /R` quarter turn,
    normalised into (−180, 180] and rounded to 9 places. A multiple of 90 is
    upright (`None`).
  - `FreeTurn{degrees, w, h}`: counterclockwise degrees, and the unturned box's
    size from the `/BBox`, swapped when the quarter turn is 90 or 270.
- `opacity(session, widget) -> Option<f64>`: the widget's `/CA`, when it is a
  number.
- `normalise(deg)` and `is_quarter(deg)` are the same rules `canvas::widgetturn`
  commits with.

# Differences from the engine's private reader

The engine's `free_turn` tries N, then R, then D, and only follows references.
This reader looks at N only and also accepts a direct stream. Both agree on
every widget `turn_widget` itself writes, because it writes N.
