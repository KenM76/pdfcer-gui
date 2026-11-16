# `ui-verify/checks/group_numbers_at_creation`

`a_new_groups_numbers_are_set_when_it_is_made`, on `fixtures/dimension-scaled.pdf`
(400×300 page, default group calibrated at 5 mm/pt), through `dimdrive::run`.

Review mode, linear tool armed, Manage groups open. The *Add a group* fold
makes two ce-dimension groups, both feet and inches with scale *Same as
Default*:

1. `Reduced` — the unit's own numbers.
2. `Fixed` — the new-group Numbers editor's lowest-terms box unticked
   (`dimension-groups.new_numbers.precision.reduce`), so eighths are kept.

Then one linear ce dimension into each, A (60,60) → B (260,210), placed at
(160,30), and `Fixed` is switched to inches and back from its own unit combo.

## Oracle

From the geometry alone: 250 pt × 5 mm/pt = 1250 mm = 49.2126 in = 4 ft 1 in
and 1.70 eighths, which rounds to 2/8 and reduces to 1/4. The check computes
this from its constants and refuses to run if the picks give a fraction that
reads the same both ways (an odd numerator), because then a reduced and a
kept reading are indistinguishable.

Undo: a freshly opened document has an empty stack, so the
`dimension-group-added ... undo_depth=` line must say 1 after the first group
and 2 after the second. An add followed by a separate format or scale edit
that failed to fold would read 2 and 3.

## PASS

| reading | must contain | must not contain |
|---|---|---|
| `Reduced`'s ce dimension | `4'-1 1/4` | `2/8` |
| `Fixed`'s ce dimension | `4'-1 2/8` | `1/4` |
| `Fixed` switched to inches | `49 2/8 in` | `1/4` |
| `Fixed` back to feet and inches | `4'-1 2/8` | `1/4` |

Readings are matched on the raw `dimension-member-shown` line, because a
feet-and-inches text ends in `"` inside the quoted `text=` field.

## Trace lines read

- `dimension-group-added group= unit= numbers=<decimal:N|reduced:D|kept:D> undo_depth=`
  from `app::actions::dimensions::add_group`.
- `dimension-added dim= group=` and `dimension-member-shown group= dim= text=`.
