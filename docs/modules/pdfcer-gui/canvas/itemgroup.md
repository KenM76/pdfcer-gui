# `canvas::itemgroup`

Dragging several selected form fields, or several selected markups, links and
ce dimensions, together. A field selection and an annotation selection exclude
each other, so a group is one or the other. With one item selected the
single-target routes (`canvas::widgetdrag`, `canvas::dimdrag`,
`canvas::annotdrag`) answer instead: `Group::fields` and `Group::annots`
return `None`.

Boxes come from `canvas::itembounds`, the same reading the Align panel
measures, so what Align lines up and what a drag picks up are one set of
boxes.

## Members

- **Annotations:** every selected annotation (`SelectionState::annots`), when
  there are two or more and at least one is unlocked. `toggle_annot` keeps the
  selection to one page.
- **Fields:** the primary (`OpenDoc::selected_field`) and every box in
  `OpenDoc::field_set` that lies on the primary's page and is placed this frame
  (`canvas::forms::placed`), when two or more fields are selected. Selected
  boxes on other pages are left where they are: the operator cannot see them
  follow, so moving them would be a sneaky edit.

## Where a group drag starts

`canvas::pressing` sets `markup_body` or `widget_body` when the press lands
inside any member that will move (`Group::body_at`), not only the primary. A
locked annotation cannot pick the group up. The primary's grips, node anchors
and ce-dimension grips keep their precedence over the body: they act on the
primary alone. `canvas::dragroute` asks `Group::annots` before its dimension
and markup branches, and `Group::fields` first in its form-field arm.

## Ghosts

One rectangle per member that will move, its box translated by the drag delta,
in `Previews::group`. A locked member gets none. A ce dimension's ghost is its
box translated, while the commit places it by `dimdrag::placed`, which keeps it
on what it measures, so its box can land short of its ghost.

## Commit

The release raises one `VectorAction::MoveMixed { gesture: "drag" }` with every
member and one page-space delta; `app::actions::movemixed` applies it and folds
it into one undo step. Locked annotations are sent as well so the engine
refuses each by name and the status line says how many moved. The delta is
converted on the page the caller passes: the current page for annotations, the
group's page for fields. Batch verbs owed: G189 (widgets), G190 (annotations).

## Trace

`move-mixed-applied gesture=drag page objects widgets annots=moved/asked
one_step moves`.
