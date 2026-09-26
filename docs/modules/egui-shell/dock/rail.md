# `egui-shell/dock/rail`

## Item notes

### `fn visible_ids`

Filtering happens **before** the ladder runs, exactly as
[`crate::ribbon::trailing`] filters before it measures, and for the same
reason: a hidden item that was counted would make the rail fold a group to
make room for a control nobody can see.

This is where **mode gating** lands. An entry marked
`visible_when("mode.edit_content")` is, in a mode that does not set that
condition, not in this list, not measured and not folded — it is simply
absent, which is R9: *an unavailable capability renders nothing.*

### `fn read_mode_drops_the_points_tool_entirely`

R9: an unavailable capability renders nothing. Folding it would put it
behind the chevron, where the operator could reach a control the
mode's own dispatch refuses.

### `fn the_width_is_constant_at_every_rung_and_every_budget`

The R128 argument in a test: nothing about the content, the rung, the
number of folded entries or the length of a caption can move it. A
build that sized the rail from its widest word would fail here.
