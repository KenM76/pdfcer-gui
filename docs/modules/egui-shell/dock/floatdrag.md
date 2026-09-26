# `egui-shell/dock/floatdrag`

## Item notes

### `fn draw`

Runs from [`super::Dock::show`] after [`super::overlay::draw`] and
[`super::tear::draw`], so a frame on which the pointer's own gesture
published has already published.
