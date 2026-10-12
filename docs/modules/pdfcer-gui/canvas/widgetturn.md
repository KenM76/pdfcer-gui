# `canvas::widgetturn` — a selected form field's box turned by its rotate grip

# Contract

- `target(ctx, doc)` reads the selected widget once per document revision:
  field name, widget index, free angle, `/MK /R`, and the unturned size. It is
  cached in egui temp memory keyed by `(path, edit_epoch, field, widget)`.
- `offer(ctx, doc)` is the grip set: all nine grips upright, the rotate grip
  alone while turned. `pressing` and `forms::selecting` both call it, so the
  grips painted and the grips hit-tested are the same set.
- `commit(target, degrees, actions)` turns a release angle (counterclockwise,
  page space) into actions:
  - the new angle is `free + degrees`, normalised;
  - landing on a non-zero quarter turn: `TurnWidget{0}` first when the box is
    turned, then `RotateWidget{quarter + steps·90}`;
  - landing on 0: `TurnWidget{0}`;
  - otherwise `TurnWidget{total}`.
  It traces `widget-turn-commit`.

# Why a quarter turn becomes `/MK /R`

`/MK /R` is the one rotation every viewer keeps, including one that
regenerates appearances. A free angle lives only in the appearance `/Matrix`.
So a drag that lands on a quarter turn is stored where it survives.

# Why the rotate grip alone while turned

`resize_widget` refuses a turned box (`WidgetTurned`). Offering the resize
grips would offer a gesture that can only be refused.
