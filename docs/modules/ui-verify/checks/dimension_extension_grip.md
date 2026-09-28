# `ui-verify/checks/dimension_extension_grip`

`an_extension_line_grip_shortens_its_line` checks that dragging a linear ce
dimension's extension-line grip towards the dimension line shortens that
line. The grip follows the pointer while the button is held, and the release
commits the change.

## What it drives

1. Review → Measure → Linear. Pick two points and an offset on blank paper
   below the frame of `blank-overhang.pdf`, then add the dimension.
2. Zoom ×3 with ctrl+scroll at the midpoint of the A-side extension line, and
   re-read the page mapping. At fit-page (about 45 %) an extension line is
   thinner than a pixel, and the pixel measure cannot see it.
3. Arm the select tool and click the dimension line to select it.
4. Read the `canvas.dimension-extension.0` region from the trace. Drag it
   half the way to the dimension line with `drag_observed`, capturing the
   window and reading the trace with the button held.
5. Release, press Escape to clear the selection, and capture again.

## What it asserts

- **The grip followed the pointer.** With the button held, the published
  grip rect has moved at least half the drag's length along it. The previous
  build, which drew the grip at its committed position, fails here.
- **The drag was previewed.** There is no `dim-preview baked=0` line, and
  there is at least one `dim-preview baked=1` line.
- **The release committed.** A `dimension-extension-gap` line and a
  `set-dimension-extension-gap` funnel line both follow.
- **The line is shorter on the glass.** The check samples a strip
  `HALF_WIDTH` px either side of the stretch between the grip and the drop,
  shortened by `END_CLEAR` px at each end. The strip must be at least 5 %
  ink before the drag and under 1 % after it.

The picks are on blank paper because the fixture's title block, and the
selection outline, both put ink in the strip and hide the line.
