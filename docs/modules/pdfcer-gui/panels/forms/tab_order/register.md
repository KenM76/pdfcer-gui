# `panels::forms::tab_order::register` — the rows that put an unclaimed
form control back into the form

## Why this is here and not in a dialog of its own

Because [`super::model`] already answers the question the dialog would have
had to re-ask. *"Which widgets does this page list that no field claims?"*
is one `/Annots` walk cross-referenced against one parsed `/AcroForm`, and
the Tab-order section performs it every frame it is open — it is the whole
reason that section exists.

A separate Register window would have walked `/Annots` a second time, from a
second parse, at a second moment. Two answers to one question is how a list
and the button beside it come to disagree about the set, and the failure is
silent: the operator presses Register on the third box and a different third
box is registered.

It is also where the operator already is. They opened Tab order because
something on the page would not fill, and the section told them *"3 boxes on
this page are drawn as form controls that no field claims"*. The next thing
they want is to do something about it, and R9's spirit — no control that
looks available and is not — reads the other way round here: **a stated
problem with no offered remedy is the same defect wearing different
clothes.**

## Every row can be pressed with the name box empty, and that is the
recommended answer

The engine measured a real form and found **11 of 13** unclaimed widgets to
be merged field-widgets (§12.7.3.1) — one dictionary serving as both field
and widget, carrying its own `/T`, `/FT`, `/V` and `/DA`. For those,
`adopt_widget(id, None)` recovers the field exactly as it was, and typing a
name would *override* the one the file already holds.

The other 2 were **bare kids** with no identity at all, and they refuse. The
refusal is worded, arrives in the status bar, and says what typing a name
will actually produce — a new, empty field, not the radio button that was
lost. See [`crate::text::status::adopt_declined_no_name`].

## The pre-flight, asked once per row before the press

`EditSession::adopt_preview` is `&self` and writes nothing. Each row asks it
— with whatever the operator has typed **so far**, not with `None` — and the
answer decides the row:

| preview says | the row shows |
|---|---|
| `Ok(outcome)` | a live button reading **"Register as `Address`"** — the name from the file |
| `Err(WidgetHasNoFieldIdentity)` | a greyed button whose hover says typing a name will **create** a field, not recover one |
| `Err(FieldNameTaken)` | a greyed button whose hover explains why two fields with one name are one field |
| any other `Err` | greyed, and the hover **does not guess** — see [`refusal_hint`] |

### Why this is safe to draw from

Because the preview and the call **share one guard set**. The engine split
`adopt_plan(&self, ..)` out of `adopt_widget` rather than writing a second
implementation, and stated the reason in terms this project keeps arriving
at independently: *"two implementations of one guard set are two things that
must agree, and eventually will not. The way that fails is a greyed-out
control for an operation that would have worked, or a live one for an
operation that refuses — which is worse than the discovery-by-pressing it
was meant to replace, because now the shell is confidently wrong instead of
silent."*

There is a test on their side that would notice if a later change gave the
preview its own body. *"The preview said yes and the call refused"* is not a
state the code can reach.


It read *"Why there is no pre-flight, said out loud rather than left as a
gap"*, and described the honest interim: every row offered a live button,
and a bare kid refused **after** the press with a worded decline. It cost
one press and converged, and what it could not do was say *in advance*
which shape a box was.

That paragraph existed for about six hours. The request went out naming the
consequence — discovery by pressing — and `adopt_preview` shipped the same
day. It is kept as a correction rather than deleted because the shape
recurs: **the interim was not wrong, and writing down exactly what it could
not do is what made the ask specific enough to be answered.**

### Two facts that only a pre-flight can deliver in time

Both were in the request and neither is cosmetic:

- **the name.** For a blank box it is in the file and **not on screen** —
  the widget belongs to no field, so no field row names it. A button reading
  *"Register"* is a guess; one reading *"Register as `Address`"* is a
  decision.
- **`field_type: None`.** The registration will **succeed** and the box will
  **still not be fillable**, because `/FT` is inheritable and a top-level
  field has no ancestor left to inherit from. Disclosed after the fact, that
  sentence tells an operator their successful action did not do what they
  wanted. Disclosed before it, it is a choice.

## Item notes

### `struct Drafts`

# Keyed on `(path, edit_epoch)`, which is what makes undo correct

Exactly `super::super::FormsUi`'s rule, for exactly its reason, and it is
worth restating because the consequence here is the opposite of what a
naive reading suggests.

A successful registration bumps the epoch, so **every draft in this map is
discarded**. That is right rather than lossy: the widget the operator was
typing about is no longer unclaimed, its row is gone, and the box they typed
into does not exist any more. Keeping the text would mean re-showing it
against whichever box happened to take that row next.

An **undo** bumps the epoch too, which restores the row and clears the box.
Also right: the name went into the document and came back out, so a box
still holding it would be showing a value the document no longer has, which
is the precise defect `FormsUi`'s own comment records for the fill drafts.

### `fn id`

Distinct from `FormsUi`'s. The two are different lifetimes of thing
keyed the same way — one holds field values, one holds proposed field
names — and sharing an id would make each frame's store overwrite the
other's.

### `const REGION_PREFIX`

Suffixed with the widget's **tab position** rather than its index in the
unclaimed list, because the position is the only stable thing about a row
across a registration: registering one removes it from the list and
renumbers every index after it, so a check that pressed "row 1" twice would
press two different widgets and a check that pressed "the box at position 4"
twice would press the same one or find it gone. The second is a question
with an answer.

### `fn refusal_kind`

Deliberately not the error's own `Display` prose: that is a sentence, and a
trace line is parsed by field. `check-ui-strings.sh`'s exclusion 3 says an
error type's prose is not permission to route text through it, and this is
the same rule pointing at the diagnostic channel instead of the screen.

### `fn refusal_hint`

# Three named arms and a catch-all that does NOT guess

`pdfcer_core::edit::EditError` is `#[non_exhaustive]`, so this needs a
wildcard whatever it does. The question is what the wildcard says, and the
answer is *"pdfcer cannot, and this panel does not know why"* rather than a
plausible guess.

Two of the five refusals are unreachable from here by construction —
`NotAWidget` and `WidgetAlreadyOwned`, because the ids come from exactly the
widgets the `/Annots` walk found unclaimed — so reaching the catch-all means
the listing and the engine disagree about what this widget is. That is a
fault to find in the trace, and handing an operator a confident wrong reason
for it is worse than handing them none.

### `fn a_page_with_nothing_unclaimed_draws_nothing`

R9: a page whose widgets are all claimed has no problem to offer a
remedy for, and a "0 boxes need registering" line is a placeholder
wearing a number.

### `fn each_unclaimed_widget_gets_its_tab_position`

Asserted through the positions rather than through pixels: a row for
position 2 must exist and must say 2, because the number is the only
handle the operator has on a box with no name — they press Tab that many
times to find it.

### `fn an_edit_forgets_every_typed_name`

The property that makes undo correct here — see [`Drafts`]. Exercised on
the struct rather than through a frame, because the thing being asserted
is the key comparison and not the widget.

### `fn the_draft_store_does_not_collide_with_the_fill_panel`

Two `Clone` types in one `data` store under one id is a silent
overwrite: whichever stores second wins, and the symptom is a text box
that forgets a keystroke at a time.

### `fn an_unexpected_refusal_says_so_rather_than_guessing`

The two the operator can act on get their own sentence. Everything else
gets one that says pdfcer cannot and does not say why — because reaching
it means the listing and the engine disagree about what this widget is,
and a confident wrong reason for that is worse than none.

`WidgetAlreadyOwned` is the probe worth having: it is the refusal that
would arrive if this panel ever offered a widget that already has a
field, and *"type a different name"* would be actively misleading advice
about it.

### `fn the_button_names_the_field_when_the_preview_knows_it`

The blank-box case is the one that matters: the name comes out of the
FILE, and it is a string the operator has never seen — nothing in the
panel could have shown it, because the widget belongs to no field and so
no field row names it.

### `fn the_typeless_warning_says_both_halves`

Rule 4's half that survives: an inference the operator cannot
see. Both halves have to be in the sentence — a hover that only said
"this will register" would be true and useless, and one that only said
"no viewer can fill it" would read as a refusal for something that is
about to succeed.

### `fn a_dotted_name_greys_the_register_button_and_the_hover_names_the_rule`

[`crate::app::actions::forms::correctable`]'s reachability table marks
this surface **yes** for `DottedPartialName` — the one route of three
that can raise it — on the grounds that this box is free text gated only
on non-empty. True of *this shell's* gate. Beside the point, because the
engine put `reject_dotted_partial` inside `adopt_plan`, and
`EditSession::adopt_preview` is documented as sharing that plan by
construction. The refusal therefore arrives in the preview this row draws
from, the button greys, and the press the table describes cannot happen.

⇒ **A guard's placement decides which surface has to explain it.** The
engine moved this one for its own reasons — one predicate for three
enforcement sites — and the disclosure moved with it, out of the status
bar and into a hover, silently.


Driven on `ORPHAN_WIDGET`, the hand-authored fixture: one page, one
`/Widget` owned by no field and no `/AcroForm` at all, which is the only
shape this panel offers a Register row for.

### `fn a_name_with_a_bare_dot_is_refused_with_its_own_sentence`

⇒ *a private predicate with three callers is three behaviours until
something forces them to agree.* Asserted here because this surface is
one of the three, and because a regression would present as the hover
going back to the catch-all rather than as anything visibly broken.

### `fn only_unclaimed_widget`

Derived through [`super::super::model::collect`] rather than typed as
a literal `ObjId`, and that is not fastidiousness about magic numbers:
**it is the derivation the row the operator presses uses**, so the tests
above drive the id this surface would hand to `FieldAction::Adopt`. A
number typed into a test is a claim about bytes nobody re-reads, and this
project has already had a harness report defects that did not exist from
exactly that.

`form` is `None` here — the fixture's catalog has no `/AcroForm`, which is
the whole point of it — and `collect` is documented to put every widget in
`unclaimed` in that case.

### `fn rows`

`page_index` is 0-based — it is carried into the action for the trace and
the re-raster, not for the engine, which edits the document-level
`/AcroForm` and never asks which page.

# At most one registration per frame, and it is not an accident

The loop `break`s after a press. Two presses in one frame would queue two
`AdoptWidget`s against a listing computed **before** either ran, and the
second would be acting on a set the first has already changed — the same
stale-index hazard the engine hit in its own CLI and described plainly:
*"the indices shift after every add … I got this wrong myself and nested
something two levels deeper than intended, and the output looked entirely
plausible."*

The ids here are stable where indices are not, so the second action would in
fact still name the right widget. The `break` is kept anyway, because
*"queue only what was computed against the state you have"* is the property
worth holding mechanically rather than re-deriving each time a queued verb
is added. It costs the operator nothing: physically, one press per frame is
all there is.
