# `egui-shell/menu/report`

## Item notes

### `fn the_reported_names_are_a_stability_contract`

These strings are consumed by a harness in another tool, possibly
in another repository, by literal comparison. A rename is a
breaking change with no compiler to catch it: the harness keeps
building, its assertions simply stop matching anything, and a test
that matches nothing passes.

### `fn a_body_can_never_be_mistaken_for_a_row`

The reason the body is `menu.body.` rather than bare `menu.`: a
context id is an arbitrary application string, and one called
`item` would otherwise produce a body name that a harness
filtering for rows would match.
