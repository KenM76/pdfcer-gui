# `ui-verify/checks/dimension_corner_count`

`a_corner_can_be_added_and_taken_away` — the operator's *"I also can't edit
or delete nodes of a markup shape once it is drawn"*, for the one shape
where the engine can do it.



> *the selected shape published no `canvas.dimension-vertex.1`, so there is
> no corner to aim at … Regions seen: none.*

**The application was blameless.** The step went straight from the click
that CLOSES the perimeter to the click that was meant to SELECT it, with
the Perimeter tool still armed — so the second click was taken as the first
vertex of a new shape. The trace says it in two lines, `add-dimension
page=0 n=1` followed by `measure-perimeter-vertex n=1`, and no painter was
ever asked for a handle. Putting the pen down first (step 3 below) makes
the whole check pass: `4 → 5 → 4` corners, both engine verbs reached.

**The lesson is about the negative assertion, not about the tool.**
*"Regions seen: none"* was a true statement that could not distinguish
*"the handles are not drawn"* from *"nothing was ever selected"* — an
absence is only evidence when the thing that would produce the presence is
known to have been attempted. The message at that step now says which
trace line separates the two.

**And its preconditions were the half that did not get copied.** Both
siblings this step was modelled on — `measure_perimeter` and
`markup_node_edit` — put the pen down before selecting, and
`markup_node_edit`'s comment names *this check's* first run as the reason
it does. The warning was in the tree before the failure happened; the check
that needed it was not reading it. **A step copied from a passing sibling
must copy its preconditions, not only its clicks.**

## What it is for

`pdfcer-core` has had three vertex verbs since `Pass 107.0` and this shell
called exactly one of them:


So a corner could be dragged and the number of corners could not change.
`canvas::dimdrag::count_edit` now reaches both, on a Ctrl-drag and a
Ctrl+Shift-drag from a corner handle **with the Points tool armed**, and
this check is the only instrument that can say whether that arrives.

## Why every link here needs a running window

`canvas::dimdrag::tests` already asserts the arithmetic, the preflight and
the tool gate without a window — and all of it would pass on a build where
the operator can do none of this. Six things stand between those tests and
the operator's hand, and not one is observable in-process:

| # | link | why a unit test cannot see it |
|---|---|---|
| 1 | the Points tool **arms in Review** | `Capabilities::for_mode` reads the real manifest, and the mode is entered by clicking a segment |
| 2 | `A` reaches `view.tool_node` in Review | a chord is filtered by **tab** visibility, and the ribbon item is `shown_when("mode.edit_content")` — so the chord is the ONLY route and nothing in-process exercises it |
| 3 | Ctrl survives the OS → winit → egui path during a drag | `press_held`'s own note: a modifier that goes down and up inside one frame's event batch is applied and undone before the event that was meant to carry it |
| 4 | the press on the handle classifies as `DragKind::DimensionVertex` | `canvas::pressing` resolves it from a screen position through two coordinate spaces |
| 5 | `count_edit` is reached with a `session` that holds the record | the sidecar is read from the real `EditSession`, not a fixture struct |
| 6 | the engine accepts it and the annotation is regenerated | `insert-dimension-vertex` is the funnel's own line, and only a real edit writes it |

Link 2 is the one most likely to be the reason this check fails first,
and it is worth reading before diagnosing anything else. The ribbon and rail
items for `view.tool_node` both carry `shown_when("mode.edit_content")` in
`shell::manifest`, which another track owned on the day this was written.
**In Review the tool is reachable by the `A` chord and by nothing else.** A
failure at the arming step is therefore a manifest gap rather than a canvas
one, and the message at that step says so.

## What it deliberately does NOT assert

**That the drawing changed shape on screen.** A screenshot is captured as an
artifact for a human, and the assertions read the trace, for
`measure_perimeter`'s stated reason: asserting on accent pixels needs the
theme's colour and the polyline's route through the canvas transform, which
is a second derivation of the thing under test.

**The refusal path** — a closed triangle that cannot lose a corner. That is
asserted without a window in `canvas::dimdrag::tests`, against the engine's
real predicate, and driving it would need a second traced shape with three
corners for a fact the engine owns. What is worth driving is the path that
crosses six process boundaries, which is the one below.

## The shape it traces, and why four corners

A closed square. Four is one more than the minimum for a ring, which is what
makes **both** halves of this check legal on the same shape: the insert
takes it to five and the remove takes it back to four, so a single traced
shape exercises the two verbs in sequence and the corner count is a running
number a reader can check by eye — `4 → 5 → 4`.

## Item notes

### `const TOOL_DECLINED`

**The Points tool's arming is asserted as the ABSENCE of this line, and
that is forced by the instrument rather than chosen.** `canvas::tool::select`
writes no trace at all — it is a two-line memory write, and the four tools
that DO trace (`text-tool`, `markup-tool`, `measure-tool`,
`text-edit-tool`) each do it from their own toggle rather than from
`select`. So there is no positive line saying *the Points tool is now armed*.


⇒ The absence is a weak signal on its own and it is **not carrying the
assertion alone**. The positive proof is downstream and total:
`canvas::dimdrag::intent` returns `Move` unless
`canvas::tool::active(ctx).is_node()`, so a `dimension-vertex-insert` line
**cannot** be produced by a build where the tool did not arm. Step 5 is
therefore the real evidence, and this step exists to make a failure legible
— without it a build with the old gate would fail at step 5 with a message
about modifiers.

Reported rather than worked around: a one-line trace in
`canvas::tool::select` would make this a positive assertion and would serve
every future tool check. The session that wrote this owned only that file's
Node capability arm.

### `const ENGINE_INSERT`

Distinct from [`SHELL_INSERT`] deliberately, and the distinction is the
whole reason both are asserted: one says the gesture was understood and the
other says the document changed. A check that read only the first could not
tell a shell that never asked from an engine that refused — which is
`measure_perimeter`'s own note about `dimension-vertex` versus
`move-dimension-vertex`, and the reason `check-trace-names.py` exists.

### `fn drag_holding`

**The modifier is held ACROSS the press, the walk and the release, and
that is not politeness.** `Driver::press_held`'s own note records the
finding: a modifier that goes down and up inside one frame's event batch can
be applied and undone before the event it was meant to carry is dispatched,
because modifier state reaches egui through winit's `ModifiersChanged`. A
harness that pressed Ctrl just before the button would produce a plain drag
and report *"the corner was moved, not added"* about a perfectly working
build.

It also matches what `canvas::dimdrag::intent` actually does: it reads the
modifiers **live on every frame**, so a Ctrl released half way through the
walk turns the gesture back into a move — deliberately, and visibly, because
the preview follows. Holding it throughout is the only way to drive the
gesture the operator's hand makes.

Written here rather than on `Driver` because `Driver::drag_via` already
takes a single `Option<Key>` modifier and this needs two; widening that
signature is a change to a shared instrument, and the session that wrote
this check did not own `tools/ui-verify/src/input.rs`.
