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

## Item notes

### `fn collected_exports`

A deliberately thin view: the shell does not track a radio group's full
membership, so the best it can do is advance past the one it last wrote.
That is enough for the sequential placing this exists to serve, and the
dialog shows the value so an operator placing out of order can correct it.

### `fn next_free`

It starts at 1 and scans upward rather than counting `taken`, because
counting gives a collision the moment anything has been deleted: a document
with `Text1` and `Text3` has two fields, and `Text2` is free while `Text2`
derived from the count would collide with nothing and `Text3` would.

The scan is bounded by `taken.len() + 1` iterations by construction — with
*n* names taken, one of the first *n + 1* candidates must be free.

### `fn both_mk_colours_carry_across_kinds_and_default_to_stating_nothing`

Two claims in one test because they are the two halves of O202's
before-placement ask. Carrying is the ask itself — a row of check boxes
in one colour is the workflow. Not inventing is the part that would
never be noticed: a default of `Some(MkColor::Rgb(1.0, 1.0, 1.0))`
looks identical on a white page and writes a `/MK` key into every field
pdfcer authors, changing bytes in files the operator did not ask to
change.
