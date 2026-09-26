# `canvas::annotnodes` tests — the shell's half, and the engine's ruling

## What these can and cannot prove, stated first

**They cannot prove the operator can edit a node.** Every test here calls a
function directly, and the whole point of R1 is that a passing unit test is
not a report of working software: this project once had eight green tests
while the feature performed 1 of 14 steps. Six process boundaries stand
between [`super::resolved`] and a hand on a mouse — the tool arming, the
chord reaching dispatch, the modifier surviving winit, the press
classifying, the anchor being painted where the hit test looks, and the
engine accepting the write — and **not one of them is observable in
process**. `tools/ui-verify`'s `a_markup_shapes_nodes_can_be_edited` is the
instrument for that, and its own header enumerates the six.

What these DO prove, and what makes them worth the second they cost:

1. **The shell's subtype table is right** — which shapes show anchors. That
   is a painting decision this shell owns, so nothing else can check it.
2. **The engine's ruling is asked rather than restated.** Every count test
   below authors a real annotation into a real fixture and drives the real
   `reshape_annotation_preview`. A test that faked the annotation would get
   `AnnotationNotFound` for every case *while looking exactly like a test
   that passed for the right reason* — `dimdrag::tests` records that trap in
   the same words.
3. **A refusal is a sentence.** Half of each refusal test asserts the
   decline was raised, because a build that merely dropped the gesture would
   pass the other half and would be the operator's original complaint.

## The shapes are chosen so the boundary is exercised

A test on a five-sided polygon proves nothing about the floor. The shapes
here are:

| shape | nodes | floor | what removing one does |
|---|---|---|---|
| `/Polygon` triangle | 3 | 3 | **refuses** — at the floor |
| `/Polygon` square | 4 | 3 | succeeds, leaving a triangle |
| `/PolyLine`, three points | 3 | 2 | succeeds, leaving a straight line |
| `/Line` | 2 | — | **refuses by name** — a Line is two ends by definition |
| `/Ink`, strokes of 3 and 2 points | 5 anchors | 2 **per stroke** | succeeds on the first stroke, **refuses** on the second — the same mark, two answers |

The triangle and the three-point polyline are the same three points and give
**opposite** answers, which is what proves the refusal measures the shape's
own floor rather than blanket-refusing three-node shapes. The two-stroke ink
(`Pass 278.0`) proves the same thing one level down: the floor is the
**stroke's**, not the mark's, and a flat anchor index that was converted to
the wrong stroke would get the opposite answer from the engine.

## The ink tests sit at the stroke boundary, on purpose

Every ink assertion below is made at **index 2 or 3** of a five-anchor mark
whose first stroke holds three points — the last point of stroke 0 and the
first point of stroke 1. That is the one place a flat-list implementation
goes wrong in a way that looks right: an off-by-one in the stroke table
addresses the neighbouring stroke, a naive preview draws a segment across
the gap, and a naive insert-after puts the new point into stroke 1. A test
on the middle of a long stroke would pass all three defects.

## Item notes

### `fn authored`

**Through `add_markup`, not through a hand-built dictionary.** The
preflight reads the annotation out of the session's own graph, so a fixture
that faked one would be asking the engine about a shape that does not exist.

### `fn the_three_shapes_with_nodes_report_their_geometry`

The `closed` flag is what decides whether the preview draws the segment back
to the first node, so getting it backwards would draw an open triangle over
a closed one — a picture that is wrong in a way an operator would report as
*"it looks like it lost a side"* and that no count assertion would catch.

### `fn a_shape_with_no_editable_nodes_shows_no_anchors`

`/Ink` was the third row of this list until `pdfcer-core` `Pass 278.0`
(2026-09-09), and the paragraph that kept it there is still true of the
rule if not of the shape: `Annotation::ink_list` was **readable** while
every edit on it was refused, so a shell that derived *"draggable"* from
*"readable"* would have drawn anchors that refused every drag. The verbs
exist now, so the anchors do; the ink assertions live in section 5 below.

### `fn a_locked_shape_offers_no_anchors`

§12.5.3 Table 165 bit 8 is the *file* saying the user interface may not
change this. An anchor drawn on it would be a promise the release cannot
keep — `annotdrag::eligible` states the identical rule for the whole-shape
drag, and this is that rule applied one level down.

### `fn a_ce_dimension_is_dimdrags_and_not_this_modules`

The load-bearing half. A ce dimension is a `/Line` with
`/IT /LineDimension`, so it passes every *"is this markup?"* test; reshaping
one through `reshape_annotation` would move the drawn line and leave the
sidecar record — and therefore the printed number — describing geometry that
is no longer there. The engine refuses it by name as the backstop; this is
what stops the backstop being reached.

### `fn a_closed_shape_previews_its_closing_segment`

Four nodes closed is four segments; four nodes open is three. Getting this
wrong draws a shape the release does not commit, which is the one thing the
honesty contract in this module's header forbids.

### `fn a_single_list_shapes_segments_are_preview_of_unchanged`

`Geometry::segments` replaced the direct `preview_of` call when `/Ink`
needed a segment list that respects stroke boundaries, and this is the
assertion that the polygon and polyline paths came through that change
**byte-identical**: same pairs, same order, same closing rule. A closed
square and an open one, because the closing segment is the only place the
two derivations could have disagreed.

Falsified: dropping the `closed && n >= 3` push in `segment_pairs` makes
the closed comparison fail on length.

### `fn moving_a_node_moves_that_node_and_no_other`

Asserted on the resulting geometry rather than on the action's `dx`/`dy`
alone, because a build that raised the right delta and previewed the wrong
node would pass an argument check and show the operator the wrong picture.

### `fn only_the_release_raises_an_action_and_it_raises_one`

`drag-moves` D4: one gesture is one `Ctrl+Z`. A build that raised per frame
would fill the undo stack with sixty entries a second and would look
perfectly correct on screen.

### `fn a_move_sends_the_displacement_of_the_node`

The node at `(300, 100)` dragged to `(150, 150)` is `dx = -150, dy = +50`.
A build that sent the absolute target would move the shape by the target's
distance from the origin — a very large jump, and one that looks like a
coordinate-space bug rather than an arithmetic one.

### `fn a_triangle_refuses_to_lose_a_node_and_says_why`

The floor is the engine's (`/Polygon` keeps three) and this test drives the
real `reshape_annotation_preview` against a real annotation, so it fails the
day the engine's ruling changes — which is the property that makes it worth
more than an assertion about a constant in this crate.

Both halves are asserted deliberately. A build that simply dropped the
gesture would pass the first and fail the second, and **it is the second
that is the operator's actual complaint**: a node drag that is refused with
nothing said anywhere is the founding defect of this project.

### `fn a_refused_removal_previews_the_shape_that_is_already_there`

The frame before the release draws the shape it would commit, and here that
is the shape already on the page. A build that drew the node vanishing and
then refused would be showing an edit that never happens — which looks like
it worked until the next repaint.

### `fn an_open_three_point_path_may_lose_a_node_and_becomes_a_line`

A `/PolyLine` keeps two, so this removal is legal and what it leaves is a
straight line: one segment, from the first node to the last. This is what
proves the test above measures the shape's own floor rather than a blanket
refusal on three-node shapes.

### `fn a_node_is_added_after_the_one_that_was_grabbed`

Asserted on the geometry rather than on the action's `after` field alone,
because a build that raised `after: 1` and previewed the point at index 0
would pass an argument check and show the operator the wrong thing.

### `fn a_line_moves_its_ends_and_refuses_to_gain_one_by_name`

This is the brief's own case: *a refusal that names a shape type is a real
refusal and should be shown as a sentence, not a grey anchor.* The anchors
are drawn (a Line's two ends are draggable), the count edit is refused, and
what the operator gets is a sentence about lines rather than silence.

### `fn no_node_refusal_calls_a_markup_shape_a_measurement`

R8b rule 15. `crate::text::measure::VertexEditRefusal` says *"measurement"*
because its subject is a **ce dimension**; this enum's subject is a comment
somebody drew, and reusing those words would tell an operator their polygon
was measuring something. This is the assertion that stops a future tidy-up
merging the two enums.

### `fn a_square_is_called_a_rectangle_and_a_circle_an_ellipse`

`/Square` is what pdfcer's own Rectangle tool authors, and telling an
operator "Square" for the thing they drew with the Rectangle button is the
surface disagreeing with itself about what it just did.

### `fn the_points_tool_arms_wherever_a_markup_shape_can_be_authored`

The count gestures require the **Points tool** to be armed — `Ctrl` alone
with the Select tool still moves the node, which is the safety that stops a
mis-held modifier destroying a node during an ordinary nudge. That tool's
arming predicate is `edit_content || author_measure`
(`canvas::tool::arm::retire_forbidden`, and an identical copy in
`app::dispatch::navigate`), and **it does not name `author_markup`**.

Markup is authored in **Review**, so if Review ever lost `author_measure`
the Points tool would stop arming there and adding or removing a node of a
comment shape would become unreachable — silently, with the anchors still
drawn and still draggable for a plain move. This test is the tripwire for
that, and it is here rather than in a paragraph because a paragraph cannot
go red.

Why the predicate was not simply widened: the two copies of it must stay
identical, and a disagreement shows as a tool that arms and is retired on
the next frame — a flicker with no sentence attached. One of the two copies
lives in `app::dispatch`, which this session did not own. Reported rather
than half-changed.

### `fn an_ink_with_two_strokes_yields_every_point_and_draws_no_bridge`

Five points, three segments: `(0,1)`, `(1,2)` in the first stroke and
`(3,4)` in the second. A naive flat preview would draw four — the fourth
being a bridge from `(160,150)` to `(300,300)` that the file does not hold
and the release would not commit, which is exactly what the honesty
contract in the module header forbids.

Falsified: replacing `segment_pairs` with the single-list rule makes the
count assertion read 4 and the bridge assertion find the segment.

### `fn an_ink_anchor_index_round_trips_through_stroke_and_point`

Index 3 is the one that matters: it is `(1, 0)`, and an off-by-one in
either direction makes it `(0, 3)` — a point stroke 0 does not have, which
the engine would refuse — or `(1, 1)`, the wrong point, which the engine
would accept and move. The second is the dangerous one: it looks like a
working gesture on the wrong node.

Falsified: changing `flat < start + len` to `<=` in `address` reports
index 3 as `(0, 3)`.

### `fn inserting_after_a_strokes_last_point_extends_that_stroke`

Index 2 is stroke 0's last point. After the insert the mark has six
points, stroke 0 has four, stroke 1 still has two, and the preview's
segments are `(0,1) (1,2) (2,3) (4,5)`: the new point at flat index 3
joins the old last point of stroke 0 and does **not** join stroke 1's
first point, now at flat index 4. The release raises
`InsertInkPoint { stroke: 0, after: 2 }`.

Falsified: a stroke table that did not grow stroke 0 (`after_edit`
returning `self.clone()`) shifts the boundary and the preview draws a
segment from the new point into stroke 1.

### `fn moving_an_ink_point_raises_the_engines_address_not_the_flat_index`

Flat index 3 is `(300, 300)`; dragged to `(150, 150)` it is
`dx = -150, dy = -150`. A build that sent `index: 3` to a vertex verb would
be refused by name; a build that converted it to `(0, 3)` would be refused
as out of range; a build that converted it to `(1, 1)` would move the wrong
point and pass every test that only counted anchors.

### `fn a_two_point_stroke_refuses_to_lose_a_point_and_names_the_stroke`

Both halves asserted, as for the triangle: a build that merely dropped the
gesture would pass the first and fail the second, and the second is the
operator's actual complaint. The sentence is `StrokeWouldLeaveTooFew`, not
`WouldLeaveTooFew`, because *"the shape has as few corners as it can have"*
is false of a mark whose other stroke has three.

### `fn every_ink_refusal_is_a_sentence`

Built from the variants directly rather than provoked through the engine,
because three of them cannot be provoked from an anchor this shell drew —
which is the whole point: a refusal that becomes reachable silently must
already have words waiting.

### `fn an_ink_with_nothing_readable_has_no_anchors_and_no_segments`

Exercised on the table rather than on a file, because pdfcer's own verbs
cannot author an `/Ink` without strokes: an empty stroke list flattens to
no anchors and no segments, which is what `geometry` hands the painter.
