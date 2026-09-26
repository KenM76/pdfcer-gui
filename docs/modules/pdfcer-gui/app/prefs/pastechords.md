# `pdfcer-gui/app/prefs/pastechords`

## Item notes

### `fn every_order_binds_the_two_commands_to_two_different_chords`

The failure this forbids is a build where both commands end up on the
same chord — one silently unreachable from the keyboard, with the
ribbon still showing both and the shortcuts dialog still listing a key
that reaches the other one. Nothing on screen shows it; it is invisible
until an operator presses the chord and gets the wrong paste.

### `fn the_acrobat_order_is_exactly_the_pdfcer_order_reversed`

Asserted as a property rather than by restating the four literals,
because restating them is how a table and its test come to agree with
each other and disagree with the operator.

### `fn the_default_is_the_operators_ruling`

The preference exists to **offer** Acrobat's order, never to impose it:
a build that shipped with `AcrobatOrder` as the default would change what
`Ctrl+V` does on an upgrade for everyone who never opens settings.
