# `ui-verify/checks/dimension_circular_label_drag`

`a_radius_label_drag_swings_the_leader_to_the_drop` checks that dragging a
radius ce dimension's label previews the moved dimension while the button is
held, and that the release stores a leader facing the drop with the text
outside the rim.

## What it drives

1. Review → Measure → Radius/diameter. Click three rim points of a circle on
   the blank right half of `blank-overhang.pdf`, at 90°, 210° and 330°, then
   click Finish. No pick sits at 0°, where a new circular ce dimension draws
   its leader.
2. Arm the select tool and click the leader between the label and the rim.
3. Capture the window, then drag from the leader to a point that puts the
   label anchor at (2R, 2R) from the centre, capturing with the button held.
4. Release, press Escape to clear the selection, and capture again.

## What it asserts

- **The new dimension's label starts where a 0° leader puts it.** The box
  half way along the 0° radius carries ink before the drag.
- **The drag was previewed.** There is no `dim-preview baked=0` line, and
  there is at least one `dim-preview baked=1` line.
- **The preview is on the glass.** The label quad the last `baked=1` line
  names is blank before the drag and inked with the button held. A
  destination already inked before is SKIPPED, never passed.
- **The leader faces the drop.** The release's `dimension-place` line carries
  `text_along` (the leader angle, degrees) within 5° of 45°, and a positive
  `offset` (the text distance past the rim).
- **The label moved rather than copied.** After release the destination box
  is inked, and the starting box is blank.

The destination is 2R out on both axes because the selection outline hugs
the circle, and a nearer drop puts its edge inside the label box.

The previous release, which could not drag a circular ce dimension's label,
fails at the preview assertion: it draws nothing with the button held and
writes no `dimension-place` line.
