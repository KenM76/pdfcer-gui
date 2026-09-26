# `ui-verify/checks/form_field`

`form_field` — **place a form field on the page, then click an existing one
and get its properties.**

The driven assertion for the operator's request, in both its halves:

> *"when I click one I should be able to click on the canvas to place the
> position or drag a box for size then a pop up lets me set the details for
> the feature"* … *"when I click on an existing form field on the page it's
> properties should come up in our side pane for editing it's properties."*

# ★★★ Why this check is the only oracle for most of the feature

Everything between a click and an authored field crosses boundaries a unit
test cannot: an armed tool in `egui::Memory`, a gesture resolved from a real
pointer, a canvas→page transform against a real page, a **second OS window**,
and five `pdfcer-core` verbs. The unit tests cover each rule; not one of them
covers the sequence, and the sequence is where this project's defects live.

The precedent is the shell's own founding defect: the Delete key's guard was
*"analysis-confirmed, NOT empirically verified"*, its unit test built a bare
context with no widgets, and the condition that broke the real application
could not occur in the harness.

# ★★ The dialog is answered by a seam, and that is not a shortcut

`PDFCER_DIAG_FORM_ACCEPT=1` makes the placement dialog press its own Add on
the first frame it is authorable. This harness drives **one** window — the
one `Session::launch` found — and the dialog is a deferred viewport with a
window of its own, so without the seam everything downstream of placing is
unreachable: the five engine verbs, the narrowing in
`app::actions::forms::author`, and all four rule-4 disclosures.

Two seams already exist for exactly this shape — `PDFCER_DIAG_OPEN_PATH` and
`PDFCER_DIAG_INSERT_PATH`, both substituting the answer to a native picker.
What this one substitutes is the **operator's press**, not the authoring:
it sets the same flag the Add button sets, so the readiness guard, the
action, the remembering and the engine call are all the path an operator
takes.

# ★ The two clicks aim at deliberately different places

The first must land on **empty page** — a click on an existing widget would
place a field on top of one, which is legal and would make the second phase
ambiguous. The second must land on a widget whose canvas rect the
application itself published in a `form-box` line, so the check aims at
where the program says the box is rather than at where the fixture author
thought it would be. ★★ **A click that hits the field next to the one it
aimed at produces the same screenshot as a click that worked**, so the only
safe target is a rect the application itself published this frame.

# Phases

| Phase | Does | Expected |
|---|---|---|
| A | launch with `mode.edit,edit.form_text_field` | `form-tool-armed kind=Text` |
| B | click empty page | `form-field-open kind=Text`, then `add-form-field` succeeded |
| C | Escape, then click a published `form-box` | `form-field-selected field=…` |
| D | read the properties region | `properties.form_field` declared |
