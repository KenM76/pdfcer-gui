# `pdfcer-gui/canvas/textsel/clipboard`

## Item notes

### `fn an_idle_frame_asks_for_no_text_chord`

It is asserted at the level of the **predicate** rather than by counting
extractions, because that is where the property lives: the caller's `if
let` cannot fetch anything when this answers `None`, whatever else it
does.

### `fn a_focused_text_field_keeps_the_text_chords`

These are the two chords an operator presses *inside* the Find field. A
canvas that took them would select and copy the page instead of the text
being typed, which is the same failure D1 produced with Delete and would
be more surprising, because the operator can see the field they are in.

Built against a **real** `TextEdit`, for the reason
`canvas::keys::a_focused_text_field_keeps_delete_for_itself` gives:
`text_edit_focused()` resolves the focused id and looks for a
`TextEditState` under it, so a hand-requested focus on a bare id would
pass vacuously.
