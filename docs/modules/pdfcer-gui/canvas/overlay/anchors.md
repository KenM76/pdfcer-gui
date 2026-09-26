# `pdfcer-gui/canvas/overlay/anchors`

## Item notes

### `const ANCHOR_PX`

An anchor is a **target**, not merely a statement: the Node tool makes
a single click on an anchor the way an operator picks one, so it must be as
grabbable as a grip and no smaller than its own Bézier control point —
`OPERATOR_REQUESTS.md` O69: *"the nodes are hard to see and click on."*

Seven, and both halves are borrowed rather than invented: it is
[`HANDLE_PX`], so an anchor is never smaller than the handle hanging off
it, and it is Inkscape's node size, which is the operator's stated
tie-breaker for this whole family of decisions. It stays visibly under the
8 px resize grip, so the three marks are still three sizes.

The upper bound is legibility: a run of anchors along a polyline must read
as a row of dots rather than as a second outline. Seven does; twelve would
not.

**It has a second consumer** — `canvas::pressing::grabbable` inflates the
inner-rung move box by this amount, so that an anchor sitting on the
object's bounding edge (half outside it) is still draggable. Widening the
mark widens that box by the same pixel, which is the right direction and is
named here so the coupling is not rediscovered.

### `const MAX_UNSELECTED_ANCHORS`

A real number from a real document rather than a round one: `canvas::moving`
records **6,681 anchors on one measured CAD export**, and a single object on
this operator's drawings routinely carries thousands. Painting all of them
would put several thousand filled rects in the frame for a rung the operator
entered in order to move *one* point, and the canvas would visibly stutter
at the exact moment they are doing precision work.

Above the cap the **selected** anchors still draw — they always draw, at any
count, because they are the answer to "what did I pick?" and that question
has no other surface — and the fact that the rest were suppressed is
disclosed off-canvas. That is rule 4's half that survives: an operator who
cannot see the unselected anchors would otherwise conclude the object has
none.

### `fn draw_anchors`

# Why it exists

The Node rung is enterable and multi-node selection is representable, so an
operator can descend two rungs and Shift-click four anchors. Without a mark
there is **no surface whatever** that says which four. A set the operator
cannot see is a set they cannot choose deliberately, and a move of an
invisible set is indistinguishable from a bug — which is why the marks and
the multi-node move are one feature rather than two.

# Why selected and unselected are drawn differently, and how

Filled for selected, hollow for not. The same language as the resize grips
one rung up — filled square, selection-coloured stroke — so the visual
vocabulary of "a thing you can grab" is one vocabulary across the ladder,
and a reader who has learned the grips has learned these.

# This is the CURSOR, not content

Rule 4 forbids styling *applied content* to express pdfcer's own uncertainty
and explicitly welcomes *pre-commit affordances*: "snap indicators, hover
highlights, rubber-bands and selection handles are the cursor". An anchor
mark is a selection handle. It describes where the operator may act, changes
nothing about how the page renders, and disappears the moment the rung is
left — so the one-line test holds: a screenshot of this canvas and a
screenshot of the same document saved and reopened differ only in the
cursor.

# Coordinates

`points` are in **canvas space**, already converted by the caller.

The conversion is the caller's because `PageMapping` speaks canvas ⟷
screen and knows nothing about PDF user space — turning a `vector::Point`
into a canvas position needs the `Page`'s own box and rotation, which is
`viewer::pdf_space_to_canvas`' job. Passing the `Page` in here so that this
function could do it would give the overlay a second coordinate authority,
and `coords`' standing rule is that a coordinate is produced by exactly one
conversion in exactly one place.

### `const HANDLE_PX`

Slightly larger than an anchor mark and **round** where anchors are
square, which is the vector-editor idiom every tool this operator has used
shares — Illustrator, Inkscape, Figma and the old shell all draw an on-curve
point as a square and a control point as a circle. It is not decoration: the
two are different kinds of thing and the shape is what says so at a glance,
with no legend and no hover.

### `fn draw_handles`

# Why the tether is not optional

A control point with no line back to the anchor it governs is an unexplained
dot floating beside a curve — and on a path with two selected anchors, four
such dots are ambiguous about which belongs to which. The tether is the only
thing that says *this handle steers that point*, and every editor draws it
for that reason.

It is drawn **thin and in the selection colour**, not dashed: a dashed line
on a CAD drawing competes with the drawing's own dashed linework, which is
the class of collision `DEFECTS.md` D12 records for the guide overlay.

# This is the CURSOR, not content

Rule 4's welcome list — "snap indicators, hover highlights, rubber-bands and
selection handles are the cursor". These vanish when the rung is left and
change nothing about how the page renders, so a screenshot of this canvas
and one of the same document saved and reopened differ only in the cursor.

### `const PUBLISHED_ANCHORS`

Six: enough for a driven check to aim at one and find a neighbour, few
enough that a subpath with two hundred anchors does not put two hundred
lines in the trace every time the layout moves.

### `fn anchor_region`

A fixed set of `&'static str`s rather than a `format!`, because
`crate::diag::ui_rect` takes a `&'static str` by design — a region name is
part of the application's published vocabulary, not a runtime string, and
the harness's `driving::declared` matches on it exactly.
