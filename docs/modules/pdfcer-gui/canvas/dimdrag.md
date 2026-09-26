# `canvas::dimdrag` — **dragging a ce dimension to where it should be drawn**

## The operator's report, verbatim

> *"I need to be able to move the dimension after it has been laid down,
> and there should be a preview of the dimensioning lines as I lay it down
> and click to position it when it is created and after the fact."*

Two halves. The first half — *as I lay it down* — already shipped:
[`crate::canvas::measure`]'s third click places a new ce dimension and
previews it through `measure::pick::dimension_preview_segments`. This
module is the second half, *after the fact*, and it reuses that same
preview function for the same reason: a preview derived a second way is a
preview that can disagree with what commits.

## What "move a dimension" means, and why it is NOT `move_dimension`

`pdfcer-core` offers two verbs and picking the wrong one is the whole design
decision here:

| verb | what it changes | what the number does |
|---|---|---|
| `EditSession::move_dimension` | translates the **measured points** with the drawing | unchanged (a rigid motion preserves a distance) — but the dimension leaves the feature it was measuring |
| `EditSession::place_dimension` | writes `offset` and `text_along` only | **cannot** change, by construction: the value function does not read either field |

Dragging a dimension is `place_dimension`. That is what SolidWorks does —
the attachment points stay on the geometry and the extension lines stretch
— and it is what the engine's own doc comment says the verb exists for:
*"This, not `move_dimension`, is what dragging a dimension does."*

The consequence worth stating out loud, because it is the property that
makes this gesture safe enough to be the *default* action on a press:
**no drag, however far, can alter the printed number.** An operator can drag
a dimension across the sheet and back and the document's measurements are
unchanged. `move_dimension` has no such guarantee — it would take a
dimension off the feature it annotates — so it is deliberately not wired to
a drag and remains available only where the operator has said they mean it.

## The delta is resolved in the dimension's OWN frame

A page-space delta is projected onto the two axes `axis_frame` gives:

```text
offset'     = offset     + delta · n    (perpendicular — how far the line stands off)
text_along' = text_along + delta · u    (parallel      — where the number sits along it)
```

**A delta, not `placement_from_point`.** Both were available and the
absolute form is shorter, but it resolves the placement from wherever the
*pointer* is, which means the dimension jumps on the first frame of the drag
so that its anchor lands under the cursor. A delta preserves the grab: the
dimension moves exactly as far as the hand does, and whatever part of it the
operator grabbed stays under their finger. The absolute form is the right
one for authoring — where there is no grab to preserve, because the
dimension does not exist yet — and that is precisely where `canvas::measure`
uses it.

## Two kinds may be dragged, and a perimeter is the easier of them

**Linear** resolves the delta in the dimension's own axis frame, above.
**Perimeter** does not have one — a shape has no single axis to build a
frame around — so the engine anchors its label at `centroid + (text_along,
offset)` in the **page's** axes, and the delta is the answer with no
projection at all.

One consequence is worth naming rather than discovering: a perimeter's label
is **strictly more free** than a linear one's. Drag a linear label
diagonally and it is flattened onto its axis, because that is where a
dimension line's text lives; drag a perimeter's and it lands where you
dropped it. That is not an inconsistency to iron out — it is the difference
between a label that belongs to a line and one that belongs to a shape.

## Why an ANGULAR dimension may not (and why that is not a stub)

`place_dimension` accepts angular dimensions too, but its two arguments mean
something different there: `offset` is an **arc radius** from the apex and
`text_along` is in **degrees**. Adding a dot product measured in points to a
quantity measured in degrees is not a smaller version of the right answer,
it is arithmetic on mismatched units, so [`placed`] refuses.

The refusal is honoured at the *press*, not at the release: [`grab_box`]
returns `None` for anything it cannot drag, so the press falls through to
the ordinary marquee and no gesture is ever started that could not finish.
That is this project's no-placeholders invariant applied to a gesture rather
than to a widget — an inert drag is a visible control that silently does
nothing, which is the exact failure class `DEFECTS.md` is made of.

Angular placement by drag is worth having and is written up rather than
faked; see the TODO note on [`placed`].

## What this module does NOT decide

* **Whether the press is a drag at all.** `canvas::gesture` owns that. This
  module only supplies the hit box that makes the press mean *move*.
* **Where the dimension is drawn once committed.** `pdfcer-render` draws the
  annotation; this module draws only the in-flight preview, and only from
  the same segment function a committed dimension is previewed from.
* **Undo granularity.** `place_dimension` is one command, so one drag is one
  undo entry, decided by the engine.

## ADDING AND REMOVING A CORNER — 2026-09-05, the operator's report

> *"I also can't edit or delete nodes of a markup shape once it is drawn."*

That sentence is three separate facts. The one this module can close is the
**ce dimension** half: `pdfcer-core` has had `insert_dimension_vertex` and
`remove_dimension_vertex` since `Pass 107.0`, beside the
`move_dimension_vertex` this module already drove, and **this shell called
neither**. So *"delete a node"* was unbuilt on our side for the one shape
where the engine can do it. The other two facts are not ours: a **markup**
shape's `/Vertices` and `/InkList` are not modelled at all, which is filed
(`request_a_markup_shapes_vertices_cannot_be_read_or_edited.md`) and
deliberately **not** worked around here — re-parsing the annotation
dictionary in the shell would be a second, weaker implementation of
geometry the engine owns.

### The gesture, and why both verbs are DRAGS

| gesture on a corner handle | means |
|---|---|
| drag | **move** that corner (unchanged since 2026-08-20) |
| **Points tool armed** + `Ctrl` + drag | **add** a corner immediately after it, dropped where the pointer lands |
| **Points tool armed** + `Ctrl`+`Shift` + drag | **remove** that corner |

**A click cannot reach this module, and that is a fact about the gesture
machine rather than a preference.** [`crate::canvas::gesture::GestureState::update`]
starts a drag on `frame.drag_started` — egui's drag recognition, past its
travel threshold — and a press released before that threshold never becomes
a `DragKind`; it becomes `GestureOutcome::Click`, which is routed by
`canvas::clicking`. So the engine's own framing of these two verbs
(*"what a right-click on a segment offers"*, *"what a right-click on a
vertex offers"*) describes a surface that lives in `canvas::menus`, and the
drag is what this module can actually be handed. The right-click menu is the
discoverable form and is **reported rather than faked**.

**The Points tool is the safety, and that is what earns it its place in
Review.** Ctrl alone, with the Select tool, still moves the corner. An
operator has to have deliberately armed the tool whose whole subject is
points before a drag can change how many there are — so a mis-held modifier
during an ordinary corner drag cannot destroy a corner. It is also the
answer to *"what does the Points tool do in a mode that cannot edit page
content"*, which is the question [`crate::canvas::tool::retire_forbidden`]'s
Node arm now answers with `edit_content || author_measure`.

### The modifiers are read LIVE, and here that is the honest form

Every *sampled* property of this drag — which vertex, which grip, which
marquee intent — is fixed at the press, because a gesture means what it
meant when it started and re-deriving a hidden choice per frame lets a drag
hop onto something the operator never aimed at. **A modifier whose effect is
continuously previewed is the opposite case.** Releasing Ctrl mid-drag turns
the gesture back into a move *and the preview says so on that very frame*,
so nothing is hidden and nothing can be committed that was not on screen
when the button came up. That is the identical argument Alt (snap suspend)
already ships under, six rows above, and it is why neither needs a
per-drag memory slot that an Escape-cancelled drag could leave stale.

### The preflight, and why the shell asks before it acts

`EditSession::vertex_edit_preview` is the engine's own refusal predicate —
*"`vertex_edit_preview(id, edit).err()` is `Option<EditError>`: exactly the
narrower `*_refusal` shape a shell needs"* — and it shares **one body**
with the mutating verbs (`vertex_edit_plan`), so it cannot disagree with
them. It is asked on every frame of a count-editing drag, for the reason
the engine gives in the `adopt_widget` lesson it cites: *"a verb with no
preflight makes the UI find out by pressing."*

What it buys, concretely, is that **the preview never promises a shape the
release would refuse**. A closed perimeter with three corners cannot lose
one — [`EditError::PerimeterWouldBeDegenerate`], because two closed vertices
trace a line there and back and print twice the distance between two points
— so the preview shows the shape *unchanged* and the release records a
worded decline instead of raising an action. Silence there is exactly what
produced the operator's report.

## Item notes

### `const VERTEX_GRAB_SLACK_PT`

The drawn square is the promise; this is the target. They differ because a
7 pt square is a hard thing to hit with a mouse on a dense drawing, and the
standing convention here — stated at `handles::grip_at` — is that a grip's
live area may exceed its drawn one, never the reverse. A target smaller than
its picture is the operator missing something they can see.

### `struct CountEdit`

A struct for [`VertexFrame`]'s reason and one more: five of its ten members
are already in scope at the one call site under names that would be trivial
to transpose positionally — `index` and the two `usize`-adjacent values, and
two point-shaped things in the same space.

### `fn preview_of`

Through `dimension_preview_segments`, this module's standing rule: what is
previewed and what is committed come from one function, so a corner cannot
be shown in one place and written to another. `offset` and `text_along` are
zero because the preview draws the *shape*, and the label's placement is not
what this gesture changes.

### `fn refusal_for`

The mapping lives here rather than in `crate::text::measure` for
`app::actions::annots::refusal_for`'s reason, which is this project's
standing division: the engine's error enum is a *shell* concern, and the
catalog holds operator prose only. A `crate::text::` module that matched on
`EditError` would put the engine's vocabulary into the string catalog and
give the catalog a reason to change every time the engine adds a variant.

The `_` arm is not laziness. The remaining refusals —
`DimensionNotFound`, `DimensionGroupNotFound`, `VertexIndexOutOfRange`,
`DimensionHasNoVertices`, `DocumentEncrypted`, the certification guard and
`SidecarWrittenByNewerBuild` — are either unreachable from a handle the
painter drew from this same model, or are properties of the FILE that no
wording about corners could help with. They get the general sentence rather
than a fabricated specific one.

### `fn count_edit`

# The preflight is asked FIRST, and the preview is derived from its
answer

`vertex_edit_preview` shares one body with the mutating verb
(`EditSession::vertex_edit_plan`), so it cannot disagree with what the
release would do. Asking it before drawing anything is what makes this
gesture obey the honesty contract the label drag states at [`drag`]: **the
preview is a shape the release would commit, or it is the shape that is
already there.** A build that drew the corner vanishing and then refused on
release would be showing the operator an edit that never happens — the
worst reading of a gesture, because it looks like it worked until the next
frame repaints.

It costs one `read_dimension_model` per frame of a count-editing drag.
That is a sidecar read on a gesture that lasts a second or two and is
deliberate: the alternative is a second copy of the minimum-count rule in
this shell, which is the *"two things that must agree and eventually will
not"* the engine's own doc comment argues against by name.

# The decline is recorded on the RELEASE and not on every frame

A refused frame in flight is not an event — the operator is still holding
the button and can drop the gesture by moving off, or by letting Ctrl go.
What is an event is releasing on a refusal, and that is the one frame that
writes a sentence. Recording per frame would rewrite the same slot sixty
times a second and would fire on gestures the operator abandoned.

### `const TRACE`

An in-flight placement and a committed one are the same screenshot at the
moment of release, which is defect 8's lesson: the harness needs a line that
distinguishes *"the preview followed the pointer"* from *"the release
reached the verb"*, and no pixel can carry that.

### `fn selected`

Returns the record's id together with its geometry, because every caller
needs both and a second lookup could resolve differently after an edit.

# Why the whole model is walked rather than indexed

An annotation selection carries an *object* id — the annotation in the file
— and a dimension record carries a [`DimensionId`]. The sidecar holds the
mapping one way only (`record.annot`), so the reverse lookup is a scan. It
is a scan over the dimensions on the document, which is a handful even on a
heavily dimensioned sheet, and it runs once per press rather than per frame.

### `fn grab_box`

The annotation's `/Rect`, projected. That is the same rectangle
`canvas::overlay::draw_selection` already strokes when a dimension is
selected, which is the property that matters: **the drawn outline and the
live target are the same shape.** An operator aims at what they can see.

# Why this is not `overlay::grip_box`

### `fn placed`

Returns the placed geometry together with the two scalars `place_dimension`
takes, so the preview and the commit are derived from one calculation rather
than two that could disagree. That pairing is the point of the return type:
a caller cannot draw one placement and commit another without going out of
its way.

`None` when the delta cannot be resolved — an **angular** dimension (see the
module header: its placement is a radius and an angle, and this delta is in
points), a **circular** one (which the engine refuses outright, having no
axis to place along), or a degenerate `Aligned` linear one whose two picks
coincide and which `axis_frame` refuses rather than fabricating.

# TODO — angular placement

`place_dimension` accepts an angular dimension, taking an arc radius and a
position in degrees. Dragging one is a genuinely different calculation
(radial distance from the apex; angle subtended) rather than this one with
different names, and it needs its own preview and its own tests. Filed
rather than approximated.

### `fn drag`

Returns the **page-space segments** the dimension would be drawn as, if the
operator released now, or `None` when the drag reaches no verb.

# The honesty contract, restated because it is the same one everywhere here

The preview is `Some` if and only if a release would commit, and it is
derived from the *same* [`placed`] result the commit uses. So the operator
cannot be shown a dimension standing off by 40 points and then get one
standing off by something else — the two numbers are literally the same
`f64`.

# Rule 4, and why this preview is allowed to exist at all

Rule 4 forbids marking *applied* content as provisional. This draws
something that has not been applied yet: it is the rubber-band of a drag in
flight, which the rule names explicitly as a pre-commit affordance — *"a
snap indicator, a hover highlight, a rubber-band … these are the cursor"*.
It disappears on release, and what replaces it is the annotation itself,
rendered by `pdfcer-render` with no marking of any kind.

### `const VERTEX_HANDLE_PT`

The same 7 pt the Bézier handles use (`canvas::handles::GRIP_SIZE_PX`'s
neighbourhood), because they are the same affordance to a hand: a small
square you grab. Zoom-invariant — it is a screen-space control, so it does
not grow with magnification, and a corner on a plan at 20 % is as grabbable
as one at 400 %.

### `fn vertices`

Empty for every other selection and for every other dimension kind, which is
what makes both the painter and the hit test one call with no branch of
their own.

### `fn vertex_at`

# The comparison is in SCREEN space, and that is the whole of why this
function converts rather than the caller

A handle is a screen-space affordance of a fixed size. Comparing in canvas
or page space would make the target shrink as the operator zooms out —
exactly when a plan's corners are closest together and precision matters
most — and balloon as they zoom in, so that at 800 % a press anywhere near a
corner would grab it. The conversion has to happen on the side of the
boundary where the tolerance is meaningful, and that is here.

# Ties go to the LAST vertex, deliberately

Two coincident vertices are legal — [`super::measure::perimeter`] does not
de-duplicate, on the argument that a repeated point is invisible rather than
wrong. If the operator has made one and wants it gone, the one they can
reach is the one they can drag away, and the later index is the one they
just placed.

### `fn drag_vertex`

Returns the page-space segments the shape would be drawn as if the operator
released now, or `None` when the drag reaches no verb.

# This one RE-MEASURES, and that is the difference from every other
gesture in this module

[`drag`] writes `offset` and `text_along` — two fields the value function
does not read — so no label drag can alter the printed number. This one
moves a corner of the measured shape, so it changes the number **by
design**. The engine says so plainly: `move_dimension_vertex` is *"the first
ce-dimension verb that deliberately changes what a ce dimension measures"*.

The consequence for this shell is a disclosure obligation the label drag does
not have. `VertexOutcome` carries `previous_label` and `label` precisely
because **the old value cannot be reconstructed afterwards** — the geometry
it came from is gone — and a status line reading `12.40 m → 13.85 m` is a
disclosure where one reading `13.85 m` is just the number the operator can
already see on the page.

# No guard, no probe, no first-move check — the engine ruled on it

I asked whether the verb could refuse mid-drag, so that the preview could be
withheld. The answer was that it cannot, and it was a ruling rather than an
omission: a self-intersecting polyline has a perfectly well-defined total
length (a figure-eight is a real fence run), and a zero-length segment
contributes 0.0 and disappears the moment the vertex moves again. Every
remaining refusal is structural and knowable before the drag begins.

**⇒ Draw the preview. Always.**

### `struct VertexFrame`

A struct rather than eleven parameters — `canvas::resizing::Frame`'s own
argument, and this one crossed clippy's arity limit the moment snapping
arrived. Three of the members are `Option`s of borrowed things and two are
`Pos2`s in the same space, both of which a positional list would let a
caller swap silently.

### `fn intent`

# The Points tool is a gate and not a shortcut

With any other tool armed this returns [`VertexIntent::Move`] whatever is
held, so a Ctrl that the operator was using for something else — subtracting
from a selection, say, which is what `canvas::marquee::Combine` spells it as
everywhere else on this canvas — cannot destroy a corner of a shape they
were merely nudging. Changing how many corners a measured shape has is a
deliberate act and it costs one deliberate arming.

It is also what gives the Points tool a subject in a mode that cannot edit
page content. See [`crate::canvas::tool::retire_forbidden`]'s Node arm.

# Why `command` and not `ctrl`

On Windows they are the same key and on macOS `command` carries this
meaning; `canvas::interact` reads the same field for the same reason and
says so at its own call site.

### `struct VertexDrag`

Two fields rather than one, because the preview and the snap indicator are
different pictures with different lifetimes: the polyline is drawn in page
space through the dimension painter, and the marker is a screen-space glyph
at the candidate. Folding them would make the caller unpack a tuple whose
members it uses in two different places, forty lines apart.

### `fn annot_shapes`

# Why this exists: a bounding box is not a shape

A click selects what is under the cursor, not what merely encompasses it.
A ce dimension's `/Rect` is the box around two witness lines, a dimension
line, two arrowheads and a label — mostly empty air for anything but a
perfectly horizontal one, and for a perimeter traced round a building it is
the entire footprint. Hit-testing that box meant the operator could not
select the drawing underneath their own dimensions.

The segments come from `measure::pick::dimension_preview_segments` — **the
same function the dimension is previewed and drawn from**. That is this
module's standing rule applied to hit testing: what is clickable and what is
visible are one derivation, so they cannot drift apart. A second "where is
the ink" calculation would be a second thing to keep right.

# What it costs, and why it is per-click rather than cached

One pass over the sidecar's dimension records, projecting each one's
segments. A heavily dimensioned sheet carries tens of these, not thousands —
the 129,758 objects on the benchmark drawing are page CONTENT, and none of
them is here. It runs on a click, not on a frame.

The label is deliberately NOT included. It is drawn by `pdfcer-render` from
the appearance stream and this shell does not know its box; a dimension is
selected by its lines, which is the part an operator points at. If that
proves too strict in use, the fix is to ask the engine for the label's box
rather than to guess one here.
