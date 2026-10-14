# `canvas::annotgroup`

Dragging several selected markups, links and ce dimensions together. With
one annotation selected the single-target routes (`canvas::dimdrag`,
`canvas::annotdrag`) answer as before; `annotgroup::drag` returns `None`
for them.

## Where a group drag starts

`canvas::pressing` sets `markup_body` when the press lands inside any
**unlocked** member's `/Rect` outline (`annotgroup::body_at`), not only the
primary's. So a group can be picked up by any member, including when the
primary is locked or is a ce dimension. The primary's grips, node anchors and
ce-dimension vertex, extension and label grips keep their precedence over the
body: they act on the primary alone.

`canvas::dragroute` asks `annotgroup::drag` first, before the dimension and
markup branches.

## Ghosts

One rectangle per unlocked member, its outline translated by the drag delta,
drawn with `overlay::draw_annot_ghost`. A locked member gets none because it
will not move. A ce dimension's ghost is its box translated, while the commit
places it by `dimdrag::placed`, which keeps it on what it measures, so its box
can land short of its ghost.

## Commit

The release raises one `VectorAction::MoveMixed { gesture: "drag" }` with
every member and one page-space delta; `app::actions::movemixed` applies it
and folds it into one undo step. Locked members are sent as well so the
engine refuses each by name and the status line says how many moved
(`annots_partly_moved`). Batch verb owed: G190.

## Trace

`move-mixed-applied gesture=drag page objects widgets annots=moved/asked
one_step moves`.
