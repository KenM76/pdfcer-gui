# `ui-verify/checks/chords`

`chords` — **every declared keyboard shortcut is PRESSED and asserted.**

The driven counterpart to `app::keyboard`'s
`every_chord_the_manifest_binds_actually_fires`, and the check whose absence
let **fourteen of twenty-one declared shortcuts ship dead** — `Ctrl+Z`,
`Ctrl+Y`, `Ctrl+Shift+Z`, `Ctrl+S`, `Ctrl+E`, `Ctrl+Shift+E`, `Ctrl+H`,
`Ctrl+Shift+C`, `Ctrl+Alt+N`, `F11`, `[`, `]`, `Alt+Up`, `Alt+Down`.

Undo had a keyboard shortcut everywhere except the keyboard.

# ★★ Why a headless gate is not enough, and why this one is not either

The unit gate presses each chord into a bare `egui::Context` and asserts the
command comes back. That covers the dispatcher and the manifest. It does
**not** cover the link this check exists for: an operating-system keystroke
becoming an `egui::Event::Key` in *this* window, with *this* window's focus
rules and *this* application's other keyboard claimants running.

And this check does not cover what the unit gate does — it presses a fixed
list, so a chord added to the manifest tomorrow is silently unswept here.
**Both are needed and neither is redundant**, which is worth stating because
the argument that killed the last driven attempt was that the keymap test
already covered it. It did not: that test swept `Ctrl+<digit>` only.

# ★ How the belief that this was impossible survived for months

Nine module headers in this crate recorded, as a fact about the machine,
that *"synthetic keyboard input does not reach the target window from the
session that injects it"*. It was inferred from `Ctrl+E` producing no trace
— which was the dead-keymap defect, one layer below where the conclusion was
drawn. Eight of those headers cited `crate::checks::find_bar` as the source;
`find_bar` **passes**, and its own report says *"control chord Ctrl+2
arrived, so the input channel works"*.

So the record contradicted itself in the same run report, for months, and
nobody read the two lines together. The lesson is the one the operator's own
standing rules already state: **a constraint an agent infers about its
environment is a reading, not a fact**, and a reading that stops people
testing something is the most expensive kind.

# What is pressed, and what is deliberately not

Every chord whose command is safe to invoke against an open document with
nothing saved. `F11` (fullscreen) is **excluded**: it resizes the window
mid-run, which would invalidate every rect the harness has measured, and its
dispatch is proven by the unit gate. That exclusion is named here rather
than left to be inferred from a short list.

The assertion is on `chord-command chord=… id=…`, the line
`app::keyboard::commands` traces the moment a chord resolves. That is the
link under test; whether the command then does its work is each feature's
own check.
