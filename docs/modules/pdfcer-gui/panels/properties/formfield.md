# `panels::properties::formfield` — the properties of a form field clicked
on the page

**Operator request, 2026-08-26:** *"don't forget that when I click on an
existing form field on the page it's properties should come up in our side
pane for editing it's properties."* This is the side pane's half; the click
is `crate::canvas::forms`'s.

## What can be changed, what can only be read, and why the difference
is disclosed rather than hidden

`pdfcer-core` has exactly four verbs for a field that already exists:
`rename_field`, `delete_field`, `delete_widget` and `fill_text_field`. It has
**none** for a field's flags — required, read-only, multiline, comb, the
border, the tooltip. Those are settable only at authoring time, through the
five `add_*` specs.

So this panel offers rename, delete and delete-this-box, and **shows the
rest as read-only facts**. That is a real limitation and it is stated in the
panel rather than expressed as an absence, because an absence is
indistinguishable from an oversight:
[`crate::text::panels::formfield::not_editable_note`] says which properties
cannot be changed after placing and what to do instead.

It is also written up as an engine request rather than worked around.
The standing rule is *report every workaround, even a successful one* —
anything the GUI has to work around is a place the crate boundary was drawn
wrong. A properties panel that can show a flag and not change it is exactly
that shape.

## Why this is not `SelectionState`, and why the panel says so

`crate::app::state::SelectedField`'s doc carries the argument: a form field
is a document-level entry with a **name** for identity and possibly several
widgets on several pages, where everything `SelectionState` holds has
paint-order indices, a bounding box and drag handles. Merging them would arm
the Format tab's Delete over a field and hand the resize grips a rectangle
nothing can move.

The visible consequence, which this panel is careful about: **a field is not
"selected" in the sense the rest of the shell means.** No handles appear, the
Format tab does not open, and Delete on the keyboard does not remove it. The
delete controls are *in this panel*, labelled, and there are two of them
because "remove this box" and "remove this field" are different requests.

## Rename and Delete are OFFERED ONLY WHERE THEY WOULD WORK (R83)


The consequence, on the ordinary real-world certified fillable form: three
live controls — Rename, Delete field, Delete this box — every press of which
returned a refusal to the trace and nothing at all to the operator. That is
the failure this project is named after in miniature: a visible control that
is silently inert.

Both gates are now asked in [`section`], **before anything is drawn**, and
each control asks **its own question**: `rename_refusal` for the rename box,
`deletion_refusal` for the two delete buttons. They compute the same answer
today and are deliberately separate functions on core's side; that
reasoning, and why borrowing one for the other is a silent-failure waiting
on a spec nuance, is quoted in full at the call site.

Where a gate refuses, the controls **are not drawn at all** and a sentence
takes their place. R9: greying is for the temporarily unavailable and must
explain itself on hover; a certification signature is neither temporary nor
arguable. And a sentence rather than a silence, because a panel that simply
omits half its controls looks half-drawn.


The fix above closed the panel and claimed the rule. It was true here and
nowhere else: the `canvas.field` context menu's Delete carried no
`visible_when` at all, `canvas::keys`' Delete rung 0 asked nothing, and
`app::conditions`' `selection.delete_permitted` was guarded by
`doc.selected_field.is_none()` — **false whenever a field is selected**,
so the gate was a no-op by construction on exactly the state it was written
for.

Worse, `app::actions::forms`' delete verbs cleared `doc.selected_field`
*before* calling the engine, and this section is drawn from that field. So
on an ordinary certified fillable form, pressing the Delete those three
surfaces offered left the box in place, said nothing, **and blanked the
sentence above** — a refused gesture that destroys its own explanation.

⇒ [`refuses_delete`] is now the single derivation all four doors read, and
the verb clears the selection on success only
(`app::actions::forms::delete`). The driven proof is
`tools/ui-verify`'s `field_delete_gate`, which presses Delete on a
certified form and asserts the sentence is **still on screen afterwards**.

## The rename box is a draft, not a live write

Typing into a `TextEdit` bound straight to the field would rename on every
keystroke — `Address` would pass through `A`, `Ad`, `Add`, each of them a
real rename of a real field, each undoable separately, and any of them
capable of colliding with an existing name. So the box holds a draft in
`PanelsState` and a button commits it, which is the same shape
`super::geometry` uses for the same reason.

## Item notes

### `const REGION`

A published region name is a cross-repo stability contract: the harness
asserts on it by string, so renaming one turns a check into a skip rather
than a failure.

### `const REGION_RENAME`

"Only when drawn" is the whole value of it. On a document that refuses a
rename this section draws a sentence and no control at all (R9), so the
region's *absence* is the evidence a driven check reads — and it is
admissible evidence only because [`TRACE_GATES`] is written on every frame
either way, so a check can tell "the control was withheld" from "the section
never drew". `crate::checks`' rule 4 in the harness states the same
obligation from the other side: never treat an absence as evidence unless
you have shown the thing that would have produced it was working.

### `const REGION_DELETE_REFUSED`

[`REGION_DELETE`]'s negative twin: exactly one of the two is declared on any
frame this section runs, so a driven check can tell *"the control was
withheld and explained"* from *"the panel never opened"* — the second half
being what [`TRACE_GATES`] proves. See `REGION_RENAME` for the rule.

### `fn row`

`truncate()` rather than wrapping, and the value on hover. A
fully-qualified name can run to any length, and a panel row that grew to
three lines would push everything under it around as the operator clicked
between fields — the same restlessness `disclosure_line` exists to prevent
in the status bar.

### `fn rename_row`

# On a document that refuses a rename, this draws a SENTENCE and no box

`refused` is `EditSession::rename_refusal`'s answer, asked in [`section`]
before anything was drawn. When it is true the operator gets one line saying
the document forbids it, and **no text field and no button** — which is R9's
ruling rather than a preference:

- Greying is for a capability that is *temporarily* unavailable, and is
  always explained on hover. A certification signature is not temporary and
  cannot be argued out of.
- A permanently-refused capability renders **nothing**, or a sentence saying
  where the thing actually lives. Here there is no elsewhere, so it is the
  sentence.

And a sentence rather than silence, because the section around it is full of
controls: an operator who finds the rename box missing with no explanation
has found a panel that looks half-drawn.

What this replaces is worse than either. Before this, the box and the
button were drawn unconditionally, the operator typed a new name, pressed
Rename, and the engine refused **after** the typing — with the refusal
reaching the trace and nothing else. That is the shape R83 exists to remove:
discovery by pressing, on a control the program already knew would refuse.

### `fn delete_row`

Two, not one, and they are different requests. See
`Action::DeleteFormField`'s doc: one field may be drawn in several places,
so "remove this box" and "remove this field" have different consequences,
and offering only one of them makes the other impossible.

The per-box control renders **nothing** when the field has one widget, which
is R9 rather than greying: with one box the two buttons would do the same
thing, and a control that duplicates its neighbour is worse than absent.

# `refused` — the second half of a finding, and this is what it cost

`EditSession::deletion_refusal` has existed for the whole life of this
shell, carries a doctest that spells out this exact call site, and was
**consulted by nothing**. It appeared in this crate only inside comments
— three of them, in `panels::forms`, arguing correctly about which query
Flatten should ask and never noticing that Delete asked none at all.

So both of these buttons were drawn live on every document, including a
certified one, and every press of them returned the same refusal to the
trace and nothing to the operator. R83's whole subject.

The remedy is the same as the rename box's, for the same reason: the
controls are **not drawn**, and a sentence takes their place. Deleting a
field and deleting one of its boxes are both structural, so they share one
gate and one sentence — unlike rename, which asks its own query in
[`section`] because it is a different question that happens to have the same
answer today.

### `fn certifier`

Named by `/T` rather than discovered, because the whole value of the
fixture pair is that the two documents are identical apart from the
catalog's `/Perms` — a test that went looking for "a field" could find a
different one in each and report the difference as a gate difference.
See `tools/gen-certified-fixture.py`.

### `fn a_certified_document_refuses_to_delete_a_selected_field`

The positive half. `fixtures/certified-comments.pdf` carries an enforced
certification — `/Perms << /DocMDP … >>` with `/P 2` — and §12.8.2.2
Table 257 permits *filling* such a form while forbidding changes to its
structure. Deleting a field is a structural change, so
`EditSession::deletion_refusal` must answer `Some`.

### `fn an_uncertified_document_permits_deleting_a_selected_field`

The two fixtures differ in **one dictionary** (`tools/gen-certified-fixture.py`
builds both from one function), so any difference between these two
assertions is caused by that dictionary and by nothing else.

⇒ This half matters more than it looks. A derivation that refused
unconditionally would satisfy every other test in this module and would
be a **worse** defect than the one being fixed: a control withheld where
it would have worked leaves the operator no gesture that reports it.
`threaded-comments.pdf` carries the same `/Type /Sig` as an ordinary
approval signature, and an approval signature is not an enforced
certification.

### `fn no_field_selected_is_not_a_refusal`

The derivation answers *would the engine refuse?* and never *is there
anything to delete?* — the second question is `selection.actionable`'s.
Without this the condition ladder in `crate::app::conditions` would take
the field arm's answer on a frame with no field selected, and
`format.delete` would vanish from the `canvas.object` menu on every
certified document for a reason about forms.
