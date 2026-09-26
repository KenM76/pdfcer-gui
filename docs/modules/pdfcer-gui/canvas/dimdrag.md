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
