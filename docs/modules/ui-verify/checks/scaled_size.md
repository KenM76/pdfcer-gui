# `ui-verify/checks/scaled_size`

`a_scaled_width_sets_the_drawn_size`, for O456.

## What it drives

`fixtures/scaled-box.pdf`: a 40 x 20 pt box and one ce dimension whose
default group reads 5 mm/pt. Edit mode with Properties open. It clicks the
box, types `500 mm` into the scaled Width row, presses Enter, then Apply.

## Verdict

PASS when `geometry-scaled-typed` reads `pts=100.000`, the following
`resize-scale` has `sx` within 0.005 of 2.5, and `transform-objects-applied`
follows. FAIL when the rows never draw, the length is refused or read as
another number, Apply commits another factor, or nothing reaches the engine.

## Falsified

With the read length planted five times too long (the scale applied twice),
the check FAILs at the `pts` assertion. With the source restored it PASSes
with `pts=100.000` and `resize-scale sx=2.5000`.

## What it does not prove

The Height row, the annotation arm, and a page with `/UserUnit`, which share
the same code path through `user_units_from_measurement_on`.
