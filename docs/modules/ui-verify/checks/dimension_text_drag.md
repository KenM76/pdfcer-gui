# `ui-verify/checks/dimension_text_drag`

`a_dimension_text_drag_moves_the_dimension` checks that pressing on a selected
linear ce dimension's text and dragging moves the dimension as a press on its
line does: the text follows the pointer along the line and across it, with a
live preview while the button is held.

## What it drives

Off the desktop, through the scripted pointer, on `fixtures/dimension-scaled.pdf`:

1. Click the ce dimension's line to select it and read `canvas.dimension-label`.
2. Press at the text's centre and drag 50 points along the line and 36 across
   it, over twelve frames, and release.

## What it asserts

- **A live preview.** At least one `dim-preview baked=1` line during the drag.
- **The release placed the dimension.** A `dimension-place` line and a
  `place-dimension` funnel line follow the press.
- **The text followed the pointer both ways.** The re-published text region
  moved at least half the drag along the line and at least half across it. The
  across part is what a slide along the line cannot produce.

That the baked preview reaches the glass is
`a_dimension_drag_previews_what_it_places`'s subject: the two drags share one
painter.
