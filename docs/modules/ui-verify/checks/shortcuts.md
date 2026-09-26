# `ui-verify/checks/shortcuts`

`shortcuts` — **the keyboard reference opens, and every chord it declares
names a command this build has.**

# Why this window is worth driving, when its whole design is not to have
a list

`DEFECTS.md` D5 is *"the keyboard-shortcuts reference omits six live
bindings"*. The old shell's reference was a hand-maintained list in a
7,912-line catalog; six bindings existed and were not in it, and nobody
noticed, because **a reference is read by operators and by no test**.

The new one holds no list at all — `dialogs::shortcuts::rows_from` folds the
live keymap against the live command registry — so D5's specific failure is
structurally impossible. What replaces it is a *different* failure with the
same symptom, and this check is aimed at that one:

> a chord in the manifest naming a command the registry does not have.

Such a chord is dropped from the window and **does nothing when pressed**.
The key is declared, the operator reads about it nowhere, presses it, and
gets silence. That is R8's failure mode — capability presence is expressed
by registration — arriving through the keymap instead of the ribbon.

# Why `dropped == 0` is the assertion, and why it is not tautological

On a **full** build every chord's command is registered, so the number must
be zero, and any other value means the manifest and the registry have
drifted. Nothing else in the workspace checks that pairing end to end:
`shell::commands::reach` proves every *registered command* is routed, and
this proves every *bound chord* has a command. They are opposite directions
and neither implies the other.

It is deliberately **not** a unit test. `rows_from` is unit-tested against
hand-built keymaps, which proves the folding is right and says nothing about
the pair this executable actually shipped with — the manifest is loaded from
RON at startup and the registry is populated at runtime.

⚠ On a **stripped** build a non-zero count is correct and expected, and the
window says so in words. This check would then fail, and the right response
is to teach it which build it is looking at rather than to soften the
assertion — see the note at the assertion itself.

# Phases

| Phase | Does | Expected |
|---|---|---|
| A | click **File ▸ Keyboard shortcuts**, with NO document open | `dialog:shortcuts` declared |
| B | read the census trace | `shortcuts-listed commands=N dropped=0`, `N > 0` |
| C | check the list drew | `shortcuts.list` declared with a non-empty rect |
| D | capture the window | attached as evidence |
