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

## Item notes

### `fn a_click_places_something_with_area`

The guard that stops a click producing an invisible field. A zero or
negative default would author a control that exists in the document,
cannot be seen, and cannot be clicked to select — the exact shape of the
zero-height Large control this project shipped once before.

### `fn no_kind_is_authorable_but_inert`

It read *"exactly one kind is authorable-but-inert, and it is the push
button"*, with the instruction: *"if pdfcer ever runs PDF actions, this
test fails and the failure is the prompt to un-grey the button."* On
2026-09-01 it failed for exactly that reason and this is what it became.

Inverted rather than deleted, because the WELD is the point. Three
surfaces have to agree about whether a kind is useful once placed — the
ribbon's `enabled_when`, `app::dispatch::forms`' worded refusal, and this
predicate — and a build where they disagree is one where a greyed control
still works by chord, which is a defect this project has already shipped
once and found by driving rather than by testing.

⇒ So the assertion now says *"the set is empty"*. A sixth kind that pdfcer
can author and not use fails here, and the failure names what to do.
