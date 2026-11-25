# `ui-verify/checks/dimension_like`

`a_group_made_like_another_copies_its_set_up`, on
`fixtures/dimension-scaled.pdf` (default ce-dimension group calibrated at
5 mm/pt), through `dimdrive::run` with `mode.review,measure.manage_groups`.

1. Right-click the default group's row (`dimension-groups.row.0`) ▸ **New group
   like this one**.
2. Select the same row, open the *Add a group* fold, press **Add group like
   ‹name›** (`dimension-groups.add-like`).

## PASS

Each press is followed by a `dimension-group-like` line with `template=0`,
`same_scale=1 same_format=1 same_standard=1 same_style=1`, and
`undo_depth=1` after the first and `2` after the second — one entry each on a
freshly opened document. The named group must then be drawn as a row, and the
two routes must name different groups.

## Why the default group is the template

Its scale is a calibration, not the *Same as Default* a plain new group gets,
so a route that fell back to `add_dimension_group` reads `same_scale=0`. The
`same_*` fields compare the engine's model of the new group against the
template's after the call, so they read what was made, not what was asked.
