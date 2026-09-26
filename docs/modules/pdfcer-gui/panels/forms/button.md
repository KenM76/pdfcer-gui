# `panels::forms::button` — **what an existing push button does, and how to
change it**


## Why this row did not exist until the reader did


A button **already in the document** does not. `pdfcer-core` could write an
action and not read one back, and this project declined to draw the control
anyway. The three ways to do it without a reader were all bad, and the
request said so:

1. **Show "Nothing".** A button carrying `/A << /S /JavaScript … >>` would
   read as inert — pdfcer asserting a fact about somebody else's document
   that it had not checked. The sneaky half of rule 4 in its purest form.
2. **Show nothing, and make the control a one-way *"set this button to:"*.**
   Honest, and an invented interaction. No form editor works that way, and
   this project's standing rule forbids inventing one.
3. **Write first and read the result.** `ButtonActionChange::replaced` names
   what was there — so the only way to learn what a button does would be to
   destroy it. Not a control; a trap.

⇒ Filed as `request_a_buttons_action_can_be_written_and_not_read.md`,
answered by `Pass 212.0`, and this file is the consumption.

## The engine shipped FOUR states where three were asked for

And the fourth is the one that makes the row honest.

| state | what a control may offer |
|---|---|
| `None` | *"does nothing"* — offer to set one |
| `Known(a)` | show it, offer to change it |
| `Unmodelled(s)` | name the subtype, offer to **replace**, never claim to show it |
| `Foreign(s)` | name the subtype, offer **nothing** |

`Unmodelled` and `Foreign` differ in exactly one thing — **whether replacing
is offered** — and that is the decision the operator is being asked to make.
Three states would have forced a wrong answer in one direction: a
`/SubmitForm` this reader does not yet decode is not *"an action pdfcer will
not author"*, because pdfcer authors submits happily, and calling it `Foreign`
would have greyed a row that should be live.

R9 is what makes `Foreign` render *nothing* rather than a greyed Change
button. A greyed control says *"not now"*; the truth here is *"not ever, by
decision"*, and greying would advertise a capability pdfcer has chosen not to
have.

## Where the row is
