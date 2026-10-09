# `app::actions::restack` — page objects moved in paint order

One call to `EditSession::restack_objects` through
`apply::vector_edit_on_page`, labelled `restack-objects`, one undo step
(`CommandKind::RestackObjects`). Reached from Edit ▸ Arrange's four commands,
the canvas object menu and the Arrange chords, which `app::dispatch::arrange`
routes here when page objects are selected in a mode that edits content.

## Contract

- **In:** a page, the selected page-object indices, an `ArrangeTo`.
- **Out:** the objects' new indices are reselected (`SelectionState::marquee`),
  because a restack renumbers every object it passes — the old indices name
  other objects afterwards. Undo and redo of a restack clear the selection
  for the same reason (`actions::history`).
- **Trace:** `restack-applied page= to= asked= moved= limited= indices=`, lists
  comma-joined, `-` for none.

## Disclosure

The engine's `RestackOutcome` says what it could not do, and the status line
says it (`text::arrange`):

- `moved` empty → the objects are already at the front, back, or next to
  nothing they overlap in that direction.
- `RestackLimitReason::Scope` → an object stopped at the nearest position its
  scope allows (a different clip, `/OC` layer or structure element lies
  beyond it).
- `RestackLimitReason::Entangled` → an object's own bytes set a clip, marked
  content or an unbalanced `q`/`Q` that later objects depend on; it was not
  moved.

A part of a placed drawing has no restack verb; the commands are not offered
for it.
