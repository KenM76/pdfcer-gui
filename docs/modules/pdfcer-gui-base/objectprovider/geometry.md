# `objectprovider::geometry` — **the same questions, asked of either index space**

`OPERATOR_REQUESTS.md` **O70**, 2026-09-01. Everything here answers *"what
is inside this thing?"* for a [`TargetId`] rather than for a page
paint-order index, which is what lets the Part and Node rungs be offered for
something painted inside a form XObject.

## Why a module rather than more methods in [`super`]

R2's gate, and it points at a real seam. `super` answers *"what is on this
page and where is it?"* — the decomposition, the hit tests, the canvas
projection, the panel's tree. This answers the narrower question of what one
already-identified object is made of, and it does so for **both** address
spaces, which is a distinction none of the rest of that file has to make.

## Why these are additions rather than changed signatures

The index-based methods in `super` have thirty-odd call sites and every one
is correct: `canvas::input`, `canvas::painting`, `canvas::shapes` and the
Objects panel all hold page indices for good reasons. Changing them in one
edit is a diff nobody can review against a hot path this project has already
broken twice by second-guessing.

⇒ So this is the general form, the index form delegates to it, and call
sites move over one at a time behind their own tests. The duplication is a
`TargetId::Object(i)` wrapper, not a second implementation — the distinction
`canvas::mapping`'s header draws between a shared helper and a second
opinion.

## The engine already published the leaf-friendly forms

`hit_test_subpaths_of(&PathObject, …)` takes the object rather than a model
and an index, and `PathObject::page_subpaths` is geometry in page space
whoever holds it. So none of this needed a request — the shell had been
asking the narrower question because the narrower question was all it had
ever needed.

Proven against a real file rather than a stub, in
`crates/pdfcer-gui/tests/leaf_geometry.rs`: a stub carries rectangles and
cannot model where a path's anchors are, so a unit test against one would
assert that the plumbing returns what the stub was given.

## Item notes

### `fn object_for`

`None` for an index the page does not have, which is the contract
every accessor here inherits: a selection can outlive an edit that
removed what it named, and the honest answer is to drop the entry
rather than to panic on the frame that is trying to draw it.

### `fn text_run_move_refusal_of`

# The engine's own guard, called rather than copied


**It does not promise success.** A singular `Tm` or CTM is
discovered during planning, from geometry, not from the run's
structure, so `None` here means *"the move will be planned"* and not
*"the move will land"*. The residual failure arrives as an ordinary
engine refusal through the edit funnel, which is the right place for a
condition nothing could have known in advance.

# Returns

`None` when the move would be planned — **and also** for a target that
is not a text object at all, because there is no run-move to refuse.
The caller has already established the kind through
[`Self::part_kind_of`]; this is not the function that decides whether a
run is what was selected.

### `fn subpath_hits_of`

Through `hit_test_subpaths_of` rather than `hit_test_subpaths`: the
first takes the path, the second takes the model and an index into its
page list. The engine published both, and the object-taking form is the
one a leaf can use — the geometry is identical, only the addressing
differs.

### `fn subpath_node_points_of`

The indices are **object-scoped** — the space `vector::anchor_count`
reports and the node verbs take — and they are computed by the same
running-offset walk the page-object form uses, because the two must
agree about what anchor 7 is or a drag would move a different point
from the one drawn.

### `fn nearest_node_of`

Ties resolve to the lower index, exactly as the page-object form does —
the rule is restated in one place by delegating the point list, so the
two cannot answer differently for the same geometry.

### `fn node_handles_of`

The Bézier control points of one anchor: the incoming one is the second
control of the segment **before** it, the outgoing one the first control
of the segment **after**. Only cubics have them, so a polyline answers
empty and no handle is drawn — which is the honest answer rather than a
grab target for a gesture with nothing to move.

The object-scoped anchor index is brought back into the subpath's own
space with the SAME running offset [`Self::subpath_node_points_of`]
computes. Two walks that disagreed about which subpath anchor 7 falls in
would draw a handle on one curve and move another.

### `fn subpath_bounds_canvas_of`

Computed from the subpath's own anchors rather than through
`vector::subpath_bounds`, which takes a model and a page index. Anchors
alone under-report a curve whose control points bow outside them — the
same approximation the page-object form inherits from the engine on a
Bézier, and the box is a selection outline rather than a measurement.
`None` for an empty subpath, which has no box to draw.
