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

★★ **This shell knows the first two columns and nothing else in that
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

## ★★ `/Ink` — one anchor list, two index spaces, no bridging segment

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

## ★★★ The preflight is asked EVERY FRAME, including for a plain move

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

★ It is asked for the **move** as well, which is where this module differs
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

★★ What they get instead is a **sentence**, and it is delivered by
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
