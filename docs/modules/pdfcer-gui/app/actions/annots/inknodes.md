# `app::actions::annots::inknodes` — **the three point verbs of a freehand
mark**, and the one body behind them

Held here rather than in [`super`] so that the annotation-action router
stays inside R2's 1,500-line ceiling: the apply side of
`EditSession::reshape_ink` is two hundred lines of function and argument.

## The seam is a VERB FAMILY, not a line count

[`super::reshape`] is the one body behind the three `/Vertices` verbs
(`move_node`, `insert_node`, `remove_node`) and it reaches
`EditSession::reshape_annotation`. [`reshape_ink`] is the one body behind
the three `/InkList` verbs and it reaches `EditSession::reshape_ink`. They
are twins — same funnel, same `/M` stamp, same disclosure list — and they
are **two functions rather than one with a branch** for the reason the
engine gave when it declined to widen `VertexEdit` with an optional stroke
index: *"a sentence about `/Ink` appearing in code that has nothing to do
with it."* A `match` on the edit's family inside `reshape` would have put
the ink forecast's fields (`stroke`, `stroke_points_before`,
`appearance_was_pdfces`) into a trace line about polygons, and the polygon (old-name-exempt: the engine's own field name)
forecast's (`measure_not_recomputed`) into one about ink.

So the boundary is the same one `canvas::annotnodes::Plan` draws on the
canvas side: `Plan::Vertex` arrives here as `AnnotAction::{MoveNode,
InsertNode, RemoveNode}` and goes to [`super::reshape`]; `Plan::Ink`
arrives as `AnnotAction::{MoveInkPoint, InsertInkPoint, RemoveInkPoint}`
and comes here. Neither side converts an address; the `(stroke, point)`
pair the canvas planned is the pair the engine is handed.

## Why [`apply`] exists, and the one catch-all in it

Destructuring these verbs in the router costs it a vertical five-field
struct pattern each, which is what pushes it against R2's 1,500-line
limit. So [`super::apply_action`] has **one** arm for the three, an
or-pattern binding nothing, and [`apply`] destructures them here. The
`_` arm that leaves is reachable only by a caller that widened the
or-pattern without adding a case here, which the or-pattern's own comment
forbids; it traces rather than panics, for the reason every other "cannot
happen" in an apply arm does — a silent no-op on one release is recoverable
and a crash mid-edit is not. `dispatch::markupnodes` accepts the identical
shape for the identical reason.

## The engine types this module consumes, by name

`pdfcer_core::edit::InkReshape` is what `reshape_ink` returns: its
`forecast` (a `pdfcer_core::edit::InkForecast`, identical to what the
preview answered), its `appearance`, its `dropped` list and
`mod_date_written`. `pdfcer_core::edit::InkEditKind` rides in the forecast's
`edit` field and in `CommandKind::ReshapeInk`'s undo label —
`InkEditKind::PointMoved`, `InkEditKind::PointInserted`,
`InkEditKind::PointRemoved`, `InkEditKind::StrokeReplaced`,
`InkEditKind::StrokeMoved`, `InkEditKind::StrokeRemoved` — of which this
shell raises the three point edits. The trace line prints it through
`InkEditKind::as_str` rather than `{:?}`, so it reads `move-point` exactly
as `pdfcer ink-edit --op move-point` does. `InkEditKind::changes_stroke_count`
is not read: the trace carries `strokes=before->after` outright, which says
the same thing as a number a wrong build would get wrong.

## The operator's report

> *"the draw a line that follows the pointer tool — I can't edit the nodes
> that make it"* (O158, 2026-09-08)

The engine's reply to O158 is the source for
every fact this module states about the verb: that it re-bakes the
appearance, that pdfcer bakes an `/InkList` as a polyline, that `/Rect` is
derived rather than preserved, and that `appearance_was_pdfces == false` (old-name-exempt: the engine's own field name)
is *"the one disclosure you should not drop"*.

## Item notes

### `fn apply`

Called from [`super::apply_action`]'s single or-pattern arm over
`MoveInkPoint | InsertInkPoint | RemoveInkPoint`, and from nowhere else. The
module header carries why the routing is split across two files and what
the `_` arm means.
