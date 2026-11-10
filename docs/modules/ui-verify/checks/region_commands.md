# `ui-verify/checks/region_commands`

Three checks on the marquee region's commands, each launched on
`blank-overhang.pdf` (drawing inside x 6–50 %, y 50–92.5 %) with
`PDFCER_DIAG_INVOKE=mode.edit,view.tool_snapshot`, because the region's
content commands are drawn only where content can be edited. They share the
rig and the box-line reader of [`snapshot_box`](snapshot_box.md), press the
buttons of the Tool Options Region block, and run under `--no-input`.

## `delete_in_region_removes_what_the_box_holds`

1. Lay a box over the drawing, (0.04, 0.48)–(0.52, 0.94). Its
   `region-selection objects=` must be above zero.
2. Press Delete in region. `region-delete-requested objects=` must equal that
   count.
3. Lay a box over the same drawing from a corner outside the first,
   (0.02, 0.46)–(0.54, 0.96), so the drag draws a new box rather than moving
   the laid one. It must select zero objects.

The third step is what makes the check fail on a delete that is asked for and
not carried out.

## `redact_outside_hatches_then_marks`

1. Lay a box on blank paper, (0.70, 0.10)–(0.90, 0.30); choose Outside region.
2. Shoot the window, press Redact outside…, require `redact-outside-opened`,
   shoot again.
3. Between the shots, the strip of paper left of the box (paper's left edge to
   the box's, the box's height) must change on at least 2 % of its pixels, and
   the box's interior, inset 16 px from the outline and grips, on none. The
   window is its own viewport, so it never covers the strip.
4. Press Mark outside in that viewport. `redact-mark-outside-marked marked=1`
   must follow on the one-page fixture.

## `crop_to_region_crops_to_the_box`

1. Lay a box over the drawing; read its corners from `snapshot-box`.
2. Press Crop to region…, then Crop in the window's viewport.
3. `page-crop-commit` must name one page, and its `llx lly urx ury` must sit
   within 1.5 pt of the box's: the window works in whole millimetres, so each
   edge may round by up to half a millimetre (1.42 pt).
