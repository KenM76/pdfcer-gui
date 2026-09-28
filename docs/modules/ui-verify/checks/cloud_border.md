# `ui-verify/checks/cloud_border`

A square already on the page is made cloudy from the Properties panel.

## The drive

1. Draw a square on blank paper with the Rectangle tool (Review mode), then
   zoom in 3x on it: at fit-page on an A1 sheet a scallop is a pixel deep.
2. Measure the ink in a strip just outside the edge at the shape's larger y:
   2 to 9 pt out, across the middle 60 % of the edge. A straight 2 pt line
   reaches 1 pt out, so the strip is empty (at most 4 ink pixels).
3. Select the square, open the Properties tab, and check that
   `properties.markup.cloud` is drawn, the panel's `markup-cloud-row` trace
   reads `intensity=none`, and `properties.markup.cloud.intensity` is absent.
4. Click the checkbox. A `set-markup-style` line must follow, the panel must
   read an intensity back from the file, and the intensity field must appear.
5. Click blank paper above the square to deselect it, so selection handles
   are out of the strip, and measure again:
   the scallops must put at least 12 ink pixels in it.

## Why each assertion

The trace proves the write; the read-back proves the panel shows what the
file holds rather than what was clicked; the pixels prove the page draws the
cloud. A check stopping at the trace would pass a restyle that the renderer
never showed.

A straight edge with more than 4 ink pixels in the strip SKIPs: the strip is
not beside the edge at that zoom and cannot tell a cloud from a line.
