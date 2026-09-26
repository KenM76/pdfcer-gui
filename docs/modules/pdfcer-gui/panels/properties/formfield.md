# `panels::properties::formfield` — the properties of a form field clicked
on the page

**Operator request, 2026-08-26:** *"don't forget that when I click on an
existing form field on the page it's properties should come up in our side
pane for editing it's properties."* This is the side pane's half; the click
is `crate::canvas::forms`'s.

## ★★★ What can be changed, what can only be read, and why the difference
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

★★ It is also written up as an engine request rather than worked around.
The standing rule is *report every workaround, even a successful one* —
anything the GUI has to work around is a place the crate boundary was drawn
wrong. A properties panel that can show a flag and not change it is exactly
that shape.

## ★★ Why this is not `SelectionState`, and why the panel says so

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

## ★★★ Rename and Delete are OFFERED ONLY WHERE THEY WOULD WORK (R83)


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
