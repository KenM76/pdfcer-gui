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

## ★ Every row can be pressed with the name box empty, and that is the
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

## ★★ The pre-flight, asked once per row before the press

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
