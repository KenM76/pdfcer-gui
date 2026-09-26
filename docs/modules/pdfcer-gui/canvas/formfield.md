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

### `enum FormFieldKind`

Exactly the five `pdfcer-core` has verbs for — `add_text_field`,
`add_check_box`, `add_radio_button`, `add_choice_field`, `add_push_button`.
The list is not a design choice here and must not become one: a sixth entry
would be a button with nothing behind it, which is the placeholder R9
forbids.

### `fn is_useful_once_placed`

It answered `false` for [`Self::PushButton`] for the life of the
project. Authoring worked — `add_push_button` places a correct widget —
but the resulting control ran nothing, because giving a button an action
means writing `/A` and `pdfcer-core` authored none by decision 009
posture A. A button that looks right and does nothing is this project's
recurring failure mode, so the tool was greyed and the placement dialog
said why.


**The predicate is kept rather than deleted**, and not out of
sentiment. `app::dispatch::forms` decides its worded refusal on it and
the ribbon's `enabled_when` is welded to it by a catalog test — so a
future kind that pdfcer can author and not use has one place to say so,
and both surfaces follow from it. Deleting it would mean the next such
kind ships a control that does nothing, silently, which is the whole
class of defect this predicate exists to make impossible.

It is a `const fn` returning a literal `true`, which clippy would
otherwise call trivial. That is the point: the interesting state is that
**nothing** is currently in the excluded set.

### `fn command_id`

Ids rather than a shared command with a parameter, because R8 says
**registering a command is the only way the GUI learns a capability
exists** — and it is what lets a build without one of these simply not
register it, with the ribbon item disappearing rather than being
special-cased.

### `fn default_size_pt`

A click has to mean something, and a zero-sized field is not it. The
numbers are per-kind because the kinds are not the same shape: a text
box is wide and one line tall, and a check box is square. Sizing them
alike would make every click need a resize afterwards, which defeats
the point of offering a click at all.

The text height is one line at the size a form typically uses, and the
square kinds match it so that a check box beside a text field sits on
the same baseline.

### `fn noun`

Returns the **text function**, not a string, so the words themselves
stay in `crate::text` where `check-ui-strings.sh` can see them. The
contrast with [`Self::name_prefix`] two functions down is the whole
point and is easy to get backwards: that one is a PDF `/T` written into
the file and must never be translated; this one is prose in a status
line and must always be.

### `fn name_prefix`

**A PDF name, not UI copy, and the distinction is load-bearing** —
which is why these are literals here rather than in `crate::text`. This
string is written into the file as the field's `/T`, is what a
form-filling script keys on, and is what an FDF import matches against.
Translating it would rename every field in a document opened by an
operator running a different language, and the renaming would be
invisible until the data import failed.

The prefixes are Acrobat's own, so a form authored here and a form
authored there are named alike.

### `fn drag_shape`

Always a rectangle: every form control is a `/Rect`, and none of them
has a second geometry. Returning it explicitly rather than assuming it
at the call site keeps the borrowing visible.
