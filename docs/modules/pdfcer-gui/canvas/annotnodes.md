# `canvas::annotnodes` — **the nodes of a markup shape, and moving them**

## The operator's report, verbatim

> *"I also can't edit or delete nodes of a markup shape once it is drawn."*

That sentence named three separate absences and this module closes the one
it literally describes. The **ce dimension** half was closed the same day by
[`crate::canvas::dimdrag`] — a ce dimension's corners have had engine verbs
since `Pass 107.0`. The **markup** half could not be built at all until
`Pass 255.0`, because `pdfcer-core` did not model a markup annotation's
geometry: `/Vertices`, `/L` and `/InkList` were not in the read model and
there was no verb that could rewrite one. It was filed
(`request_a_markup_shapes_vertices_cannot_be_read_or_edited.md`) and
deliberately **not** worked around — re-parsing the annotation dictionary in
this shell would have been a second, weaker implementation of geometry the
engine owns, and the engine's own note explains exactly what a shell that
did so would ship:

> A move has two halves and **only one of them shows up in a render.**
> `/Rect` moves the painted result for free; the geometry keys hold
> *absolute page coordinates*, and they are what any **other** tool
> regenerates an appearance from.

⇒ Every instrument this project owns — the rendered canvas, a screenshot, a
driven pixel check — reads the appearance stream. A shell that moved a node
by rewriting `/AP` alone would pass all of them and be wrong in Acrobat a
week later.

## What the engine gives us, and the matrix it enforces


| `/Subtype` | family | move | insert | remove | floor |
|---|---|---|---|---|---|
| `/Polygon` (plain or cloudy `/BE`) | `VertexEdit` | yes | yes | yes | 3 |
| `/PolyLine` | `VertexEdit` | yes | yes | yes | 2 |
| `/Line` (incl. arrows) | `VertexEdit` | yes (index 0/1) | refused | refused | — |
| `/Ink` | `InkEdit`, addressed `(stroke, point)` | yes | yes (after a stroke's last point **extends** it) | yes | 2 **per stroke** |
| `/Square`, `/Circle`, text markup | — | refused | refused | refused | — |

**This shell knows the first two columns and nothing else in that
table.** [`geometry`] decides which shapes have *anchors to draw* and which
verb family addresses them — both are routing questions that have to be
answered locally — and every question about whether an edit is **allowed**
goes to the engine. The distinction still matters, and `/Ink` is still the
row that proves it, from the other side now: until `Pass 278.0` it had
readable geometry (`Annotation::ink_list`) and no editable geometry, and
this module refused to draw anchors from a readable-but-uneditable field.
The day the verbs shipped, the row flipped **here**, in one `match` arm,
and the honesty of the anchors is still the same rule: an anchor is drawn
only where a verb can act on it.

## `/Ink` — one anchor list, two index spaces, no bridging segment

`/InkList` is a list **of** strokes, so the engine addresses an ink point as
`(stroke, point)` while everything on this canvas — the painter's trace
regions, the press classifier, the gesture machine — carries one flat
`usize`. [`ink::StrokeTable`] is the side table that converts between them,
and its header carries the two rules that fall out: the preview draws **no
segment between strokes** (a bridge the file does not hold), and *insert
after a stroke's last point* **extends that stroke** rather than crossing
into the next — the engine's own rule on `InkEdit::InsertPoint`. Every
point of every stroke is an anchor today; decimation is a known follow-up,
argued in that header, not an oversight.

The engine's reply also settled the one objection that would have made this
a different feature: pdfcer draws an `/InkList` as a **polyline** (`m` then
`l`), so a point drag moves exactly the two segments beside it and the
polyline preview this module already draws is *exact* for ink. What it
cannot draw is the consequence for a stroke **another producer** drew and
smoothed: re-baking straightens it. `InkForecast::appearance_was_pdfces` (old-name-exempt: the engine's own field name, quoted verbatim)
reports that and `app::actions::annots` discloses it off-canvas, through
the same list `measure_stale` travels on — never as a mark on the canvas.

## The preflight is asked EVERY FRAME, including for a plain move

`reshape_annotation_preview` shares one body with the mutating verb
(`reshape_plan`), so it cannot disagree with what a release would do. The
engine's standing advice, given to this project when the ce-dimension
vertex verbs landed and repeated on this Pass, is not optional:

> *ask the preview verb every frame rather than catching the error
> afterwards* — *"a verb with no preflight makes the UI find out by
> pressing."*

What it buys is the honesty contract every drag in this canvas is held to:
**the preview is a shape the release would commit, or it is the shape that
is already there.** A drag that begins and then fails is worse than a drag
that never starts, because it looks like it worked until the next frame
repaints.

It is asked for the **move** as well, which is where this module differs
from [`crate::canvas::dimdrag`]. That module does not preflight a corner
move, on an explicit engine ruling: a ce dimension's move cannot be refused
once the drag has begun, because a self-intersecting polyline has a
well-defined length and every remaining refusal is structural. A **markup**
move can still be refused mid-drag — `AnnotationVertexNotPlaceable` fires on
a non-finite coordinate, which is precisely what a page whose transform will
not round-trip produces — so the cheaper reasoning does not carry over and
is not borrowed.

## Rule 9 — what an unavailable capability draws

**Nothing.** A `/Square`, a `/Circle`, a text markup — and an `/Ink` whose
`/InkList` the engine could not read — get no anchors, no greyed anchors and
no ghost anchors: [`geometry`] answers `None` and the painter's loop is
empty. There is no *temporarily* unavailable case here to grey — the refusal
is a property of the shape's kind and will not change while the operator
looks at it.

What they get instead is a **sentence**, and it is delivered by
[`explain_unreshapable`] at the moment the operator asks: with the Points
tool armed — the deliberate act of arming the tool whose whole subject is
nodes — a selected markup that has no nodes says so, once, naming its own
kind. That is the difference between *"this shape has no nodes"* and *"this
program forgot to draw them"*, and it is the only difference the operator
can see.

## Rule 4 — this is the cursor, not the document

Node anchors and the in-flight polyline are **pre-commit affordances**,
which R8b rule 4 names by name: *"a snap indicator, a hover highlight, a
rubber-band … these are the cursor"*. They are drawn for the selected
annotation only, they vanish with the selection, and nothing already applied
to the page is tinted, badged or flagged. The one-line test passes: a
screenshot of the canvas mid-drag differs from the saved file by a marching
outline and some small squares, which is where the pointer is and not what
the document says.

## Rule 15

Everything here is about a **markup shape** — a `/Polygon`, `/PolyLine`,
`/Line` or `/Ink` the operator drew as a comment. A **ce dimension** is also a `/Line`
and is claimed by [`crate::canvas::dimdrag`] before this module is reached;
the engine refuses it from these verbs by name
(`EditError::AnnotationIsCeDimension`) as the backstop. **pdf dimensions** —
CAD page content — are not annotations at all and are nowhere near this
module.

## Item notes

### `const NODE_GRAB_SLACK_PT`

The drawn square is the **promise**; this is the **target**. They differ
because a 7 pt square is hard to hit on a dense drawing, and the standing
convention here — stated at `handles::grip_at` and again at
`dimdrag::vertex_at` — is that a grip's live area may exceed its drawn one
and never the reverse. A target smaller than its picture is the operator
missing something they can plainly see.

### `fn edited`

Returned rather than drawn, so the preview and the action are built
from **one** value. A second derivation of *"what would this look like"*
is the defect `measure::Resolved` exists to prevent, and it has shipped
on this canvas twice.

`None` for an index the list does not hold, which the preflight would
also refuse — asked here as well because this function is where the
slice is indexed and a panic mid-drag would take the window with it.

For an `/Ink` the stroke table moves with the points:
[`ink::StrokeTable::after_edit`] grows or shrinks the **grabbed**
stroke, so an insert after a stroke's last point extends that stroke —
the engine's rule — and the preview's boundaries stay true to the list
it is drawn from.

### `fn planned`

`None` only for an `/Ink` anchor index the stroke table cannot place — a
press the painter could not have drawn an anchor for. A single-list shape
always answers, and leaves an out-of-range index to the engine's own
`AnnotationVertexIndexOutOfRange`, exactly as before ink existed.

### `fn refusal_for`

The mapping lives **here** rather than in `crate::text::markup`, for
`dimdrag::refusal_for`'s reason and this project's standing division: the
engine's error enum is a *shell* concern, and the string catalog holds
operator prose only. A `crate::text::` module that matched on `EditError`
would put the engine's vocabulary into the catalog and give the catalog a
reason to change every time the engine adds a variant.

The engine offers a `reason: &'static str` on
[`EditError::GeometryNotReshapable`] and says a shell may show it verbatim.
It is **not** shown verbatim, and the choice is deliberate rather than
squeamish: those sentences are written for a developer reading a CLI —
*"author a PolyLine instead"*, *"use resize_annotation"*, *"/QuadPoints are
text-anchored quadrilaterals"* — and they name verbs and PDF keys this
operator has never seen. The `subtype` field is what is used, because that
is the fact the operator can check against the shape in front of them. The
engine's sentence goes to the **trace**, where the developer is.

# The five ink refusals of `Pass 278.0`, each answered

| engine says | sentence | why that one |
|---|---|---|
| `InkStrokeWouldBreachPointFloor` | `StrokeWouldLeaveTooFew` | the floor is **per stroke** (two), so *"the shape has as few corners as it can have"* would be false of a mark whose other strokes have plenty; the next act is *add a point to this stroke* |
| `InkPointIndexOutOfRange`, `InkStrokeIndexOutOfRange` | `PointNotFound` | the anchors and the file have gone out of step; the next act is to reselect, which rebuilds both from one walk |
| `InkWouldBeEmpty` | `WouldLeaveNothing` | only a whole-stroke verb can raise it and this shell calls none — worded anyway, because an unreachable refusal that becomes reachable silently is how a grip comes to do nothing |
| `InkVerbOnNonInk` | `Refused` | a routing defect in this shell (a non-ink shape reached the ink planner); no sentence about nodes helps the operator, and the engine's own sentence names it in the trace |

The `_` arm is not laziness. The remaining refusals —
`AnnotationNotFound`, `AnnotationIsCeDimension`, `AnnotationLocked`,
`AnnotationVertexIndexOutOfRange`, `DocumentEncrypted`, the certification
guard, `MarkupSpec` — are either unreachable from an anchor this shell drew
from this same geometry, or are properties of the FILE that no wording about
nodes would help with. They get the general sentence rather than a
fabricated specific one, and the operator learns that the press was heard.

### `fn shape_word`

A mapping and not a passthrough. `"PolyLine"` is a PDF name; *"a
polyline"* is a shape. `"Square"` is the PDF name for what pdfcer's own
ribbon calls a **rectangle**, and showing the operator "Square" for the
thing they drew with the Rectangle tool is the surface disagreeing with
itself. The unknown arm keeps the raw name rather than inventing one,
because a subtype this shell has never heard of is better named exactly than
named wrongly.

### `struct Resolve`

A struct for [`NodeFrame`]'s reason and one more: it is the seam that lets
every rule below be tested **against the real engine** without a window, a
pointer or an `egui::Context`. `dimdrag::CountEdit` draws the identical seam
for the identical reason, and its own tests are the precedent — a test that
faked the annotation would be asking the engine about a shape that does not
exist and would get `AnnotationNotFound` for every case while looking
exactly like a test that passed for the right reason.

### `const NODE_REGION`

Published by the painter so a driven check can **aim** at a node. Where a
handle is sits at the end of a page → canvas → screen conversion and is a
fact only the running application knows; a harness that guessed *"somewhere
near the corner I clicked"* would land on the page instead, start a marquee,
and then pass while exercising a completely different gesture.

Deliberately **not** `canvas.dimension-vertex`. Two subjects that reach
two different engine verb families must be distinguishable in the trace, or
a check that aimed at a ce dimension and hit a markup shape would report a
working build as broken and vice versa.

### `const TRACE_MOVE`

`address=` is `stroke/point` for an `/Ink` and `none` otherwise;
`family=` is `vertex` or `ink`, naming which engine planner was asked.

**`markup-node-`, NOT `markup-vertex-`, and the rename is a caught
defect rather than a preference.** `canvas::markup::vertex` has written
`markup-vertex kind=… page=… n=… x=… y=…` since polygons became
authorable — one line per CLICK while the operator is drawing a shape.
A move line under the same first token would have made `Trace::last("markup-vertex")`
return whichever came later, so a check asserting a node MOVED would have
read a line about a node being PLACED and reported a working build as
broken, or the reverse. `tools/gates/check-trace-names.py` catches this
collision only against **funnel labels**; a module-to-module collision is
still found by reading, which is how this one was found.

Distinct from the funnel's `move-annotation-vertex`, which is the engine's
acknowledgement that the document changed. A check that read only one of the
two could not tell a shell that never asked from an engine that refused; see
`tools/gates/check-trace-names.py` for the three times that cost a day.

### `struct Geometry`

Returned by [`geometry`], read by the painter, the hit test, the preview and
the right-click menu — one value, so the anchor the operator sees, the anchor
the press finds and the address the engine is asked about are derived from
the same list in the same frame.

# `strokes` is the one field that knows which verb family this is

`None` is a `/Polygon`, `/PolyLine` or `/Line`: the engine addresses a node
by one index into `/Vertices` (or `/L`), through `VertexEdit`. `Some` is an
`/Ink`: the engine addresses a point as `(stroke, point)`, through `InkEdit`,
and the table converts this list's flat index to that pair. [`Plan`] is the
only place that branches on it, and everything else in this module treats
the two alike — which is what makes a freehand mark's drag look and behave
exactly like a polyline's, the property the operator asked for.

### `fn geometry`

# Which shapes answer, and why the list is short

| `/Subtype` | source | closed | addressed by |
|---|---|---|---|
| `Polygon` | `/Vertices` | yes | one index, `VertexEdit` |
| `PolyLine` | `/Vertices` | no | one index, `VertexEdit` |
| `Line` | `/L`, two points | no | index 0 or 1, `VertexEdit` |
| `Ink` | `/InkList`, flattened | no | `(stroke, point)`, `InkEdit` — see [`ink`] |

Everything else answers `None`, and the exclusions are the interesting ones:

* **`/Square` and `/Circle`.** Defined by `/Rect`, not by vertices — they
  already have eight resize grips, which is the verb for them.
* **Text markup.** `/QuadPoints` follow the words they cover and have no
  corners of their own; the engine refuses them by name.
* **An `/Ink` with no readable `/InkList`.** `Annotation::ink_list` is
  `None` for a malformed or absent array, and an anchor list with nothing
  in it is the honest answer — the engine would refuse every address.

`/Ink` was on the refused list until `pdfcer-core` `Pass 278.0`
(`c8a6697`), and the argument for refusing it then is the argument for
drawing it now: an anchor is drawn only where a verb can act on it. The
verbs exist, so the anchors do. [`ink`]'s header carries what changed.

# A cloudy `/Polygon`'s anchors are on its VERTICES, not on its outline

A revision cloud is a `/Polygon` carrying `/BE << /S /C >>`; its scallops
are baked into `/AP` from the pre-bulge vertex list, and `/Rect` bounds the
**bulged** outline. The engine states this and states what a shell should
do with it: *"A shell drawing anchors draws them here, not on the cloud's
outline."* So the anchors sit slightly inside the ink, which is correct and
is what every editor in the class does with a stylised stroke.

# Three "no"s, and they are different kinds of no

| condition | what it means |
|---|---|
| an annotation is selected | otherwise the content branch owns the press |
| it is [`AnnotKind::Markup`] | a ce dimension is `dimdrag`'s, and it re-measures |
| it is not **locked** | §12.5.3 Table 165 bit 8 — *the file* says the user interface may not change this |

The locked case is honoured **here**, before an anchor is drawn, rather than
being left to the engine's refusal. A handle drawn on a shape the document
forbids changing is a promise the release cannot keep. Note this is bit 8
(`Locked`, 128) and **not** bit 10 (`LockedContents`, 512) — the engine
consults exactly the same one, and its own note records that treating either
as the other is a spec-contradicting bug in one direction or the other.

### `fn node_at`

# The comparison is in SCREEN space, and that is why this converts rather
than the caller

An anchor is a screen-space affordance of a fixed size. Comparing in canvas
or page space would make the target shrink as the operator zooms out —
exactly when a shape's nodes are closest together and precision matters most
— and balloon as they zoom in, so that at 800 % a press anywhere near a node
would grab it. The conversion has to happen on the side of the boundary
where the tolerance is meaningful.

# Ties go to the LAST node, deliberately

Two coincident nodes are legal in a `/Vertices` array and the engine does
not de-duplicate. If the operator has made one and wants it gone, the one
they can reach is the one they can drag away, and the later index is the one
they just placed. `dimdrag::vertex_at` resolves the identical tie the
identical way; two node gestures on one canvas that disagreed about which
coincident point they grabbed would be a difference nobody could see and
everybody would trip over.

### `fn preview_of`

One function, used by the preview and by nothing else that could disagree
with it. A closed shape gets its closing segment here rather than at the
call site, because a caller that had to remember to add it is a caller that
will one day draw an open triangle over a closed one.

### `fn segment_pairs`

A single-list shape joins consecutive nodes and, when closed with three
or more, the last back to the first — [`preview_of`]'s rule, restated
here as indices so the menu can name the segment's first node. An
`/Ink` joins consecutive points **within** each stroke only; the
boundary between two strokes is not a segment, because the file does
not hold one and the release would not commit one.

### `fn segments`

One derivation with [`Self::segment_pairs`], so a pair the menu offers
is a segment the painter draws; and for a single-list shape identical
to [`preview_of`], which the tests assert rather than assume.

### `enum Plan`

Built once by [`planned`] and handed to **both** the preflight and the
action, so the question asked and the question answered are literally the
same value. A shell that preflighted `Remove { index }` and then committed
`Remove { index: index + 1 }` would pass every unit test either half has.

Two variants and not a trait object, for `AnnotAction`'s own reason: the
two families reach **two different engine planners** with two different
refusal vocabularies, and the one thing that must never happen on this
canvas is a gesture aimed at the wrong verb. An `/Ink` handed to
`reshape_annotation` is refused by name (`GeometryNotReshapable`); a
`/Polygon` handed to `reshape_ink` is refused by name (`InkVerbOnNonInk`).
Both refusals are worded in [`refusal_for`] as the backstop, and neither
is reachable while this enum is built from [`Geometry::strokes`].

### `struct NodeFrame`

A struct rather than ten parameters — `dimdrag::VertexFrame`'s own argument,
adopted rather than re-argued: three members are `Option`s of borrowed
things and two are `Pos2`s in the same space, both of which a positional
list would let a caller swap silently.

### `struct NodeDrag`

Two fields rather than one, for `dimdrag::VertexDrag`'s reason: the polyline
is page-space geometry drawn through the canvas transform, and the marker is
a screen-space glyph at the snap candidate. Folding them would make the
caller unpack a tuple whose members it uses in two places, forty lines
apart.

### `fn drag`

Returns the page-space segments the shape would be drawn as if the operator
released now, or an empty [`NodeDrag`] when the drag reaches no verb.

# The order, and why the preflight comes before the arithmetic that draws

1. resolve the shape, the page and the grabbed node;
2. turn the pointer delta into a page-space target, **snapped**;
3. read the live intent — move, insert or remove;
4. **ask the engine whether that edit is allowed**;
5. draw the answer: the edited shape if it is, the shape as it stands if it
   is not;
6. on release, raise exactly one action — or record exactly one sentence.

Step 4 before step 5 is the whole design. A build that drew the node
vanishing and then refused on release would be showing the operator an edit
that never happens.

### `fn explain_unreshapable`

# Why this exists at all, and why it is not a greyed anchor

R9: *an unavailable capability renders **nothing**; greying is only for
temporarily unavailable, always explained on hover.* A `/Square` will never
grow nodes, so greyed anchors on one would be a control that is permanently
inert — the exact failure class this project's `DEFECTS.md` is made of, and
the shape of the operator's own report.

But *nothing* is also what a build that forgot to draw the anchors renders,
and the operator cannot tell those two apart by looking. So the absence is
**stated**, in a sentence, at the moment they ask for it.

# When it fires, and why that moment

The **Points tool is armed** and a markup shape with no nodes is selected.
Arming that tool is the deliberate act — its whole subject is nodes — so it
is the moment the question *"where are the nodes?"* is actually being asked.
It is the same reasoning `Declined::NodeToolNeedsEditMode` already uses one
step earlier: *a key that does nothing has no control to hover, which makes
it the case that most needs a sentence rather than the least.*

**Once per subject, not once per frame.** The pair
`(annotation, is-the-tool-armed)` is remembered in `egui::Memory` and the
sentence is raised only when it changes. Writing the decline slot sixty
times a second would work — the write is idempotent — and would silently
stamp on every other sentence the operator was reading. A status line that
cannot be replaced by anything else is not a status line.

Returns `true` when it raised the sentence, which is what the unit tests
assert and what makes "did it fire once?" a question with an answer.
