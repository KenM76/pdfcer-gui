# `app::dispatch` — one command in, one effect out

The routing table. Every operator gesture that names a command — a
ribbon click, a quick-access button, a context-menu item, a keyboard
chord, a custom item reporting its own token — arrives here, and this is
the **one** place that decides what the application does about it.

## Why one choke point rather than a closure per command

`egui_shell` stores an opaque `HandlerToken` and hands it back; it never
interprets it. That is what keeps the shell reusable — a registry of
closures would force it to name pdfcer's state type. The consequence on
this side is a single `match`, and the consequence of *that* is the
property worth protecting: **a confirmation gate, an undo entry or a
trace has exactly one place to go.** Scatter dispatch across as many
sites as there are commands and each of those becomes something somebody
has to remember at every site.

## Why this is separate from `app::mod`

Split out at Phase 3, when `app/mod.rs` reached 1,638 lines against the
1,500-line gate (R2). The seam is a real one rather than arithmetic:
`mod.rs` composes a frame — panels in order, canvas, dialogs, then apply
— while this file answers *what does this verb do*. The two change for
different reasons and are read at different times.

The gate's own rationale is the argument for splitting here rather than
anywhere that merely counts: the GUI this project replaces reached 25,005
lines in one `main.rs`, and two of the defects in `DEFECTS.md` are pairs
of lines thousands of lines apart that no reviewer could have been
expected to see together.

## The arms route; they do not compute

Almost every arm is one line: push an [`Action`], or call the one
function in the module that owns the rule. Zoom anchoring lives in
`crate::canvas::zoom`, the tool in `crate::canvas::tool`, the print
dialog in `crate::dialogs`. The moment an arm starts working out *how* to
do something, that rule exists in two places and only one of them will be
the one that gets fixed.

The few exceptions are marked where they occur, and each is a routing
decision rather than a rule: `file.recent` chooses between a parked
operand and the newest reachable entry; the panel commands map an id to a
panel.
