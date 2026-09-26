# `egui-shell/theme/overlays`

## Item notes

### `fn merged_roles_are_refused_and_both_are_named`

The message is the deliverable: "two roles collided" sends the
reader back to work out which two, on a preset with a dozen of
them.

### `fn an_undefined_role_fails_rather_than_passing_vacuously`

This is the difference between a gate and a decoration. If unknown
roles were treated as trivially distinct, a check whose role names
were both misspelled — or whose roles were renamed in the palette
and not in the test — would go green forever while measuring
nothing. The salvage source's whole family of "green is not
evidence" lessons is this same failure in other clothes.
