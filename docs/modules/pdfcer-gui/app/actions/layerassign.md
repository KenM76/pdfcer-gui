# `app::actions::layerassign` — put the selection on a layer

`LayerAction::Assign { layer }` moves the selection onto `layer`, or off every
layer when it is `None`. The operand is read from the selection when the act
is applied (`operand` / `operand_of`), in this order:

1. a selected form field → its widget's id (from `parse_acroform`), an annotation;
2. a selected annotation;
3. whole page objects on one page, by paint-order index.

A selection across pages (`AssignSeveralPages`), one holding a part of an
object (`AssignParts`: a part shares its object's layer) or nothing
(`AssignNothing`) is refused in words before the engine is called.

| Operand | Verb | Funnel |
|---|---|---|
| `Objects{page, indices}` | `EditSession::set_objects_layer` | `funnel::vector_edit_on_page` |
| `Annotation{page, id}` | `EditSession::set_annotation_layer` | `apply::vector_edit` |

Each is one undo entry; a move that changes nothing records none. The receipt
names the layer and the count; the engine's `disclosures` follow it. When an
annotation's previous `/OC` was not a registered layer (a membership rule),
the receipt says the rule was replaced.

`offered` (operand valid and the document has a layer) drives the
`selection.layer_assignable` condition that shows `format.move_to_layer` on
the canvas menus and enables it on the ribbon.

## Refusals

`LayerNotFound` → `NotFound`, `LayerContentNotRewritable` → `AssignOwnRule`,
`AnnotationLocked` → `AssignLocked`, `LayerSectionHoldsTaggedContent` →
`AssignTagged`, `LayerSectionCrossesNesting` / `LayerSpanUnbalanced` /
`OverlappingObjectSpans` → `AssignTangled`. Anything else keeps the funnel's
generic sentence, with the engine's words on the trace.

## Trace

`layer-assigned kind=objects page= moved= unchanged= layer=<num_gen|none>`,
`layer-assigned kind=annotation id= subtype= changed= popup= layer=`, and
`layer-assign-refused reason=` for a selection refused before the engine.

Verified by `ui-verify` check `layer_assign_moves_the_selection`.
