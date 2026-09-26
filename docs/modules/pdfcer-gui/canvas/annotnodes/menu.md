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
