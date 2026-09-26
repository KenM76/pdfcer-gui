# `ui-verify/checks/add_text`

`add_text` — **`edit.add_text` places a caret and REAL keystrokes reach it.**

The command literally labelled *"Add text"*, driven end to end with the
keyboard, and the one path in this shell that had no driven coverage at all.

# Why the existing text-editing check does not cover this

`checks::text_edit` drives the *other* verb (`edit.text`, which rewrites a
run already on the page) and supplies its characters through
`PDFCER_DIAG_TYPE` — a seam that writes straight into the draft and
**bypasses the event loop entirely**. Its header states the reason: at the
time it was written, synthetic keyboard input was believed not to reach the
target window from the session that injects it.

**That belief is false**, and this check is the demonstration. It is worth
recording how it survived: it was written down in three module headers as an
established fact about the machine, and every check that might have
contradicted it either used the seam or clicked instead. A constraint an
agent infers about its own environment is a *reading*, not a fact — and this
one cost the project its entire keyboard surface, because while it stood
nobody drove a chord, and while nobody drove a chord fourteen of the
twenty-one declared shortcuts sat dead in the manifest.

# What it proves that a unit test cannot

`canvas::textedit::typing` reads `ui.input(…).events` directly. A headless
test can push an `egui::Event::Text` into a `RawInput` and watch the draft
grow — and that passes on a build where no character ever reaches the
window, because the harness supplied the event that a keyboard was supposed
to. The link this check adds is the one nothing else covers: **the operating
system's keystroke becomes an `egui::Event::Text`.**

# The phases

| Phase | Does | Expected |
|---|---|---|
| A | Edit mode, Edit tab, click **Add text** | `text-edit-tool tool=TextEdit(Add)` |
| B | click blank paper | `text-edit-caret kind=Add` |
| C | **type two real characters** | `text-edit-typing … len=2` |
| D | click elsewhere — clicking away commits | `add-text`, and the page changes |

Phase D is the shell's own rule rather than a convenience: `textedit::click`
commits an existing draft before starting a new one, because clicking away
is every editor's *"that word is finished"*. No Enter is pressed, so the
commit path under test is the one a mouse-driven operator actually takes.
