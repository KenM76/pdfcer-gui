# `egui-shell/tabshape`

One look for every row of tabs, so a tab never reads as a button
(`OPERATOR_REQUESTS.md` O268).

## Item notes

### `fn body`

The selected tab is a top-rounded body in `fill` (the colour of the surface
below it), outlined on three sides, with a 2 pt accent rule along its top and
its bottom edge painted over the bar's baseline so it opens into that surface.
Hovered tabs get the same fill without the outline; resting tabs paint nothing
and sit flat on the bar. Painted into a `Shape::Noop` slot reserved before the
widget, because hover is only known after it.

Callers: `dock::tabs` (panel tabs), `ribbon::tabs` (menu tabs, fill
`surface`), `tabstrip` (document tabs).

### `fn underline`

The secondary style, for a row of sub-pages inside a panel (Align / Grid /
Circular): frameless text with a 2 pt rule under it, accent when selected and
outline on hover.

### R84

`RichText::strong()` in egui 0.35 changes colour only. The selected state is
carried by shape (the outline and the rule), with the label colour a third cue.
