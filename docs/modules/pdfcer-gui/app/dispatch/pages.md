# `pdfcer-gui/app/dispatch/pages`

## Item notes

### `fn handles`

`pub(crate)` rather than `pub(super)`: `shell::commands::reach`'s
`guard_claiming` calls it, because the reachability checker must be able to
EVALUATE every guard arm it finds — a guard it cannot evaluate is a place
commands could hide from the check that exists to find them.

A separate predicate rather than a `match` that returns `bool`, because the
caller is a guard on a match arm and the two must not be able to disagree
about the set: a command listed here and missing below would fall into this
arm and silently do nothing, which is the *"visible control, silently
inert"* failure this crate keeps finding.

### `fn dispatch`

`id` is guaranteed to be one [`handles`] claims — the caller's arm is
guarded on it — so the fall-through below is unreachable and says so rather
than guessing.

**The receiver is `&mut PdfcerApp` for `pages.resize` alone.** Every other
arm here builds an `Action` and pushes it; that one opens a window, which
lives on `PdfcerApp::dialogs`. The alternative — putting the arm in
[`super`] beside `pages.merge_into` and `pages.insert_from_file` — would
scatter the Pages tab across two files to save a `mut`. Widening the
receiver costs the call site nothing (it is already inside a `&mut self`
method and reborrows) and keeps every Pages arm in the file named after the
Pages tab.
