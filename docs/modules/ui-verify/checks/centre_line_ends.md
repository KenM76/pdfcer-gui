# `centre_line_ends` — dragging a centre line's end

`a_centre_lines_end_can_be_dragged` draws a centre line on `two-lines.pdf`
exactly as `a_centre_line_is_drawn_between_two_lines` does, presses Escape,
and clicks the line at (200, 140) to select it.

1. The click must trace `annot-select kind=CeDimension`.
2. Exactly two grips must be published, `canvas.dimension-vertex.0` and `.1`:
   a centre line has two ends and no corners.
3. Grip 1 is dragged 40 window points up. The shell must trace
   `dimension-vertex index=1` with a positive `dy` — page y is up, so a wrong
   axis or a wrong grip reads differently — and the engine's
   `move-dimension-vertex` must follow.

The scripted pointer drives it, so it runs under `--no-input`.
