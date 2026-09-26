# `ui-verify/checks/document_tabs`

`two_documents_get_two_tabs` — opening a second PDF adds a tab, and the
tab switches to it.

# The gap this closes


> *"make it so we can open multiple PDFs at once…"*

# Why this needs driving, and could not be unit-tested

`crate::app::documents` has nine unit tests over the tab arithmetic — the
strip order, the browser close rule, the wrap on Ctrl+Tab. **Every one of
them passes on a build where the strip is never drawn**, because the
arithmetic is a pure function of two fields and the drawing is four
frame-level facts none of them can reach:

1. the top panel is composed at all, and *before* the docks, or it starts
   at the dock's edge instead of spanning the window;
2. `egui_shell::tabstrip` is handed a non-empty list;
3. a tab's rectangle is somewhere an operator can click;
4. a click on it reaches `activate_slot`.

Exactly the shape of gap this project was founded on: `panels::pages` had
its whole capability, a passing test suite, and **no registration**, so an
operator never saw it (`panels::pages`' header §1, *"invisible rather than
broken, which is the honest failure and also the silent one"*).

# What a passing run does NOT prove

That the labels are legible, or that the ✕ closes anything. The first is
[`super::legibility`]'s kind of question and the second is a destructive
act this check deliberately does not perform — a close behind an
unsaved-edits prompt is its own gesture with its own failure modes.
