# `egui-shell/menu/ctx`

## Item notes

### `struct MenuCustomItem`

The menu reserves a row in its vertical flow, hands over `kind` and
`payload`, and gets out of the way — the same contract the ribbon's
band offers, and for the same reason: the alternative is an item
vocabulary that grows a variant per widget an application happens to
want, which is the road by which a reusable shell stops being reusable.

### `type MenuCustomRenderer`

Returns a token if the operator invoked something, so an
application-drawn control reports through the same channel as a
command — see [`super::render`]'s header on the seam.

### `fn id`

Derived rather than auto-generated for the reason
[`crate::ribbon::ctx`] gives: `egui` keeps hover and focus state per
id, and an id that shifts when a row above it is filtered out by
the no-placeholders rule produces a control that loses focus when
the *selection* changes — which reads as a focus bug rather than as
an id bug and is very hard to attribute.
