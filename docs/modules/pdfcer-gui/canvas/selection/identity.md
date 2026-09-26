# `canvas::selection::identity` — a selection is four integers, never a position

## Which half of `canvas::selection` this file is

[The parent module](super) holds the mutable state that *accumulates*
selections — [`SelectionState`](super::SelectionState), the ladder it walks,
the `(page, epoch)`-keyed re-resolve, and every rule about what a click
means. This file holds the **vocabulary that state is made of**:
[`Selection`], [`SelectionLevel`], [`ClickHit`] and [`EscapeOutcome`].

Every type here is `Copy`, none of them owns anything, none of them has a
method that mutates, and — the point of the split — **none of them can name
a place on the screen**. The parent re-exports all four, so
`crate::canvas::selection::SelectionLevel` is still the path every caller
uses; the file boundary is for the reader, not for the type system.

## Selection is an identity, not a position


> *"if I select a node or something for a tool, I should be able to pan
> and zoom out without losing my first selection."*
>
> **Navigation is not an edit. Panning, zooming, changing fit mode,
> rotating the view, switching page-display mode and changing ribbon tab
> must never alter the selection.**

The roadmap names **three** ways the natural implementation loses that, each
of which looks reasonable in isolation; [the parent module](super)'s header
tabulates all three against what closes them. **The first is closed here,
and it is the only one of the three that is closed by construction rather
than by behaviour:**

> **Selection stored in screen coordinates.** Zoom changes the mapping, so
> the stored point stops naming the thing it named.

[`Selection`] holds **no coordinate of any kind**. It is
`page + object + subpath + node`, four integers, none of which a zoom can
touch. There is no constructor that takes a `Pos2`.

Having that as a property of a *type* rather than as a rule somebody
maintains is the whole reason these four declarations are worth a file of
their own. A method can be edited to keep a cached point "in step" with the
view and still read as reasonable in review — that is precisely how the
defect arrives. A field that does not exist cannot be kept in step with
anything. Everything that could go wrong with the selection's *shape* is
therefore visible on one screen, and the other two failure modes — a press
that turned out to be a drag, and a re-decomposition that discards instead
of re-resolving — are behavioural, so they are closed by
[`SelectionState`](super::SelectionState) and argued there.

## Why paint-order index is the identity, and what it does not survive

`Selection::object` is a [`TargetId`], which is the object's index into
`PageObjects::objects` — **paint order**. It is the same number
`pdfcer object-list` prints and `object-delete` takes, so "object 412"
means one thing across every surface. That is what makes it usable as an
identity here.

It is an identity **within one revision of one page**, and no further.
Deleting an object renumbers every object painted after it. So an edit
moves the meaning of a retained index, and this module's honest position
is to say so rather than to pretend otherwise:

- A rebuild with the **same** revision (the zoom / page-return / panel
  case, which is the invariant's whole subject) re-resolves exactly.
- A rebuild after an **edit** re-resolves against the new decomposition
  and drops what no longer exists; indices that *shifted* silently name
  their new neighbour. Closing that needs a stable per-object token from
  `pdfcer-core`, which does not exist — `decompose_page` mints indices, not
  identities. It is recorded here rather than in a comment nobody reads,
  and it is a boundary finding for the engine, not a shortcut taken here.

## What this file deliberately does NOT do

It never draws, never touches egui, never reads a pointer, and never
reaches a document — and beyond that it does not *decide* anything. There is
no rule here about what a click means, when a rung is entered, or what
Escape leaves behind: four plain data types and one three-line
[`SelectionLevel::ascend`]. Every behavioural question lives one level up,
which is what lets that level be read as a state machine rather than as a
state machine tangled with its own data definitions.
