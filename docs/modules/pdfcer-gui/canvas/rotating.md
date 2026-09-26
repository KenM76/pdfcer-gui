# `canvas::rotating` — the ninth grip, and the one gesture the eight could
never express

## What this closes

`ui-conventions/handles.md` H2 — *"the standard set is eight resize grips, a
body, and a rotate handle"* — and the operator's own report, which is the
sentence that corpus row quotes:

> *"unfortunately there was no way to reposition, resize, or rotate it on
> the screen. Can I please please please have that too?"*


## Why a rotate is not a resize with different arithmetic

The eight grips answer *"how big"*, which is a **distance**, so a resize is
a delta in two axes and every one of them has an opposite corner that must
not move. A rotation answers *"which way round"*, which is an **angle**, so:

| | resize | rotate |
|---|---|---|
| measured from | the grip's opposite corner | the selection's **centre** |
| what the drag reads | a displacement | the change in bearing between two rays |
| what the pointer's *distance* means | everything | **nothing** |
| modifier | Shift preserves aspect | Shift snaps to 15° |

The third row is the one that decides the module boundary. A rotate drag
must ignore how far the pointer is from the centre entirely — an operator
swinging a long arc for precision is doing exactly what the gesture invites,
and a build that scaled with radius would shrink the object as they did it.

## The handle sits ABOVE the box, on a stem

Which is PowerPoint, Illustrator, Figma, Inkscape, Visio and Konva's
`Transformer`. The offset is what makes it reachable on a selection whose
top edge is crowded with the north grip and whatever is behind it, and the
stem is what says the two belong together — without it the handle reads as
an unrelated dot floating over the page.

**It is drawn as a circle**, not a square. Every square on this canvas
resizes; a shape that resized in one place and rotated in another would be
a private convention the operator has to learn, which is
`handles.md` H2's stated failure mode.


The header above describes a rotation of **page content**, which is what
this module did on the day it was written. `pdfcer-core` `Pass 155.0` and
`Pass 159.0` then shipped rotation for the annotation family and for ce
dimensions, and this module became the decision point for all three:

| selected | verb | operand | refused by |
|---|---|---|---|
| page **content** | `transform_objects` | paint-order indices + a `Matrix` | — |
| a **markup** annotation | `rotate_annotation` | `ObjId` + pivot + degrees | — |
| a **ce dimension** | `rotate_dimension` | `DimensionId` + pivot + degrees | `rotate_annotation`, **by name** |
| a **form field's** box | *none* | — | `rotate_annotation`, **by name** (`/MK /R`, unbuilt) |

**Everything above the branch is shared and must be**: the bearing between
two rays from the box's centre, the 15° snap, the wrap that stops a drag
past 180° spinning a whole turn, the travel threshold and the single
screen→page negation are one gesture whatever is under it. What differs is
one call. `canvas::resizing` cuts the identical seam in the identical place
and a reader who has understood one has understood both.

The three operands are the **same shape** — a fixed point and a scalar —
because the engine chose it that way on this shell's request: *"the same
anchor+factors shape as move and resize, so your grip code needs no third
convention."* So [`commit_annotation`] is a routing decision rather than a
second arithmetic.

## Why the box comes from `pressing::grabbable` and not `overlay::grip_box`

[`Frame::bounds`] is filled by `canvas::interact` from
`crate::canvas::pressing::grabbable`, and that is **the single line the
annotation rotation hangs on**.

`overlay::grip_box` derives its answer from the selection's cached *content*
outlines, which `select_annot` clears — an annotation is not content and has
nothing decomposed to cache. Over a selected markup or dimension it answers
`None`, so `bounds` would be `None`, so this function would return at its
first line and **the entire gesture would be a no-op with nothing said
anywhere.**

⇒ `grabbable` is also the function `pressing::look` hit-tests against and the
function `canvas::painting` paints from. **One box: what the operator can
see, what they can grab, and what turns.** That is rule H7, and it is the
guard against the failure this canvas has produced four times — a working
gesture aimed at the wrong verb, which never looks broken from a chair
because something moves. The most recent instance is recorded in
`canvas::presspick`: `covers()` tested the selection's move box alone, the
rotate handle sits *outside* that box, and a press on it selected the object
underneath, so the rotate became a select-and-move.


On the **first ever driven run** of `rotating_a_markup_turns_it` the rotate
handle was painted, was pressed at the centre of the rect this application
itself declared, and committed nothing, with nothing said anywhere. The
check's own report named [`Frame::bounds`] as the suspect — the section
above, quoted back at it — and **it was wrong**: `canvas::interact` has
passed `pressing::grabbable`'s box since the day this module landed.

The real cause was **fifteen lines further down the same function**: a
`selection.object_indices_on(page_index).is_empty()` guard standing *in
front of* the annotation branch. It counts page **content**, which
`select_annot` clears, so it answered "empty" on every markup and every ce
dimension and returned before the routing decision was reached. [`drag`]'s
own body carries the full account at the line that moved.

⇒ **The sixth instance of this hazard, and the lesson it adds is about where
a guard stands rather than about what it reads.** This one asked a perfectly
correct question — *has the content verb got an operand?* — of a gesture
that had already been routed away from the content verb. Three destinations
share this gesture, so a test written in one destination's vocabulary
belongs **after** the branch that picks the destination, never before it.
`canvas::resizing` already had it in the right place: its `NothingSelected`
test lives inside `resizing::action`, the pure builder for the *content*
verb, which the annotation branch returns before ever calling. Every
remaining caller of `overlay::grip_box` was audited the same day and none of
them was this bug — which is precisely why it survived a header section
written to prevent it.

## No options type, for any of the three, and that is a property of the
operation rather than an omission

A rotation is an **isometry**: every length is preserved, including the
drawn stroke width. So there is no `scale_stroke_width` question here and no
`allow_appearance_distortion` — `resize_annotation` has both and needs them,
because §12.5.5's placement matrix scales artwork *after* stroking. Rotation
composes into the appearance's own `/Matrix` instead, so **a foreign
appearance rotates correctly** where it cannot be resized.

⇒ `pdfcer-core` drew the UI consequence and this module is built on it: *"if
your grip UI offers rotate and resize together, **rotate needs no
confirmation step and no distortion warning.** Resize does."* There is no
Tool-row switch for this gesture and no dialog in front of it.

## Item notes

### `const MIN_TRAVEL_DEGREES`

`drag-moves` D7: a drag that moves nothing is not an edit. A tenth of a
degree over a 200 pt box is a quarter of a pixel at the corner — invisible,
and not worth an undo entry for somebody who thought better of it.

### `fn commit_annotation`

The routing, in one `match` the compiler checks — which is the whole
reason `canvas::selection::annot::AnnotKind` is an enum rather than an
`is_ce_dimension: bool` on the target. Its header states the rule this
function is the newest instance of: *"a bool is a fact a caller may forget
to read, while a variant is one the compiler makes them handle."*

# The two verbs are NOT interchangeable, and the engine refuses to let
them be

`rotate_annotation` returns `AnnotationMoveWrongVerb` for a ce dimension and
points at `rotate_dimension`, with the reason attached: *"a ce dimension's
orientation is part of its measurement, so turning it must re-measure rather
than spin a rectangle."*

A dimension is a `/Line` with `/IT /LineDimension` and a record in the
document's `/PieceInfo` sidecar. Handing one to the annotation verb would
turn its `/Rect` and its baked `/AP` and leave the **sidecar geometry** —
the thing the displayed number is derived from — where it was, so the
dimension would draw at one angle and measure along another.

⇒ `pdfcer-core` refuses that by name and this shell routes rather than
forces. The refusal stays as the backstop; this is what stops it being
reached.

# A widget cannot arrive here at all, and that is the R9 answer

`rotate_annotation` also refuses a **widget** by name — a widget's rotation
is `/MK /R` (§12.5.6.19 Table 189), a quantised 0/90/180/270 *declaration*
the field's appearance generator reads rather than a free-angle transform,
and it is not built.

There is no arm for it because there is no path to one:
`canvas::selection::annot` excludes `/Widget` from annotation selection
outright (the form surface owns those presses), so a widget is a
`doc.selected_field` rather than a `selection.annot()`, and
`pressing::grabbable` hands that selection `GripSet::scale_only()` — **no
rotate handle is painted and none is hit-tested.** R9: render nothing rather
than draw a handle that refuses.

### `fn a_clockwise_quarter_turn_on_screen_is_positive`

The base case, and the one whose sign is easy to get backwards: screen y
is DOWN, so a pointer moving from due-east to due-south of the centre has
gone clockwise, and `atan2` in a y-down frame calls that positive. The
caller negates once when it crosses into page space; getting the sign
wrong here would rotate the object the other way and look like a
perfectly deliberate feature.

### `fn the_radius_does_not_matter`

The property that separates this gesture from a resize, asserted rather
than assumed: an operator swinging a long arc for precision is doing what
the gesture invites, and a build that let the radius in would shrink or
grow the object while they did it.

### `fn crossing_the_far_ray_does_not_spin_a_whole_turn`

Without the normalisation a pointer moving smoothly through the ray
behind the centre makes the object jump a full turn in one frame. It
looks like a physics bug and it is an arithmetic one.

### `fn a_slow_drag_still_reaches_the_step`

Accumulating snapped increments lets a slow drag through 90° arrive at
87°, because each frame's small delta rounds to zero. Asserted as the
property rather than by simulating frames: `snap` is called on the whole
angle and there is nowhere for an increment to be rounded.

### `fn the_ghost_map_agrees_with_the_measured_angle`

`rotate_about` is the one duplication in this module — the ghost is drawn
from it and the commit from `Matrix::rotate`. This pins the half that can
be checked without a document: a point rotated by the angle measured
between two rays lands on the second ray.

### `const STEP_DEGREES`

Fifteen, which is PowerPoint's, Illustrator's, Inkscape's and Figma's. It
divides 90 and 360 exactly, so the four right angles and the four diagonals
are all reachable — which is what the operator actually wants from the key,
and what a value like 10° would give them for 90 and take away for 45.

### `fn angle`

Positive is the direction the pointer went. Both rays are measured from
`centre`, and the pointer's *distance* from it is discarded — see the module
header for why that is the whole shape of the gesture rather than a detail.

# Screen space in, screen space out, and the sign survives the hop

`centre`, `from` and `at` are all screen positions, where y runs **down**.
`atan2` therefore answers a bearing in a left-handed frame, so a clockwise
drag comes back positive. PDF user space is y-**up**, and
`Matrix::rotate(θ)` turns anticlockwise in it — so the caller negates once,
at the one place it converts, exactly as `canvas::mapping` does for every
other quantity that crosses.

Doing the flip here would put a page-space fact in a function that has never
seen a page, which is how a preview and a commit come to disagree about
which way round something went.

`None` when either ray is degenerate — the pointer exactly on the centre —
because a bearing from a zero-length ray is not a number and
`atan2(0.0, 0.0)` quietly answers zero rather than saying so.

### `fn snap`

It snaps the **total turn**, not the increment. Accumulating snapped
increments would let a slow drag through 90° arrive at 87°, because each
frame's small delta rounds to zero — the classic error, and the reason this
takes the whole angle rather than a per-frame one.

### `fn rotate_about`

The ghost is drawn from **this** function and the commit from
`Matrix::rotate(θ).about(centre)`, which are the same map in two spaces —
and that is the one duplication in this module. It is not avoidable: the
preview must be drawn in screen space before the page conversion, and the
commit must be expressed in the engine's own type. What makes it safe is
that both take **the same θ from [`angle`]**, so the two can differ only in
the y-flip, which is a sign a unit test can pin.

### `fn drag`

Mirrors [`crate::canvas::resizing::drag`] deliberately, down to the return
type, so a reader who has understood one has understood both. What it hands
back is the **angle** for the ghost, where the resize hands back two factors
and the move hands back a displacement.

Returns `Some(radians)` only when a ghost should be drawn — which, by this
project's honesty contract, is exactly when a release would commit.

# The negation, and it happens exactly once

[`angle`] measures in **screen** space, where y runs down, so a clockwise
drag comes back positive. `Matrix::rotate` turns anticlockwise in PDF user
space, where y runs **up**. The single `-` below is that crossing, and it
lives here rather than in `angle` for the reason `canvas::mapping`'s header
gives about every other conversion: one place, or the preview and the commit
eventually disagree about which way round something went — which is a defect
that looks like a deliberate feature.

The ghost is drawn from the **un-negated** angle, in screen space, by
`overlay::draw_rotate_ghost`. Both come from one call to [`angle`], so the
only thing that can differ between them is this sign.

### `struct Frame`

The `Frame` shape `canvas::resizing` and `canvas::handledrag` already use,
and for the reason they give: the members are read-only facts about one
frame, so grouping them says what they are and removes the failure a long
parameter list invites — `from` and `at` are both `Pos2` in the same space
and swapping them would compile and turn the object backwards.
