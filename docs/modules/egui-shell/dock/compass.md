# `egui-shell/dock/compass`

## Item notes

### `fn geometry_for`

Deliberately not [`super::super::plan`]: this is a plausible geometry,
not the real one, and the compass must not care which. What it must
share with the real one is the relationship the resolution depends on —
the strip along the top of the compartment — and that is asserted
directly by `over_the_tab_strip_is_a_boundary_between_tabs`.
