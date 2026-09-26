# `egui-shell/dock/geometry`

## Item notes

### `fn hit`

Last match wins: compartments are recorded in draw order and do not overlap,
so the choice is immaterial for stacks — but a later entry is the one drawn
on top, which is the answer a pointer gesture wants if that ever changes.
