# `canvas::formfield` — placing a new form field on the page


> *"get all the form buttons on the ribbon working next along with adding
> all the form feature buttons. when I click one I should be able to click
> on the canvas to place the position or drag a box for size then a pop up
> lets me set the details for the feature."*

That is exactly the interaction the existing command's own tooltip has
promised since it was written — *"Click where you want it, or drag out the
exact size."* The design was specified and never built.

## Why it was never built, and why that reason was wrong

`shell::commands::reach::register` recorded `edit.form_create_field` as
blocked on *"core's STRUCTURAL certification gate"*. **There is no such
gate.** Probed on 2026-08-26 against a real drawing:
`EditSession::add_text_field` authors a field and returns its id.

What the engine refuses is a spec whose tooltip is `Undecided` —
`TooltipDecisionRequired`, an **accessibility** requirement rather than a
permission. A form control owes a screen reader a name and the engine will
not invent one silently. So the entire blocker is a field of the very dialog
this feature needs. `app::actions::forms::authoring_is_available` is the
standing test, and it asserts both halves so it cannot rot into a
tautology.

Fourth stale blocker in this project. The standing rule that produced the
probe: **a backlog row is a record, not evidence.**

## The shape, and why it borrows from markup rather than inventing

Placing a field is *geometrically* the same act as drawing a markup
rectangle: arm a tool, put a rectangle on a page, commit once. So it reuses
that machinery rather than growing a second one —
[`crate::canvas::markup::band`]'s two-phase drag, the same page-space
conversion, the same single-`Action` release.

It differs in exactly one way, and the difference is the feature: **the
release does not author anything.** It opens a dialog. Nothing exists in the
document until the operator presses OK, which is what makes Escape free and
what stops a mis-drag leaving a stray field behind.
