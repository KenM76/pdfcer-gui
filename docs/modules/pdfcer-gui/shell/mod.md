# shell — pdfcer's ribbon, modes, QAT and keymap, as data

This module is pdfcer's half of the contract `SHELL_FRAMEWORK.md` §1
sets up:

> **The shell is data. Tabs, groups, commands, panels, layouts, modes
> and key bindings are a serializable document that the application
> *supplies* and the operator *edits* — not code that has to be
> recompiled to change.**

`egui-shell` owns the *types* and the *rules*: what a tab is, what a
group is, that a command may appear on exactly one tab, that a mode may
only name tabs that exist. It knows nothing about PDF and must never
learn. This module owns the *content*: which tabs pdfcer has, what is on
them, which verbs exist and what each one is called.

| Submodule | Holds |
|---|---|
| [`manifest`] | [`manifest::built_in`] — the whole ribbon as an `egui_shell::Shell` value: eight tabs, thirty-seven groups, three modes, the QAT and the keymap. Plus [`manifest::PLANNED`], the commands deliberately **absent**. |
| [`commands`] | [`commands::register`] — every command the manifest names, with its label, tooltip, icon key, enable predicate and opaque handler token. |
| [`menus`] | [`menus::built_in`] — the four context menus, carried on the same `Shell`, plus [`menus::MenuHost`], the one seam a right-click site uses. |
| [`ron`] | The same manifest as a `.ron` file, with a test that the two agree. |

## The third surface

[`menus`] is the context-menu half of `RIBBON_IA.md` §5.8's three
surfaces, and it is deliberately **not** a second vocabulary: a menu
item is the same `egui_shell::manifest::Item` a ribbon band holds,
resolved through the same registry into the same handler token. So P1 —
one command, one tab — is *not* extended over menus, and must not be:
§5.8 states that a menu carrying a tab's command again *"is not
duplication in the P1 sense — context menus are not tabs"*. The tests
below reflect that split, and [`menus`]' own tests carry the checks
`egui-shell` does not perform.

## The specification this implements

`RIBBON_IA.md` §5, tab by tab and group by group, amended by
`MODES_AND_PANELS.md` Part 1 (the Read/Review/Edit selector, and the
two new View ▸ Window settings). Where this module departs from either
document — and there are a handful of places where they contradict each
other or contradict what exists — the departure is documented at the
site, in the submodule that makes it.

## The rule that shapes this module more than any other

`RIBBON_IA.md` P3, **no placeholders**:

> An unavailable capability renders nothing, not a disabled stub.
> Greying is reserved for *temporarily* unavailable — no document open,
> document encrypted, undo stack empty — and is always explained on
> hover.

`RIBBON_IA.md` §5 marks every command with where it exists today: **G**
in the GUI, **C** in `pdfcer-core`/`pdfcer` only, **N** nowhere. P3
means a **C** or an **N** must be *absent from this manifest*, not
present and disabled — a **C** row is a command whose engine is written
and whose shell is not, which is a cheap win and still not a shipped
command.

Absent is not the same as forgotten, so every one of them is listed in
[`manifest::PLANNED`] with the reason it is not here. That list is
machine-readable, tested against the manifest in both directions, and
is what a later stage reads to find its work.

## Where the strings come from

Nowhere in this module. Every operator-visible string — tab labels, tab
questions, group captions, command labels, command tooltips, mode
labels — is a call into [`crate::text::ribbon`] or
[`crate::text::commands`]. `tools/gates/check-ui-strings.sh` scans this
tree recursively and fails the build on a literal that carries
whitespace, which is the mechanical half of the rule; the reason for
the rule is in [`crate::text`]'s own header.

## Where the behaviour comes from

Also nowhere in this module. A registered command carries an
`egui_shell::HandlerToken`, an opaque `u64` the shell stores and hands
back. This module assigns those numbers and says nothing about what
they do; the application dispatches on them at one choke point, which
is where a confirmation gate, an undo entry or a trace belongs.
