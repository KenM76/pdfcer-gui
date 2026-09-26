# `pdfcer-gui/canvas/dimdrag/tests`

## Item notes

### `fn a_perimeter_label_takes_the_delta_in_both_axes`

The property that separates it from a linear dimension, and the one the
engine went out of its way to point out: a perimeter's placement is in
PAGE axes, so a diagonal drag is expressible. A linear dimension's
diagonal drag is flattened onto its own axis, which is correct there and
would be wrong here.

### `fn authored`

**Through the engine, not through a hand-built model**, and that is what
makes every assertion below mean something. `count_edit` asks
`EditSession::vertex_edit_preview`, which reads the sidecar record the
session holds — so a test that faked the record would be asking the engine
about a ce dimension that does not exist, and would get
`DimensionNotFound` for every case while looking exactly like a test that
passed for the right reason.

### `fn a_closed_triangle_refuses_to_lose_a_corner_and_says_why`

A closed ring may not go below three corners — two closed vertices trace a
line there and back and print twice the distance between two points — so
the release must raise **no** `RemoveVertex`, and must instead say so.

Both halves are asserted deliberately. A build that simply dropped the
gesture would pass the first and fail the second, and it is the second that
is the operator's actual complaint: a corner drag that is refused with
nothing said anywhere is the founding defect of this project.

### `fn a_refused_removal_previews_the_shape_that_is_already_there`

The frame before the release draws the shape it would commit, and here the
shape it would commit is the shape that is already there. A build that drew
the corner vanishing and then refused would be showing an edit that never
happens — which looks like it worked until the next repaint.

### `fn an_open_three_point_path_may_lose_a_corner_and_becomes_a_line`

An open path keeps two, so this removal is legal and what it leaves is a
straight line: one segment, from the first corner to the last. That is the
case the lead asked to be asserted rather than assumed — *"use one where
removing a vertex would leave a line, and assert what happens"* — and it is
what proves the closed test above measures the ring rule rather than a
blanket refusal on three-cornered shapes.

### `fn a_corner_is_added_after_the_one_that_was_grabbed`

Asserted on the geometry rather than on the action's `after` field alone,
because a build that raised `after: 1` and previewed the point at index 0
would pass an argument check and show the operator the wrong thing.

### `fn the_closing_segment_can_take_a_corner_too`

The engine went out of its way to make that meaningful and a shell that
clamped it would silently put the corner on the wrong side of the shape. A
`Vec::insert` at `len` appends, which is the same point.

### `fn ctrl_alone_cannot_change_how_many_corners_a_shape_has`

Ctrl already means *take this out of the selection* everywhere else on this
canvas (`OPERATOR_REQUESTS.md` O104, `canvas::marquee::Combine`), so an
operator has every reason to be holding it during an ordinary corner drag.
If that could delete a corner, the feature would be a trap. The Points tool
is what makes it deliberate.

### `fn the_points_tool_survives_review_and_still_retires_in_read`

The three rows are the whole of the Node arm's rule:

| mode | `edit_content` | `author_measure` | the tool |
|---|---|---|---|
| Edit | yes | yes | armed — page anchors AND ce-dimension corners |
| Review | no | **yes** | armed — corners only |
| Read | no | no | retired |
