# `editmodel::kind` — which of the two text verbs is armed

## Item notes

### `enum TextEditKind`

One value carried on the tool, for `MarkupKind`'s
argument: the operator is doing exactly one of these, so a type that could
express both would have illegal states to prevent by discipline.

### `fn command_id`

The single binding between an id and a kind, in the shape
`shell::commands::markup_for_command` has — read from both directions by
`crate::shell::commands::text_edit_for_command` and by the label the
status bar shows, so the two cannot drift.
