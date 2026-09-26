# `ui-verify/checks/markup_move`

`dragging_a_markup_moves_it` — **draw a shape, drag it, and it is somewhere
else.**

# What this is for

`FEATURES.md` carried this under the Format contextual tab:

> *"In `pdfcer-gui` a placed markup can be selected and deleted but not moved
> or resized yet."*


## Why the failure it guards is worse than "the drag does nothing"

Before this, an annotation drag was **consumed and discarded**.
`canvas::interact` forks on *"is an annotation selected?"* and the only
module on that branch answered for ce dimensions — so a stamp took the
branch, got `None`, and the content branch that does move things was
unreachable by construction. The operator pressed inside a shape, dragged it
across the sheet, released, and it was where it started, with no message
anywhere.

⇒ A fork whose branches can **both** answer *"not mine"* is worse than a
missing feature: the gesture is eaten. This check exists at the fork.

## What this check CANNOT see, stated first because it is the point

A move has two halves and **only one of them shows up in a render**:
`/Rect` moves the painted result for free, while `/L`, `/Vertices`,
`/InkList` and `/QuadPoints` hold **absolute page coordinates** and are what
any *other* tool regenerates an appearance from. Write only the first and
the annotation looks right here, right in a screenshot, right in a pixel
check — and is rebuilt **in its old place** by the next viewer that rebuilds
it.

**So a pixel oracle is the wrong instrument for this feature**, and that
is not a limitation of this check but a fact about the format. Every
screenshot this project can take reads the appearance stream. The trace line
is the only place the second half is visible from here, through
`keys=` — the count of geometry keys the engine found and moved.

`keys=0` is **correct** for a Stamp, a Text note or a Link, whose `/Rect`
*is* their geometry. It is asserted as *reported*, never as non-zero, for
exactly that reason — and the shape drawn here is a `/Square`, which the
engine does not list among the geometry-key subtypes either. What this check
establishes is that the chain reaches the engine's own two-halves
implementation; the engine's tests establish that both halves are written.

## The chain, and every link that can break silently

| # | link | owner | a unit test can see it? |
|---|---|---|---|
| 1 | `markup.rectangle` arms the tool | shell | yes |
| 2 | a drag authors a `/Square` | `canvas::markup` | yes |
| 3 | a click on it **selects** it | `canvas::selection::annot` | yes |
| 4 | a second drag reaches `annotdrag` rather than `dimdrag` | `canvas::interact` | **no** — the fork is wiring |
| 5 | the action reaches `move_annotation` | `app::actions::annots` | yes |

Link 4 is the one this check is for, and it is the one that was broken.

## Item notes

### `const INVOKE`

`mode.review` because markup is authored there, and driving from a named
mode makes the run reproducible rather than dependent on whatever mode the
last session left behind.

### `const MOVED_EVENT`

`-applied`, per the convention this project adopted after making the
same-name mistake twice: `vector_edit` writes its own `move-annotation …`
line for the identical edit, and `.last()` on the bare name reads that one.

### `const PAGE_REGION`

`page`, not `canvas`. `canvas` is the name of a **trace event** in the
profile's vocabulary — the line carrying the view's rect and zoom — and it
is not a `ui-rect` region at all. Asking `declared()` for it answers `None`
on a perfectly healthy build, which is a check reporting the program broken
because the harness looked in the wrong dictionary. The regions this
application publishes for the sheet are `canvas-viewport`, `central-panel`
and `page`; the last is the one that means *a document is on screen*.

### `const SHAPE`

Well inside the sheet and away from the title block on a real drawing, so
the click that selects it in step 3 cannot land on page content instead —
and away from the edges, so the move in step 4 has somewhere to go.

### `const MOVE_TO`

A displacement in **both** axes, deliberately. A move that only travels in
x would pass on a build that dropped `dy` — and `dy` is the one with a sign
convention to get wrong, because PDF user space increases **upward** while
every screen coordinate here increases downward.
