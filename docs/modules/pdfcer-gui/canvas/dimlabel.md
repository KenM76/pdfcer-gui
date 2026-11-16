# `canvas::dimlabel`

Where a selected linear ce dimension's value text is, so a press on it can be
told from a press elsewhere.

## Contract

- `screen_quad(ctx, doc, map, selection)` gives the text's four corners in
  screen space. They come from `EditSession::dimension_preview` of the
  dimension as committed (`appearance.label_quad`), cached in egui temp data
  keyed on the dimension's id and kind, so a selected dimension is baked once,
  not every frame. `canvas::painting` publishes their bounds as
  `canvas.dimension-label`. Nothing is drawn: the text is the affordance.
- `label_at(ctx, doc, map, selection, screen)` says whether a press landed in
  that quad, widened by 2 screen points. `canvas::pressing` asks it after the
  vertex handles and the extension grips.
- `canvas_quad_of(doc, page, annot)` gives the same quad in canvas space for
  any ce dimension annotation, for click selection.

## A press on the text moves the dimension

`gesture::meaning` maps a press on the text to the same drag as a press on the
line (`canvas::dimdrag`): the text follows the pointer along the line and
across it, the line moving with it, previewed through `canvas::dimpreview`.
The text is asked about separately from the body because, slid far along, it
can sit outside the line's grab box.

