# `canvas::vertexroute` — **which of TWO node-edit verb families one drag
reaches**


> One gesture — press on the thing, drag it — reaches different engine verbs,
> and **which one is decided entirely by what is selected**.

## The two families

| selection | verbs | identity | preflight |
|---|---|---|---|
| a **ce dimension** | `move_dimension_vertex`, `insert_dimension_vertex`, `remove_dimension_vertex` | a sidecar `DimensionId` | `vertex_edit_preview` (count edits only) |
| a **markup shape** | `reshape_annotation` and its three wrappers | a stable `ObjId` | `reshape_annotation_preview` (**every** edit) |

R8b rule 15 is enforced by the type here rather than by care: a **ce
dimension** is the thing pdfcer authors and measures with, a **markup
shape** is a comment somebody drew, and [`Subject`] makes the two a `match`
the compiler checks. **pdf dimensions** — CAD page content — are neither and
are nowhere near this module; a content path's anchors are
[`super::handledrag`]'s.

## Why they are two variants rather than one call with a flag

Because the one thing that must never happen on this canvas is a gesture
aimed at the wrong verb, and a ce dimension arriving at
`reshape_annotation` is precisely that: it is a `/Line` with
`/IT /LineDimension`, so it passes every *"is this markup?"* test, and the
engine refuses it by name (`EditError::AnnotationIsCeDimension`) as the
**backstop** rather than as the mechanism. `canvas::selection::AnnotKind`
exists to make the routing decidable before the engine is asked, and this
module is where that decision is spent.

## What is applied here, above both branches

**Shift**, and **Alt**, once each:

* `ui-conventions/drag-moves.md` D5 — Shift locks the node to one axis,
  through [`super::constrain::reposition`], which filters the displacement
  **from the press** so the grab point survives (D8). Two copies of *"what
  does Shift mean"* is how two node drags in one program come to disagree
  about it.
* D6 — Alt suspends the snap, read live, and asked of the same
  `snap_query_enabled` a measure pick asks. It is what makes a generous
  catch radius affordable: the offer is refusable, so it can afford to be
  eager.

## What this module does NOT decide

* **Whether the press is a node drag at all.** `canvas::pressing` resolves
  which node, and `canvas::gesture::press_kind` decides what the press
  means.
* **Whether an edit is allowed.** Each branch asks its own engine preflight,
  and neither restates the engine's rules.
* **What the preview looks like.** Each branch derives it from the same
  geometry its release commits.
