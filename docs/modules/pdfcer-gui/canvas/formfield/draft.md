# `canvas::formfield::draft` — what the dialog collects, and what it remembers

A [`Draft`] is the whole of a form field's settings **before** anything
reaches the document. It is what the placement dialog edits, what
`Action::CommitFormField` carries, and what [`Remembered`] keeps for the
next placement.

## One struct for five kinds, and why that is not laziness

`pdfcer-core` has five distinct spec types — `NewTextField`, `NewCheckBox`,
`NewRadioButton`, `NewChoiceField`, `NewPushButton` — and this is one type
with a `kind`. That inverts the usual advice, so it needs its reason.

The five specs share nine of their fields (page, name, rect, tooltip,
read-only, required, border, visibility) and differ in one to five. Modelled
as five GUI structs, the **dialog** would be five dialogs, the **remembered
settings** five stores, and the shared half — which is the half an operator
actually adjusts — would be written five times. Worse, this is the type the
operator's *"remember last settings"* attaches to, and remembering across
kinds is the useful behaviour: someone who turns the border off for a text
field wants it off for the check box they place next.

The conversion to the five engine specs happens in exactly one place
([`crate::app::actions::forms`]), where the unused fields are simply not
read. That is the correct location for the narrowing: at the boundary, once.

## What is remembered and what is not, and the hazard in between

The operator, 2026-08-26: *"remember last settings"*. Everything here is
remembered **except the name**, and the exception is a correctness one
rather than a taste one.

In PDF, **two widgets that share a fully-qualified name are one field**.
`FieldAuthorOutcome::merged` is the engine reporting exactly that. So
remembering the name would mean the second text field an operator places
silently becomes a second *view* of the first — type in one and the other
changes — and nothing on the page would say so.

**Radio buttons are the deliberate inverse**, and are the reason this is a
per-kind rule rather than a blanket one. Radios that share a name are one
control, which is what makes them exclusive; a group of three is three
widgets, one name, three export values. So for [`FormFieldKind::Radio`] the
name **is** remembered and the *export value* is what advances. Getting this
backwards in either direction produces a form that looks right and behaves
wrongly, which is why [`Remembered::next`] states it in code and the tests
assert both halves.
