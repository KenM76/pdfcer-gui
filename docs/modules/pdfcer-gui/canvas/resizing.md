# `canvas::resizing` — what the eight resize grips commit

## The verb is `transform_objects`, and it is kind-agnostic

`EditSession::transform_objects` wraps each object's operator run in
`q <cm> … Q`. **That never looks at an operand**, which is what makes it
kind-agnostic — not a match arm per kind that somebody has to remember to
extend. A text run and an image therefore resize exactly as a path does
(neither has nodes to move, and neither needs any), and a selection of N
objects is **one** call, one command, one undo entry.

**Stroke width is not scaled on page content**, and that is a decision
rather than a consequence: on a CAD drawing a line weight is a *drafting
standard* — 0.25 mm is 0.25 mm whatever size the detail is — so keeping it
is right far more often than scaling it would be, and it is what every
drafting package does. It is nonetheless something pdfcer decided and the
operator did not, so it is **disclosed** ([`crate::text::resizing`]) rather
than assumed.

**The matrix is PAGE space and nothing else.** `cm` composes into the CTM
in force at that point in the stream — the object's *user* space — so the
engine emits `X = CTM × M × CTM⁻¹` per object from that object's own captured
CTM. A selection spanning two local spaces gets two different `cm` operands
for one gesture and both land where the operator pointed. Passing anything
but page space from here would be right only where an object's CTM happens to
be the identity and **silently wrong at every scale the producer left in
force**.

---

## The arithmetic, and why it is not written out here

**Scaling about an anchor is moving every point.** For an anchor `a` and
factors `(sx, sy)`:

```text
p' = a + (p - a) * (sx, sy)
```

`Matrix::scale(sx, sy).about(a)` is `translate(a) × M × translate(-a)`,
which is that expression exactly — so the map is stated once, by the crate
that owns matrices, rather than once per point here. A shell keeping its own
copy would be a second derivation of one answer in coordinate space, which
is the shape every silent defect this project has met there has had.

One gesture is **one call, one command, one undo entry** — this project's
standing rule for a gesture (`canvas::moving`'s §1), and the thing a
per-object loop would break both by producing N undo entries and by planning
each edit against byte offsets the previous one invalidated.

The operator's instruction: *"finish off phase 1 and phase 5. Get everything
unblocked on phase 5 — no excuses about slowness of feature from pdfcer as a
reason not to implement."*

## Why the arithmetic is here and not in `moving`

[`crate::canvas::moving`] is about a **displacement** — one delta applied to
whatever the rung named. This is about a **map**: every node goes somewhere
different, and the somewhere depends on where it started. Folding it in
would put two different shapes of answer behind one `MoveSubject`, and the
module that owns the ghost preview would have to branch on which.

## The ghost, and rule 4

An in-flight resize draws its **new outline**, not a tint over the old one —
`canvas::overlay`'s existing move ghost with a different transform. It is a
pre-commit affordance and therefore the *cursor*, which R8b's fourth clause
welcomes explicitly. Nothing is drawn onto the applied content, and a
screenshot of the page after a commit is a screenshot of the page as it will
save.

## Item notes

### `fn to_pdf`

Mirrors [`crate::canvas::moving::drag`] deliberately, down to the return
type, so the caller's two arms read the same and a reader who has understood
one has understood both. What it hands back is the **scale factors** for the
ghost, where the move drag hands back a displacement.

# A refusal is worded ONCE, on `Complete`

Not on every frame of the drag. `moving::drag` makes the same choice and its
reason applies unchanged: an in-flight gesture is a question, and answering a
question the operator has not finished asking would put a sentence on the
status row sixty times a second while they were still deciding.

### `fn scaled_about`

# Both corners, because a pivot is not always a corner

This is the identical map `overlay::draw_resize_ghost` paints —
`pivot + (p - pivot) * s` — so the box that is written is the box the
operator watched, by construction rather than by two derivations agreeing.

The earlier spelling derived one far corner as `bounds.min + bounds.max -
2 * pivot`, which is right only while the pivot **is** a corner.
[`Grip::pivot`] deliberately answers a **mid-edge** point for the four edge
grips, and on such a grip's cross axis that expression is exactly zero — so
the derived corner collapsed onto the pivot and an edge drag committed a
zero-extent `/Rect`. The corner grips were unaffected and kept working,
which is what O209 reports as *"only the corner drag handles work."*

### `fn an_edge_grip_scales_only_its_own_axis`

The regression O209 names: *"only the corner drag handles work."* An
east or west drag leaves the height alone and a north or south drag
leaves the width alone, because [`Grip::pivot`] answers a **mid-edge**
point on those four and the box is scaled about it rather than reflected
through it.

It asserts the extent that must survive, not merely that the rectangle
is non-empty — a box collapsed on one axis and a box that merely failed
to grow are both "not what the operator dragged", and only the first was
the defect.

### `fn a_corner_grip_pins_the_opposite_corner`

The two spellings agree wherever the pivot is itself a corner, and that
is the half that kept working — so this is the control that says the
repair widened the set of grips that commit rather than moving it.

### `fn the_south_east_grip_grows_both_axes`

The base case, and the one whose y sign is easy to get backwards: screen
y is down, so a positive `dy` on a *south* grip is growth. Getting it
wrong produces an object that shrinks when you pull it bigger, which is
the kind of defect that survives review because both directions "look
like a resize".

### `fn a_mid_edge_grip_leaves_the_other_axis_alone`

A build that treated them as corners would let an operator aiming at
"make this wider" also make it taller — a change they did not ask for,
on the axis they were deliberately not touching.

### `fn collapsing_and_mirroring_are_refused`

Clamping would silently substitute a different edit for the one the
operator made — and a mirrored path is a legal, plausible-looking
document they did not ask for.

### `fn the_anchor_stays_put_and_distance_scales`

Asserted as the two properties rather than against a table of
coordinates, because the properties are what "resize about a corner"
means and a coordinate table would pass for a build that had the anchor
at the centre.

### `enum Refusal`

Every variant is **a sentence to show**, never a silent drop —
`canvas::textedit::Refusal`'s rule. A gesture whose answer to a case it
cannot handle is to do nothing is a gesture the operator reports as broken,
because from the outside it is indistinguishable from one.

### `fn factors`

# Why the anchor is the OPPOSITE corner and not the centre

Because that is what every drawing application does, and the standing
tie-breaker for anything an operator compares against the tools they already
use is to behave the way those tools behave. Dragging the south-east grip
moves the south-east corner and leaves the north-west one exactly where it
is — so the part of the object the operator is *not* pointing at does not
move under their hand.

[`Grip::anchor`] already answers this, in **screen** space, for the drawing
side. This computes in the same frame and hands the result to the caller to
map, rather than re-deriving the opposite-corner rule: two spellings of
"which corner stays still" would eventually disagree, and the disagreement
would be an object that jumps on the first frame of a drag.

# The mid-edge grips scale ONE axis

`East` and `West` scale x and leave y at 1.0; `North` and `South` the
reverse. That is what a mid-edge grip means, and it is why they are offered
separately from the corners rather than being four more corners.

### `fn is_usable`

A factor at or below zero collapses or mirrors the object. Refused rather
than clamped — see [`Refusal::Degenerate`].

The floor is not `0.0` but a small positive number, because a drag that
passes exactly through zero would otherwise produce a
zero-area object whose next resize has no bounds to scale from: the
`w <= EPSILON` guard in [`factors`] would then answer `None` for ever and
the object could never be recovered except by undo.

### `fn action`

Pure, so the whole decision is testable without a window: the selection, the
object model, the anchor in PDF space and the two factors go in, and one
`VectorAction::MoveNodes.into()` or one named refusal comes out.

# The anchor arrives in PDF user space, already converted

The caller converts once, through `canvas::mapping`, for the reason
`canvas::textedit::resolve_run` records about its own two hops: a second
conversion is how a preview and a commit come to disagree about where the
operator's hand was.

### `struct Frame`

A struct rather than seven parameters, and it is not only clippy's
arity rule: **five of the seven are read-only facts about the same frame**,
so grouping them says what they are. It also removes the failure a long
parameter list invites — `map` and `page` are both `Option<&…>` and adjacent,
and swapping them would compile if their types ever converged.

`selection`, `provider` and `actions` stay outside it, deliberately: the
first two are *the document's* state rather than the frame's, and the third
is an output. A struct that mixed all three would be a bag rather than a
grouping.

### `const rain`

Live, unlike `gesture::Drag::shift`, and the two are different facts
that happen to read the same key. That one asks *"what did this gesture
MEAN"* — extend the selection or replace it — and must be sampled at the
press, because the meaning of a gesture cannot change half-way through
it. This asks *"is the operator constraining right now"*, and every
program in the class lets that be picked up and put down mid-drag. An
operator who starts a free resize, sees it going crooked and grabs Shift
expects the shape to snap to proportion under their hand.

### `mod ifiers`

Carried on the [`Frame`] rather than read from `egui::Memory` inside
this module, so `resizing` stays a pure decision over its inputs and
stays testable without a `Context`. Every other live fact on this struct
arrives the same way, including `constrain`.

### `fn decline`

One place, so a variant added to [`Refusal`] is a compile error in
`crate::text::resizing` rather than a drag that silently does nothing —
which is the failure `canvas::textedit`'s own history is about.
