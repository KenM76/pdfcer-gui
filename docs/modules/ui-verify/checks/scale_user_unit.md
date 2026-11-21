# `ui-verify/checks/scale_user_unit`

`a_calibration_on_a_user_unit_page_reads_the_stated_length`.

## What it drives

`fixtures/user-unit.pdf`: a 306 × 396 box at `/UserUnit 2`, framed by
`18 18 270 360 re`. Review mode with the linear tool armed. It draws a ce
dimension along the frame's bottom edge (270 box units, 540 pt) and places
it; opens Measure ▸ Set scale; presses *Measure it on the drawing*
(`scale.calibrate`); picks the same two corners; types `5 m` into the
real-length field (`scale.real_length`) and presses Accept
(`dialog:set-scale.accept`).

## Verdict

Accept sets the default group's scale, and `SetGroupScale` writes one
`dimension-member-shown` line per member. The newest after Accept must read
5 m within 0.05 m: the edge the operator called 5 m reads 5 m. FAIL when it
reads anything else, or not in metres. The defect reads 10 m: the reference
line's 270 box units taken as 270 pt while the ce dimension measures 540 pt.

## Falsified

With `ScaleEntryFields::entry` planted to build
`ScaleEntry::RealLength { drawn_pdf_length: line.measured, .. }` (box units,
`/UserUnit` ignored), the check FAILs reading `10.00 m`. With the source
restored it PASSes.

## What it does not prove

A page at `/UserUnit 1` (unchanged arithmetic, covered by
`measure_calibrates_by_picking_two_points`) or a line on a page other than
the first; `Reference` is built from the page the pick was made on, which
the unit test `a_reference_on_a_user_unit_page_is_measured_in_points`
exercises for the conversion only.
