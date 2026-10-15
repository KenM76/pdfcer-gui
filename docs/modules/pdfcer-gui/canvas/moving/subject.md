# `canvas::moving::subject`

Which verb a move reaches (`eligible`) and the one `Action` a completed move
becomes (`action`). Both are pure: `eligible` reads the selection and a
`MoveContext`, `action` a `MoveSubject` and a page-space delta.

## A mixed move

At the Object rung, parts of placed drawings selected beside page objects
(`MoveContext::beside`) make the subject `MoveSubject::Mixed`, which `action`
turns into one `VectorAction::MoveMixed` with gesture `drag`:
`app::actions::movemixed` moves the page objects with `move_objects_each`,
each drawing's parts with `move_objects_each_in_form`, and folds them into
one undo step. A part inside a selected drawing is not in `beside`
(`panels::align::leftout::uncarried`): moving the drawing moves it, so it
must not move twice. With no part beside them, page objects keep the single
`move_objects` command (or `transform_objects` when one has no coordinate
operands).

## Item notes

### `fn entered_entry`

Refuses an entry that belongs to a different page rather than addressing
page A's index space with page B's number — the same class of error the
[`TargetId`](crate::canvas::target::TargetId) newtype exists to prevent,
and one comparison to rule out.

### `fn eligible`

Consulted **twice per drag**: once per frame while the drag is in flight,
to decide whether a ghost may be drawn at all, and once on release, to
build the command. Asking the same question both times is the mechanism
behind obligation 3 in the module docs — a ghost is drawn if and only if
the release would commit, so the preview cannot promise a move the engine
is going to refuse.

Deliberately says nothing about the *distance* dragged: a zero-travel drag
is eligible (it names a real verb on real operands), it simply has nothing
to commit, and that is [`action`]'s call. Splitting it this way is what
keeps the ghost visible during the frames where the pointer happens to pass
back over the press point.

### `fn action`

`node_at` is the entered anchor's **current** page-space position, and is
consulted only by [`MoveSubject::Node`]. It is needed because `move_node`
takes an absolute destination rather than a displacement — the operand it
rewrites is a coordinate pair, and expressing the drag as "where the point
ends up" is what lets the planner map one point through the object's CTM
inverse instead of decomposing a translation into a space it would have to
re-derive.
