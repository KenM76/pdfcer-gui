# `canvas::selection` — the selection STATE, and the invariant it exists to hold

## The two halves of this module, and the seam between them

[`identity`] holds what a selection **is**: [`Selection`],
[`SelectionLevel`], [`ClickHit`] and [`EscapeOutcome`] — four `Copy` types
which between them cannot name a place on the screen. That file carries the
*"selection is an identity, not a position"* argument in full, because it is
an argument about the shape of a **type** and is answered by reading four
field declarations.

This file holds what **accumulates** them: [`SelectionState`], the ladder it
walks, the `(page, epoch)`-keyed re-resolve, and every rule about what a
click, a marquee or an Escape means. Those are answered by reading
behaviour, which is why they are worth their own file and their own tests.

All four identity types are re-exported here, so
`crate::canvas::selection::SelectionLevel` remains the path every caller
uses and the seam is invisible from outside the module.

## The invariant, stated first because everything here is shaped by it

`GUI_ROADMAP.md` Phase 1, from the operator's own words:

> *"if I select a node or something for a tool, I should be able to pan
> and zoom out without losing my first selection."*
>
> **Navigation is not an edit. Panning, zooming, changing fit mode,
> rotating the view, switching page-display mode and changing ribbon tab
> must never alter the selection.**

The roadmap names **three** ways the natural implementation loses it, each
of which looks reasonable in isolation. This module closes all three, and
each closure is a structural property rather than a promise:

**1. Selection stored in screen coordinates.** Zoom changes the mapping, so
the stored point stops naming the thing it named. ⇒ [`Selection`] holds **no
coordinate of any kind**. It is `page + object + subpath + node`, four
integers, none of which a zoom can touch, and there is no constructor that
takes a `Pos2`. This is the one closure that is a property of a *type*
rather than of a method, which is why it lives in [`identity`] — see that
file's header for the argument in full.

**2. Selection cleared by a click that was really a drag.** A gesture begins
with a press; if press-on-empty clears, every drag that starts on blank
paper destroys the selection. ⇒ Nothing in this module is called on a press.
The clear is driven by [`SelectionState::click`], which
[`crate::canvas::gesture`] raises only for a **completed click with no
drag**.

**3. Selection invalidated by re-decomposition.** The provider rebuilds on
page change and on edit; a rebuild triggered by zoom, or by a page change
that is not a page change in the operator's sense, must not drop it. ⇒
[`SelectionState::resolve`] **re-resolves against the new decomposition**
instead of discarding, and — the part that is easy to get wrong — it only
validates entries **on the page the provider serves**. An entry for another
page is left completely alone.

Row 3's second half is the one that makes the acceptance criterion pass:
*"select a node, zoom out three rungs, pan across the sheet, switch to
Continuous, come back, switch ribbon tab — the node is still selected and
still the entered level."* Going to another page builds a provider for
that page, and a `resolve` that pruned everything it could not find would
wipe the selection on the way past. Coming back would find nothing.

## Why the level is state and not derived

[`SelectionLevel`] could be inferred from whether `subpath`/`node` are
`Some`. It is stored instead, because *"inside this object, nothing picked
yet"* is a real state — reached by entering an object at a point where no
subpath was close enough — and an inferred level would collapse it into
"not inside anything at all". The operator would then find Escape taking
two presses on one path and one on another, for no reason they could see.

## What this module deliberately does NOT do

It never draws, never touches egui, never reads a pointer, and never
reaches a document. It is a state machine over four integers and a
provider trait, which is precisely why every invariant above can be
asserted in a unit test rather than hoped for in a running window.

## Item notes

### `fn outline_rect`

Falling back to the object's box when a part has no bounds is
deliberate — the alternative is drawing nothing for a selection that
exists, and a correct action with no feedback is indistinguishable
from a broken one.

### `fn click_at_object_rung`

# The second click on a text block narrows to the chunk under it

`OPERATOR_REQUESTS.md` **O215** ask 1, in his words: *"sometimes it moves
the chunk and sometimes it takes the whole block."* Selection has always
descended to a chunk — [`Self::select_part`] and the run menu address one
— but the ladder's own descent gesture is a double-click, and on text
that gesture is spent opening the caret. So the rung had no entrance a
hand could find, and which unit a drag picked up was decided on **press**
by [`crate::canvas::presspick::covers`] against a rectangle nothing drew.

The narrowing is offered on a **plain click**, and exactly three
conditions have to hold at once:

| condition | what it stops |
|---|---|
| `hit.chunk` | a path's subpath, and a text block whose boxes are switched off or which holds one line — narrowing to a unit with no box drawn around it is the same invisible aim, from the other side |
| this object is **already** the whole selection | a click on a different block, which selects it whole |
| `hit.part` is `Some` | a click in the white inside the block's box, which keeps the block |

*"Already the whole selection"* is not enough on its own, and the
hazard is the reason [`crate::canvas::presspick::changed_selection`]
exists: the press of this very click may be what selected the block, and
the state it leaves is identical. The click path folds that answer into
`hit.chunk`, so the first click of a gesture selects the block and puts
the boxes up, and the second — with somewhere visible to aim — takes one
chunk.

Everything below the Object rung is [`Self::click_inside`]'s, which
already re-picks a part on every click, so once here the gesture is
repeatable by a route that was always there.

### `fn click_inside`

Three outcomes, in precedence order: re-pick at the current rung; fall
back one rung and re-pick there; or leave the object entirely and
behave like an ordinary Object-rung click. The middle case is what
stops an operator being stranded at a rung whose targets they keep
missing — at the Node rung, a click that misses every anchor but lands
on a part ascends to that part rather than doing nothing.

### `fn descend`

A double-click on a **different** object enters that object rather
than descending inside the current one: PDF path objects do not nest,
so carrying a part or node index across would address an index in a
different object's space.

### `fn normalise`

1. **Entries are ordered and unique.** Document order, so the outlines
   paint in a stable sequence rather than re-stacking on every
   shift-click; unique, so a batched edit is handed a clean operand
   list.
2. **A rung above `Object` means exactly one object is entered.** A
   rung is a place *inside one thing*, and [`Self::entered_object`]
   derives that from the first entry. Anything that would leave the
   two disagreeing collapses to the Object rung instead — recovering
   is better than asserting, because the state is reachable from a
   marquee arriving while inside an object and the honest response is
   to step out.
