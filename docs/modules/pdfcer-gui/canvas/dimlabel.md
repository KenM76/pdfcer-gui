# `canvas::dimlabel`

Sliding a selected linear ce dimension's text along its dimension line, the
line staying where it is.

## Contract

- `screen_quad(ctx, doc, map, selection)` gives the text's four corners in
  screen space. They come from `EditSession::dimension_preview` of the
  dimension as committed (`appearance.label_quad`), cached in egui temp data
  keyed on the dimension's id and kind, so a selected dimension is baked once,
  not every frame. `canvas::painting` publishes their bounds as
  `canvas.dimension-label`. Nothing is drawn: the text is the affordance.
- `label_at(ctx, doc, map, selection, screen)` says whether a press landed in
  that quad, widened by 2 screen points. `canvas::pressing` asks it after the
  vertex handles and the extension grips and before the body, because the
  text lies inside the body.
- `slid(kind, dx, dy)` is the rule: the page-space delta is projected onto
  the dimension's axis (`DimensionKind::axis_frame`) and added to
  `text_along`; `offset` is unchanged. Only `DimensionKind::Linear` slides.
- `drag(Frame, actions)`: mid-drag it returns the engine's bake of the slid
  dimension; on `Phase::Complete` it traces
  `dimension-label id= offset= text_along=` and pushes
  `DimensionAction::Place`, which commits `EditSession::place_dimension` as
  one undo entry.

## Why along only

A linear ce dimension's text sits on its dimension line, at
`text_along` from the line's centre. The only way to move the text off the
line is to move the line (`offset`), which is the body drag
(`canvas::dimdrag`). So a text drag keeps the across-line part of the pointer
motion out; that is what makes it plain the line is staying put.

Perimeter and circular ce dimensions have no separate text drag: a
perimeter's placement already moves only its label, and a circular one's text
and leader move together by definition.
