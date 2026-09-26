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
