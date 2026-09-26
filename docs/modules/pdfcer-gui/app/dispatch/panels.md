# `pdfcer-gui/app/dispatch/panels`

`app::dispatch::panels` — the layout verbs that act on a panel or on the
chrome around it.

Float a panel, dock it back, close it, bring every floating one home, and
the two auto-hide toggles for the ribbon and the rail. The seam is the
operand: these are the commands in the program whose subject is *the
shell's arrangement* rather than the document.

# ★★★ The operand problem, which is the whole reason this file has a
shape at all

Float, dock and close act on **the panel the operator right-clicked**.
Nothing in [`super::PdfcerApp::dispatch_command`]'s signature carries
that: it is handed a command id and the application, and a command id
is a verb with no noun.

Three ways to supply the noun were considered.

| | Why not |
|---|---|
| A command per panel per verb (`view.panel_float.layers`, …) | `Panel::ALL` times the operand-taking verbs, in registered commands whose only difference is a suffix, each needing a `CommandText`, a handler token that can never be reused, and a row in the reachability register. The registry would be mostly this. |
| A `HandlerToken` that carries data | A token is an integer the operator's saved key bindings are written against. Making it a payload makes a keybinding file un-writable. |
| **Park the panel beside the dispatch** | What this does. |

⇒ [`crate::app::PdfcerApp::dock_menu_panel`] is set on the line before
`dispatch_token` and read by the arms here. It is the same "park an
answer, drain it immediately" shape `crate::dialogs`' scale hand-over
uses, and `crate::app::surfaces`' own comment on that one states the
rule it follows: the parking site and the draining site are **adjacent**,
so the value cannot be stale.

★★ **It is set from the token's own origin, not from a hover.** The
tab-menu handler runs once per drawn tab per frame — for *every* tab,
whether or not anything was clicked — so a naive `dock_menu_panel =
tab.panel()` inside the handler would leave the field naming whichever
tab happened to be drawn last. Instead `surfaces` collects
`(PanelId, HandlerToken)` **pairs**, because a token only ever comes
back from the one tab whose menu row was actually chosen. The pairing is
exact rather than nearly right, which for a command that closes things
is the difference that matters.

# Why none of these raises an `Action`

`crate::app::actions`' funnel exists for **document** state: the things
an undo log holds and a save writes. A panel arrangement is neither. It
is chrome, it is per-operator rather than per-document, it is persisted
to `layout.ron` by an entirely separate debounce, and it survives
closing every document.

⇒ The panel verbs mutate `self.dock` directly, exactly as
`view.reset_layout` and [`crate::app::PdfcerApp::toggle_panel`] already
do, and the auto-hide toggles write `Prefs`. **No `Action` variant
belongs to any of this.**

# The persistence, which is the part that is easy to leave out

A float, a dock-back and a close all change the layout, and a layout
change is worth saving. The dock reports its own edits through
[`egui_shell::dock::DockFrameReport::layout_changed`] and
`crate::app::surfaces` records those — but **that path only sees
changes the dock made**, i.e. ones that arrived as an `Intent` during
`Dock::show`. A command dispatched from a menu is not one of those: it
runs after the dock has drawn, straight against `DockState::layout_mut`.

So every arm here calls [`record`], which is the same
`Modes::record_layout` the dock's own path calls. Without it the
operator floats a panel, quits, and finds it docked again — with no
error, no trace, and nothing to blame.
