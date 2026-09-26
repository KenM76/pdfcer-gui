# `pdfcer-gui/app/dispatch/zoom`

## Item notes

### `fn handles`

`pub(crate)` rather than `pub(super)`: `shell::commands::reach`'s
`guard_claiming` calls it, because the reachability checker must be able to
EVALUATE every guard arm it finds — a guard it cannot evaluate is a place
commands could hide from the check that exists to find them.

A separate predicate rather than a `match` returning `bool`, for the reason
[`super::pages::handles`] gives: the caller is a guard on a match arm, and
the guard and the body must not be able to disagree about what is claimed.
