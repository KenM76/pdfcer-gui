# `ui-verify/checks/dimension_tool_options`

Three checks for O288 items 4–6, on `fixtures/dimension-scaled.pdf` (400×300
page, default group calibrated at 5 mm/pt). Each draws one linear ce
dimension: A (60,60) → B (260,210), placed at (160,30). Δx 200 pt, Δy 150 pt,
aligned length 250 pt.

The oracle is the pair of trace lines the commit emits:
`dimension-added dim= annot= group= constraint=`, and the
`dimension-member-shown` line with the same `dim`, whose `text` is what the
engine renders on the page.

## `a_horizontal_dimension_measures_only_the_run` (item 6)

Review mode, linear tool armed. Presses *Horizontal* in Tool options, draws.

PASS: `constraint=horizontal` and the text reads 1000 mm ±1 (Δx × 5).
The aligned reading would be 1250 mm.

Falsified: `set_linear_constraint` not storing the choice → FAIL,
`constraint=aligned` reading `1250.08 mm`.

## `a_dimension_joins_the_group_made_in_tool_options` (item 4)

Review mode, linear tool armed. Opens the group combo, picks *New group…*,
types `Detail`, presses *Create*, draws.

PASS: a `dimension-authoring-group via=created id=N` line follows Create, the
ce dimension's `group` is N (not 0), and it reads 1250 mm ±1 — the new group
started from Default's scale.

Falsified: `add_group` not queueing the new group → FAIL at the
`via=created` assertion. The group-mismatch and wrong-reading arms have not
been seen to fire.

## `a_new_group_copies_a_groups_scale_in_its_own_unit` (item 5)

Review mode, linear tool armed, then Manage groups. Unfolds *Add*, names the
group `Feet`, picks unit `ft`, scale *Same as Default*, presses *Add*, then
the new group's *draw into* radio, draws.

PASS: `dimension-group-scale-copied folded=1` (one undo step), the ce
dimension joins that group, and it reads 4.10 ft ±0.01
(250 pt × 5 mm / 304.8).

Falsified: the scale copied as the bare number, unconverted → FAIL,
`1250.08 ft` — millimetres read as feet.

## What they do not prove

Vertical (the same code path as Horizontal with the other axis), and the
non-linear tools' group row, which share `tooldim::group_row` but are not
driven.
