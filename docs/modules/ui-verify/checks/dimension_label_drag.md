# `ui-verify/checks/dimension_label_drag`

`a_dimension_drag_previews_what_it_places` — while a ce dimension's label is
dragged, the canvas shows the dimension the release will write, and the
release writes it there.

## What it drives

1. Review → Measure → Perimeter; trace a closed square (the same four
   corners as `dimension_corner_count`) and close it, which commits a ce
   dimension.
2. Put the pen down (`arm_select_from_ribbon`, `V` as fallback), then click
   the square's bottom **edge** to select it. The fill is not hit-testable —
   `dimdrag` hit-tests ink, not `/Rect`.
3. Capture the window.
4. Drag from the edge a quarter-page to the right with `drag_observed`,
   capturing with the button still held.
5. Let go and capture again.

## What it asserts

- **The engine baked the moved dimension.** A `dim-preview baked=0
  refused=…` line fails with the refusal quoted; no `dim-preview baked=1`
  fails as a preview that was never drawn.
- **The preview is on the glass.** The `label=` field of the last
  `dim-preview baked=1` line is the label's page-space quad. Its box in the
  capture must be blank paper before the drag and inked (≥ 1 % ink pixels)
  mid-drag. The painter's trace line is its account of its decision; the
  capture is the frame.
- **The preview is as dark as the dimension it previews.** The original
  label is still on the page mid-drag, one drag-width to the left. The
  preview's box must carry at least 85 % of the original's ink. A texture
  drawn off the physical pixel grid is resampled, and thin text comes out
  grey. Measured on the first driven build, before `dimpreview::paint`
  snapped its box, with a dark-pixel proxy: 0.52. The check has not been
  run against that build.
- **The release lands where the preview was.** A `dimension-place` line
  follows, and the same box is still inked after the release.

A perimeter is used rather than a linear dimension because its label follows
the pointer freely in page axes, so the destination is unambiguous; a linear
label is projected onto its axis.

## The original stays visible mid-drag

The page raster still holds the committed dimension, so the drag shows both
labels: the committed one where it was and the preview at the pointer. This
is the same convention as a content move (`chunk_ghost_pixels` asserts the
source is not marked in flight). Hiding the original would need the page
re-rendered without one annotation, which the engine does not offer.

## What would make it SKIP

No input allowed, no page size, no `canvas-viewport` region, a label box off
the canvas, or ink already in the destination box before the drag (a
fixture with content right of the page centre).
