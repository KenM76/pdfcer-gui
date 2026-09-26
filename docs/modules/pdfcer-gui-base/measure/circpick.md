# `pdfcer-gui/canvas/measure/circpick`

## Item notes

### `fn a_degenerate_removal_radius_never_removes`

The degenerate-mapping case, and the direction of the failure is the
whole point: a tool that stopped accepting points would look broken, and
a tool that accepted them and removed something else would be worse —
the operator would watch their set shrink as they clicked.

### `fn a_picks_origin_survives_into_the_set`

The disclosure `OPERATOR_REQUESTS.md` O106 rests on: five free positions
and five snapped nodes produce the same numbers, and only one of the two
is the drawing's own geometry. The canvas does not distinguish them —
rule 4 forbids marking applied content — so this value is the only thing
the Tool panel has to tell the operator with.

### `enum PickOrigin`

The circular tool takes two kinds of pick and they are not equally good
evidence, so the difference is carried rather than flattened:

* a **snapped** point is on geometry the document states, and the operator
  can rely on it being exactly where the drawing says the edge is;
* a **free** point is the operator's own judgement of where an edge is,
  which is the only thing available on a scanned or raster drawing.

Both are legitimate and neither is marked on the canvas as provisional —
rule 4 forbids that, and applied content renders exactly as saved content
will. The disclosure lives **off-canvas**, in the Tool panel's list, where
the operator can see that three of their five points were guesses and decide
whether the residual they are looking at is good enough. That is the whole
distinction between *fuzzy* and *sneaky*.

### `struct CircPoint`

Page space (PDF default user space), which is the frame
[`fit_circle_taubin`] consumes and the frame `pdfcer dimension-add
--points` takes — so a set assembled here and a set assembled on the command
line produce the identical circle.

### `struct CircularPick`

Deliberately not `canvas_selection`: a circle-fit attempt has no meaning as
the substrate's general object selection (ui-spec §3.1). The fit re-runs
live on every change; Finish authors a [`DimensionKind::Circular`].


`OPERATOR_REQUESTS.md` O105, in the operator's words:

> *"selecting a point sometimes makes a big circle, and selecting more
> points around a hole doesn't always get it to narrow down to the size of
> the hole."*

A click hit-tested for a **PDF path object** and contributed *every anchor
of every subpath of that object* to the fit. On his own drawing
(`SW41177.pdf`, page 1, measured with `pdfcer object-list`) three objects
carry **4,405**, **4,972** and **6,681** anchors, the largest spanning
550 × 500 pt — half the sheet. One click anywhere on that object handed
Taubin's fit six thousand points scattered across the drawing, and the
best-fit circle through them is enormous. That is the "big circle", exactly,
and it is not intermittent: it depends on whether the hole's arc happens to
be its own small object or one of 1,194 subpaths inside a large one, which
the operator cannot see and has no reason to think about.

The second half followed from the same design. A second click on the same
object **toggled it out**, so clicking twice around one hole added it and
then removed it; and a click on a different object added another thousand-
anchor blob, making the fit worse. *"Add more points"* meant the opposite of
what it means everywhere else.

⇒ **A click is now one point.** Three points on an arc give the arc, which
is the model every drafting package uses and the model the operator was
already working in — his own sentence says *"selecting more points around a
hole"*.

What is lost is *"click the circle once and be done"*. That is worth
having back at **subpath** granularity, because a subpath is the drawn
entity and an object is not; it is recorded on O105 as a decision rather
than an omission, and is not built here.

### `fn toggle_point`

A click within `tolerance` of a point already in the set **removes that
point**; otherwise the point is appended. Returns `true` when the set
grew, `false` when it shrank.

# Why proximity rather than equality

`OPERATOR_REQUESTS.md` O107: *"we should be able to unselect
points/clicked locations"*. An operator taking a point back out aims at
the marker they can see, and the pointer never lands on the exact `f64`
they committed — a free position is a screen pixel converted through the
zoom, so equality would make the canvas route unusable and leave only
the panel.

The radius is the **snap catch radius** (`PageMapping::snap_tolerance`),
which is the right one and not merely a convenient one: inside it, a
snapped click would have landed on the very point being removed, so the
two readings of the gesture cannot disagree about which point is meant.

A non-finite or negative tolerance removes nothing, which makes a
degenerate mapping add points rather than silently eat them.

### `fn remove`

Returns the point that went, or `None` for an out-of-range index. The
index is the row's position in [`Self::points`], and the two cannot
drift because the panel draws from that slice in the same frame it
raises the removal.

Out of range is `None` rather than a panic: a panel row is drawn from
last frame's state and acted on in this one, so a removal racing a
canvas click is an ordinary event and not a bug to crash on.

### `fn samples`

The same point set `pdfcer dimension-add --points` would pass, so the
fit (and thus the authored kind) is byte-identical to the CLI's for the
same picks.
