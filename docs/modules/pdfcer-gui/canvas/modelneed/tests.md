# `pdfcer-gui/canvas/modelneed/tests`

## Item notes

### `fn at_part_rung`

Built through `SelectionState::click` with `double`, exactly as
`canvas::deleting::tests` builds its own, so the rung is reached by the
ladder's real rule rather than by writing the field.

### `fn a_delete_at_the_object_rung_pays_for_nothing`

The other half of the tripwire above, and it is what stops a future
session "fixing" the first one by asking unconditionally. On the operator's
benchmark drawing a decomposition is **531 ms / 129,758 objects**, and
`canvas::deleting`'s Object arm never reads it.

### `fn merely_standing_at_a_deeper_rung_asks_for_nothing`

This is the choice the measurement decided. *"Ask whenever a deeper rung
is selected"* would be correct and would pay 531 ms, after each content
edit, on frames nothing reads the model — while the operator merely holds
the selection. See the module header.

### `fn a_zoom_marquee_decomposes_nothing`

The zoom half is the concrete payoff for carrying the intent on the
outcome: a region zoom over a 129,758-object drawing costs one scroll
offset, not 531 ms.
