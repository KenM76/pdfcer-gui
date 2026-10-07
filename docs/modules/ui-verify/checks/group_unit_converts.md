# `ui-verify/checks/group_unit_converts`

`changing_a_groups_unit_keeps_its_real_lengths`, for O288 item 3.

## What it drives

`fixtures/dimension-scaled.pdf` (one ce dimension reading `1000.00 mm`,
group scale 5 mm/pt), Review mode with Manage groups open. It presses the
group row, the unit combo, then the `ft` option.

## Verdict

PASS when a `dimension-member-shown` line reads a value within 0.01 of
3.28 ending in ` ft`. FAIL when none follows, or the value is anything else.

## Falsified

With the panel sending `scale: group.scale` (the number carried over), the
check FAILs with `text="1000.00 ft"`, Ken's defect. With the source restored
it PASSes with `text="3.28 ft"`.

## What it does not prove

A per-dimension unit override: the engine's `resolve_style` ignores the
group calibration for one until the pin carries the G146 fix.
