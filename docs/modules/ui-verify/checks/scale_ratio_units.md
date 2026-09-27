# `ui-verify/checks/scale_ratio_units`

`scale_ratio_reads_as_the_drawing_states_it` — the oracle for **O242**: the
Set-scale ratio reads as a title block states it, a unit beside each number,
and a unit typed into a ratio box is converted to that box's unit.

## What it asserts

1. **Layout.** The four regions `scale.ratio_paper`, `scale.basis`,
   `scale.ratio_real`, `scale.real_unit` are declared, overlap vertically with
   the first (one line), and sit left to right without overlap.
2. **Typed unit.** It reads `real_unit=` from the last `scale-seeded` line,
   types twenty feet into `scale.ratio_real` in a unit the box is *not* in
   (`20 ft`, or `240 in` when the box is in feet), presses Enter, then Accept.
   The `scale-commit` line's `ratio=` world side must equal 240 inches in
   `real_unit`, from the check's own inch table (25.4 mm, 12 in, 36 in,
   63,360 in), not the engine's, and `real_unit` must be unchanged.

Choosing a unit the box is not in is what makes step 2 able to fail: typing
the box's own unit passes on a build that ignores the suffix.

## Why driven

`ScaleEntryFields::entry` converting `real_unit` has a unit test; the
preprocessor parsing `20 ft` has its own. Neither sees whether the dialog's
box is built with the `Length` kind of the right unit, which is the link
the operator touches.

## Control

The published build fails at step 1: it declares only `scale.group`,
`scale.current` and `scale.calibrate`. Step 2 is therefore not falsified by
the control independently; it is falsified by construction (a typed unit the
box is not in).

## Coordinates

The Set-scale window is its own OS viewport. Every click inside it goes
through `frame_of` for a region in that window; the main window's frame
lands the click on the dialog's title and the harness refuses it.
