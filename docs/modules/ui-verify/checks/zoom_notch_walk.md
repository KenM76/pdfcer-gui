# `ui-verify/checks/zoom_notch_walk`

`zooming_click_by_click_keeps_the_detail_under_the_cursor` — O231.

Ctrl+wheel one notch at a time with the pointer held still over a detail,
all the way in and all the way back out, recording at every notch which page
point is under the pointer and photographing the patch around it
twice: shortly after the notch, and once rendering has settled.

# Why the existing zoom checks cannot see O231

`the_page_still_renders_at_every_decade_of_zoom` photographs at decades and
**re-aims the pointer** at the target before each photograph, so a view
that jumped is silently followed; and it asks whether the whole canvas is
blank, not the part under the pointer. `zooming_does_not_throw_away_where_
the_operator_panned` measures the position eight notches at a time and
never looks at pixels. A transition between two samples is invisible to
both — this walks every notch.

# What fails it

- **Drift:** the page point under the pointer, computed in `f64` from the
  `canvas-pos` line, moves by more than [`DRIFT_PX`] screen points across
  one notch. Zoom-to-cursor holds that point fixed by definition. Measured
  notch to notch rather than against `--doc-point`, because the harness's
  initial aim carries its own error and a constant page offset times a
  growing zoom reads as a runaway it is not.
- **An order for the wrong place:** on any `scroll`-tier frame, the region
  the shell wants rendered (`canvas-pos want=`) does not contain what the
  viewport shows at that frame's zoom and pan. A region ordered from the
  previous frame's offset against a new zoom names a place `offset / zoom`
  points away; if the debounce lets it through, the raster that arrives is
  painted at its own region and the view under the pointer is blank.
- **A white-out that recovers on the way out:** the settled patch under the
  pointer is uniformly paper at a zoom where, walking back out, the settled patch at
  the same zoom was not. The same view at the same zoom must paint the same
  thing whichever direction it was reached from; a patch that is blank only
  on the way in is the canvas losing a raster, not the drawing being empty.

Every capture is kept as an artifact, named by direction, notch and zoom.

## Item notes

### `const DRIFT_PX`

Above the floor both walks measure on SW41177 with the pointer still: up to
about 5 points per step, from the scroll area rounding the drawn page to
whole pixels on every animation frame plus the harness reading the
position from the trace at both ends. Carrying the anchor fraction across
frames did not move that figure. A region ordered for the wrong place, the
defect this guards, is off by whole screens.

### `fn region_misses_view`

The viewport in page points is rebuilt from `canvas-pos at=` (the pan in
logical points from the page's top-left), the zoom of the `canvas` line
before it, and the canvas size. The page's y axis points up, so the top of
the view is `ext.y - at.y / zoom`. A tolerance of 1e-3 of the view's width
absorbs the region's snapping and the trace's rounding. Off-page content
widens the order past the sheet, never narrows it, so clipping the view to
the sheet cannot hide a miss.

### `fn rig`

# Errors
Missing binary, PDF, target or page size; input disabled (a SKIP); launch
or trace failures.
