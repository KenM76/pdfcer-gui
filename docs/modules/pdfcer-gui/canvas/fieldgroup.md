# `canvas::fieldgroup`

Dragging several selected form fields together. With one field selected the
single-box route (`canvas::widgetdrag`) answers as before; `fieldgroup::drag`
returns `None` for it.

## Members

The primary (`OpenDoc::selected_field`) and every box in `OpenDoc::field_set`
that lies on the primary's page and is placed this frame
(`canvas::forms::placed`). Selected boxes on other pages are left where they
are: the operator cannot see them follow, so moving them would be a sneaky
edit.

## Where a group drag starts

`canvas::pressing` sets `widget_body` when the press lands inside any
member's box (`fieldgroup::body_at`), not only the primary's. The primary's
resize grips keep their precedence over the body: they act on the primary
alone. `canvas::dragroute` asks `fieldgroup::drag` first in its form-field
arm, before `widgetdrag::drag`.

## Ghosts

One rectangle per member, its box translated by the drag delta, painted in
the same group-ghost slot as `canvas::annotgroup`'s.

## Commit

The release raises one `VectorAction::MoveMixed { gesture: "drag" }` with
every member in `widgets` and one page-space delta; `app::actions::movemixed`
applies it and folds it into one undo step. Batch verb owed: G189.

## Trace

`move-mixed-applied gesture=drag page objects widgets=moved/asked annots
one_step moves`.
