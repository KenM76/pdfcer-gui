# `canvas::annotnodes::menu` — **the right-click route to a shape's nodes**

## The operator's report, and the half of it that was still open

> *"I also can't edit or delete nodes of a markup shape once it is drawn."*


> The natural way to add or remove a corner is a **right-click on the
> shape** — *"add a point here"*, *"remove this point"* — which is how the
> engine itself describes these two operations. The chords above are a
> stopgap. The right-click menu lives in a file another job was working in
> today, so it is written down here rather than half-built.

## Why the menu route needs no armed tool, and the chord route does

It looks like an inconsistency and it is a rule applied at its own edge.

The **chord** route requires the Points tool armed because `Ctrl` already
means *take this out of the selection* everywhere else on this canvas. A
bare `Ctrl`-press on a node is therefore ambiguous by construction, and
arming the tool whose whole subject is nodes is what disambiguates it: the
operator has said, in advance, *"I am working on points now."*

A **menu row is unambiguous by construction.** The operator read the words
*"Add a point here"* and chose them. There is no second reading to rule out,
so requiring an armed tool first would be carrying a rule past the reason
that produced it — and it would put a mode between the operator and a verb
they had already named, which is the shape of every complaint in
`OPERATOR_REQUESTS.md` about this canvas.

⇒ Recorded here rather than at the call site because it is the kind of
asymmetry a later reader "fixes".

## The engine is ASKED, never restated

[`super`]'s header carries the matrix — which subtype accepts a move, an
insert, a remove, and what its floor is. **This module does not read that
table.** It builds the exact [`super::Plan`] the row would commit — a
`VertexEdit` for a `/Polygon`, `/PolyLine` or `/Line`, an `InkEdit` for an
`/Ink` (`Pass 278.0`) — and hands it to that family's preview verb, which
shares one body with the mutating verb and therefore cannot disagree with
what pressing the row would do.

What the answer is used for is the R9 decision, and the **error variant is
what decides it**:

| preview says | row | why |
|---|---|---|
| `Ok` | drawn, live | it would work |
| `Err(ReshapeWouldBreachVertexFloor)` | drawn, **greyed**, tooltip explains | *temporarily* unavailable — draw another corner and it comes back |
| `Err(InkStrokeWouldBreachPointFloor)` | drawn, **greyed**, tooltip explains | the same floor, **per stroke** of a freehand mark — add a point to that stroke and it comes back |
| any other `Err` | **absent** | a property of the shape's kind, which will not change while the operator looks at it |

That is R9 exactly — *"an unavailable capability renders nothing; greying is
reserved for temporarily unavailable and is always explained on hover"* —
and it is derived rather than declared. A `/Line` gets no *Add a point here*
because the engine says `GeometryNotReshapable`, not because this file holds
a list of subtypes; the day the engine teaches `/Line` to grow a third
point, the row appears with nothing here edited. And that is precisely
what happened to `/Ink` on 2026-09-09: the engine grew the verbs, [`super`]
grew one `match` arm, and the two rows appeared on a freehand mark with this
file's *decision* unchanged — only its *addressing* grew a second family.

## The operand problem, and where it is parked

A menu row carries **a command id and nothing else**
(`egui_shell::manifest::Item::Command`). *"Add a point here"* needs to know
**which segment**, and *"Remove this point"* needs to know **which node** —
facts that exist only at the instant of the secondary click, on a surface
the dispatcher never sees.

So the pick is **parked for the life of the popup**, in `egui::Memory`,
beside the one thing already parked there for the same reason: which of the
canvas menus is open (`canvas::menus`' `MENU_MEMORY_KEY`). The two have the
identical lifetime and the identical argument —

> `egui` opens a popup ON the secondary click and draws it on every
> subsequent frame until it is dismissed. The pointer moves during those
> frames — onto the menu itself, which is not over the shape any more — so
> recomputing from the live pointer would swap the row's meaning out from
> under the operator's hand while they were reading it.

⇒ [`park`] is called once, on the click. [`parked`] is read on every frame
the menu is drawn (to decide the two rows' states) **and** again when the
command is dispatched (to build the action). One value, three readers, no
second derivation of *"which corner did they mean"*.

`Memory` rather than `PdfcerApp` state for `MENU_MEMORY_KEY`'s stated
reason: this is frame-local interaction state with no meaning across a
document, `Memory` is per-`egui::Context`, and a document change therefore
starts the next frame with no pick and no popup in flight.

## Rule 15

Everything here is about a **markup shape**. A **ce dimension** is also a
`/Line`, its nodes are `canvas::dimdrag`'s, and its verb is
`move_dimension_vertex`, which **re-measures** — a different operation with
a different undo entry and a different disclosure. [`super::geometry`] is
the one gate, and it refuses anything that is not
[`AnnotKind::Markup`](crate::canvas::selection::AnnotKind::Markup) before
this module sees it. The engine refuses again by name
(`EditError::AnnotationIsCeDimension`) as the backstop; nothing here routes
a dimension anywhere.

## Rule 4 / R8b

Nothing in this module paints. It answers three questions — *which node or
segment is under the pointer*, *what would the engine allow*, *what action
does the row raise* — and every one of them is about the cursor rather than
about the document.

## Item notes

### `const PICK_MEMORY_KEY`

One `Id`, per `egui::Context`, for the module header's reason: the pick is
taken at the click and read on every frame the popup is drawn, so it has to
outlive the click and must not outlive the session.

### `const SEGMENT_SLACK_PT`

Wider than [`super::NODE_GRAB_SLACK_PT`]'s companion tolerance, and
deliberately: a node is a drawn 7 pt square the operator can aim at, and a
segment is a hairline they cannot. The standing convention at
`handles::grip_at` — *a grip's live area may exceed its drawn one and never
the reverse* — is about drawn affordances; a segment has no drawn affordance
at all, so the number is chosen from what a hand can hold steady rather than
from a picture.

It is not so wide that it swallows the nodes: [`pick_at`] asks for a node
**first**, so a click near a corner is a corner even though it is also near
two segments. See that function's precedence note.

### `fn distance_to_segment`

Clamped to the ends, which is the C5 convention `canvas::selection::annot`
states: without the clamp a short segment would claim a stripe across the
sheet, and the insertion parameter could land off the end of the edge the
operator pointed at.

A degenerate segment (two coincident nodes, which `/Vertices` permits and
the engine does not de-duplicate) answers `t = 0.0` and the distance to that
point, so it behaves as the single point it is drawn as.

### `fn a_click_beside_a_segment_projects_onto_it`

Falsified: returning `t = 0.0` instead of the projection makes the
midpoint assertion fail on both coordinates, and returning the pointer
itself makes the y assertion fail by the 4 pt offset below.

### `fn a_greyed_row_is_drawn_and_an_absent_one_is_not`

Falsified: defining `shown` as `matches!(self, Self::Live)` makes the
greyed assertion fail, which is the exact regression — an unavailable
row disappearing instead of explaining itself.

### `fn the_default_pick_names_no_node`

Falsified: making `Node(0)` the default makes this fail, and would
have offered *Remove this point* on the first vertex of every shape
before the operator had pointed at anything.

### `const TRACE_MENU`

Distinct from [`super::TRACE_MOVE`] and its three siblings, which report a
gesture that **changed the document**. This reports a *question being
opened*, and a check that read only one of the two could not tell a menu
that offered the wrong row from a command that acted on the wrong node.

### `const TRACE_COMMAND`

It carries the **pick**, not just the command id, because the whole class
of defect this design can produce is *the right verb on the wrong corner*.
A trace line has to carry the number a wrong build would get wrong.

### `enum NodePick`

Three answers and no `Option`, because "the pointer was nowhere near the
shape" is a real state a menu has to render (both node rows absent, the rest
of the markup menu intact) rather than an error, and an `Option<Pick>` would
let a caller `unwrap_or_default` its way into treating it as node 0.

### `struct Rows`

Both default to [`RowState::Absent`], which is the honest answer for every
selection that is not a reshapable markup shape and for a pointer that was
nowhere near one: the rows are not drawn, and the rest of the markup menu —
properties, the clipboard, delete — opens exactly as it would have.

### `fn pick_at`

# Precedence: a node beats a segment, always

Every node lies on two segments, so within [`SEGMENT_SLACK_PT`] of a corner
both answers are true and exactly one can be offered. The corner wins,
because *"remove this point"* names a thing the operator can see and *"add a
point here"* would insert a duplicate one grid-unit from a node they were
plainly aiming at. It is also the safer error: an unwanted removal is one
`Ctrl+Z`, and an unwanted insertion leaves a shape that looks unchanged
with an extra vertex nobody can find.

# The comparison is in SCREEN space

[`super::node_at`]'s argument, unchanged and for the same reason: a
tolerance in canvas or page space would shrink as the operator zooms out —
exactly when a shape's segments are closest together — and balloon as they
zoom in, so that at 800 % a click anywhere near a shape would claim one of
its edges.

### `fn rows`

Asked of the shape's family's preview verb, per frame, with the exact
[`super::Plan`] the row would commit. See the module header for why the
*error variant* is what separates a greyed row from an absent one.

The cost is one annotation walk per row per frame, and only while the
popup is open — [`crate::canvas::menus`] calls this from inside its own
"is a markup menu the one being drawn" branch. [`super`]'s header records
the standing engine advice this obeys: *"ask the preview verb every frame
rather than catching the error afterwards — a verb with no preflight makes
the UI find out by pressing."*

### `fn trace`

Called by [`crate::canvas::menus`] on the frame of the secondary click, so a
driven check can read *which corner the menu thinks it is about* without
pressing anything — which is the only way to tell "the row was greyed
correctly" from "the row was greyed because the pick was wrong".

### `fn action_for`

# Why the guard is re-asked here and not trusted from the menu

The row was drawn from [`rows`] on some earlier frame, and everything
between then and now is a frame in which the document could have changed —
an undo, a background reflow, another surface's edit. Re-asking
[`rows`] with the parked pick costs one annotation walk on a press and
removes the whole class of *"the menu was right when it was drawn"*, which
is the class `MenuHost::with_condition`'s own header is about in the
opposite direction.

⇒ So a row that has stopped being live raises **nothing**, and the trace
says which. It does not raise a refusal sentence: the operator pressed a row
that the engine now declines, which is the state R9 grey already describes,
and a status line arriving after a menu closed would be a second explanation
for a first-order rarity.
