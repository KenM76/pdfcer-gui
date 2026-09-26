# `app::actions::forms::delete` — the two structural delete verbs, and the
gate that is the last door to them

`EditSession` has two verbs for removing part of a form and they are
deliberately different requests:

| verb | removes | the operator pressed |
|---|---|---|
| [`field`] | the field **and every widget it draws**, on every page | *Delete field* in the Properties panel |
| [`widget`] | **one box**, leaving the field — unless it was the last, in which case the engine removes the field too and says so | *Delete this box*, the `canvas.field` menu's Delete, or the Delete key over a widget |

## Why this is a file of its own

R2 (no `.rs` over 1,500 lines) forced the split and the subject boundary
decided where: `super` is the whole form-authoring surface — five `add_*`
specs, fill, import, export, rename, adopt, move — and these two are the
only ones that **destroy** something. Both therefore carry a gate none of
the others needs, and the gate's argument is longer than either verb.

## THE FAILURE MODE THIS FILE GUARDS AGAINST

Clear `doc.selected_field` **before** the engine call and say nothing when
the engine refuses, and on an ordinary certified fillable form — `/Perms
/DocMDP` at `/P 2`, which §12.8.2.2 Table 257 permits *filling* and forbids
*restructuring* — the operator gets:

1. right-click a widget, or press Delete over it;
2. the box stays, because `deletion_refusal` was always going to refuse;
3. the selection vanishes anyway;
4. nothing is said, because `crate::app::actions::apply::vector_edit`'s
   `Err` arm words an un-categorised decline and names no field;
5. **and the Properties panel, which was correctly showing "This document
   does not allow form fields to be removed", goes blank**, because that
   section is drawn from `doc.selected_field`.

⇒ A refused gesture that destroys its own explanation. Two rules answer it,
and both are enforced here:

- **[`refused`]** — a refusal must be a **sentence**, never a silence. R83
  gates the four *controls*, but a gate is a forecast of the engine's guard
  and this verb is where the residue those four cannot cover arrives.
- **[`clear_selection_if_edited`]** — the selection is cleared on **success
  only**, so the panel keeps the field it is describing whenever the
  document did not change.
