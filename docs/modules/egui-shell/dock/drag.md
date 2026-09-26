# `egui-shell/dock/drag`

## Item notes

### `struct TabDragPreview`

Published on [`super::DockFrameReport`] so an application can say it in
words and a driven check can assert on it: a hairline between two
near-identical tab labels is precise and not checkable.

### `fn lands`

False at the two boundaries against the dragged tab's own edges: they
are legal drops and they permute nothing. Derived rather than published
as a field of its own, because it is a function of two fields already on
this struct, and a stored copy is a second answer that can disagree.

### `fn preview`

Called at the end of a tab bar's layout, after every tab it drew has been
recorded in [`Ctx::geometry`] — the caret's position is read from those
rects rather than computed from a width, for the rule
[`super::geometry::DockGeometry::push_tab`] carries.

Cross-compartment drops are not this function. A drag over a *different*
strip belongs to [`super::overlay`], which resolves and draws it once at
the end of the frame, when every compartment is in [`Ctx::geometry`]. This
one stands down there, so that one gesture never has two carets proposing
two outcomes.

## The band, and why it is not the strip's own rectangle

A reorder is resolved by the pointer's **x alone**, so the pointer's y
decides only whether it is a reorder at all. Bounding it by the strip
exactly would make the caret flicker out under the few points of vertical
wander any horizontal drag has; not bounding it at all would make a drag
pulled down into the document silently reorder the strip it left, which is
what every application of this class treats as a tear-out instead.
[`REORDER_SLACK_PTS`] is the tolerance between those, and the branch this
function declines is where tearing a panel out will attach.

## And bounded in x by the strip exactly, with no slack at all

[`super::geometry::DockGeometry::gap_in`] is defined for **every** x on the
screen: past the last tab it answers the boundary past the last tab, from
anywhere. Every other tab strip in the dock sits at the same y as this one,
so the y band alone would let a drag carried sideways onto a *different*
strip be claimed as a reorder of the strip it left — a caret drawn where the
pointer is not, and the drop the operator was aiming at never offered.

x gets no tolerance because it is the axis the boundary is *resolved* from,
not the one wander happens along, and because the slack would land on a
neighbour: two columns on one side put their strips a splitter apart. The
strip spans its compartment's whole width, so a drag past the last tab is
inside it already.

Neither bound can be stated as *"over a different compartment"*, which is
what it means — [`Ctx::geometry`] is filled in draw order, and this runs
mid-draw, so the compartments after this one are not in it yet.

### `fn caret_rect`

Shared by the reorder preview and by [`super::overlay`], which draws the
same marker when a drag is carried over a *different* stack's strip. One
definition, because two spellings of "where is boundary `gap`" disagree by
whatever either of them gets wrong.

### `fn in_flight`

[`super::overlay`] needs the panel to replay a candidate drop against the
layout. Reached through this rather than through the memory key directly,
because where a drag is kept is this module's business — see the header.

### `fn settle`

Called once, after all three affordances have had their say. Clearing the
memory is unconditional on release; raising an intent is not, because a drag
that none of the three claimed — its strip was not drawn, the pointer is on
a seam inside the dock — has nothing to name and must land nowhere rather
than land at a guess.
