# `ui-verify/checks/dimension_text_slide`

`a_dimension_text_slides_alone` checks that pressing on a selected linear ce
dimension's text and dragging slides the text along the dimension line, with
the line kept where it is, while the button is held and after the release.

## What it drives

1. Review → Measure → Linear, and place a dimension on blank paper below the
   frame of `blank-overhang.pdf`.
2. Zoom ×3 at the text, arm Select, and click the dimension line (not the
   text) to select it. Re-read the page mapping after this: arming Select can
   change the ribbon's height and move the canvas.
3. Read `canvas.dimension-label`, press at its centre and drag 90 px along
   the line and 40 px across it, capturing the window with the button held.
4. Release.

## What it asserts

- **The preview slid along the line only.** The last `dim-preview baked=1`
  line's `label=` quad, mapped to the window, is within 2 px of the text's
  starting height and at least 45 px along. The 40 px across is the part that
  must be ignored.
- **The preview is on the glass.** The previewed text box carries at least
  1 % more ink with the button held than before the drag.
- **The release slid the text, not the dimension.** There is no
  `dimension-place` line (the body drag's), there is a `dimension-label` line
  and a `place-dimension` funnel line, and the re-published text region sits
  where the preview showed it, on the same line.

Falsified by planting an `offset` change in `canvas::dimlabel::slid`: the
check fails with the text 40 px off the line.
