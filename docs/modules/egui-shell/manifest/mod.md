# `egui-shell/manifest/mod`

## Item notes

### `const TIDY_MAX`

Generous, because these lines are nested four levels deep and the whole
point is that a one-field item reads as one thing. Past this, three short
lines really are easier to read than one long one.

### `fn a_manifest_round_trips_through_ron`

The manifest's whole value proposition is that it is a file: an
operator edits it, an application ships one, a workspace is saved
as one, and `SHELL_FRAMEWORK.md` §6 lists "inspectable, diffable,
testable without a GUI, and serializable" as what the design buys.
Every one of those claims fails if the round trip is lossy.

Both forms are checked. The compact form is what a save writes;
the pretty form is what an operator opens, and a pretty printer
that emits something its own parser rejects would be discovered by
the operator rather than by CI.

### `fn an_unstated_field_stays_unstated_through_a_round_trip`

This is the property the whole layered design rests on: `None`
means "do not mention this" and must not come back as
`Some(empty)`. If a layer's omitted `groups` round-tripped into
`Some(vec![])`, saving and reloading an operator's customization
would silently empty every tab it mentioned.
