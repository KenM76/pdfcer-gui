# `canvas::widgetdrag` — dragging a form field's box to where it belongs

The third module on the annotation branch of [`crate::canvas::dragroute`]'s
fork. [`crate::canvas::dimdrag`] answers for a ce dimension,
[`crate::canvas::annotdrag`] for ordinary markup, and this for a **form
field's widget** — the box an operator types into.

## ★★★ The same defect, one surface along, found by looking for it

Ten days after the annotation drag was found to be silently eaten, this was
the identical state: a widget could be **selected** on the canvas in Edit
mode, its position and size shown in the Properties panel with four numbers
and an Apply button — and dragging it did nothing.

⇒ It was found by asking *"where else does this shape exist?"* rather than by
waiting for a report, which is the whole value of writing the annotation one
up. A class of defect that has been named once is cheap to look for; the same
class waiting for an operator to trip over it is not.

★★ And the operator's own instruction that week was *"work on form field
editing next and the rest of the features required for editing"*. Four
numbers and an Apply button are a form for editing a rectangle. **Dragging is
how a person moves a box**, and every program in this class does it — the
typed fields are the precise route, not the primary one.

## ★★ Why this is not `annotdrag` with a different id

A widget is an annotation — `/Subtype /Widget` — and `move_annotation` would
move its `/Rect` perfectly well. The engine **refuses it by name** anyway:

> `EditError::AnnotationMoveWrongVerb` … for a **widget** (use
> `move_widget(fqn, index, dx, dy)`) … Refused rather than delegated on
> purpose: both of those do strictly *more*, and quietly doing less under
> this name would give you a second way to move the same thing that silently
> produces a worse result.

What `move_widget` does more of is the **field**: a widget is addressed by
its field's fully-qualified name and an index within it, because one field
can draw boxes on three pages and the `/Annots` entry is not the thing an
operator renamed. Addressing it by `ObjId` would work and would be a second
vocabulary for one subject.

★ So this module exists because the **address** differs, not because the
geometry does. That is worth saying plainly: two modules with near-identical
bodies are usually one module, and the reason these are two is a fact about
the engine's API rather than about dragging.

## Rule 4

The ghost is the cursor, which the rule permits by name. Nothing about the
widget is tinted or badged, and a filled field renders exactly as it will
after a save.
