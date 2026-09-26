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

### `fn custom`

Keyed by `kind` rather than by position, because position is exactly
what changes when a command above it is filtered out by the
no-placeholders rule.

### `fn icon`

Published only when the application's icon painter was actually called
for that row. See this module's header: the absence of this name is the
assertion, and it is the only signal a driven check has that a menu
surface draws glyphs at all.
