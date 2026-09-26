# canvas — the page on screen, what is selected on it, and the gestures that move both

The one place a rasterized page is drawn and the one place canvas input is
read. The navigation gestures — **wheel to scroll, Ctrl+wheel to zoom about
the cursor, middle-drag to pan**, and from Phase 3 the **hand tool with
space-to-pan**, **anchored discrete zoom**, **zoom to selection** and
**marquee zoom to region** — and, from stage S4, the **selection model**:
click, Shift+click, double-click to descend, Escape to ascend, rubber-band
marquee, eight grips plus move, **dragging a selection to move it**, and
Delete.

## Where the selection model lives

| module | subject |
|---|---|
| [`mapping`] | the ONE screen⟷page conversion, and the hit tolerance |
| [`target`] | the provider seam, and the trait re-attached to the salvaged decomposition |
| [`selection`] | selection as identity; the level ladder; re-resolution |
| [`forms`] | filling a form where it is drawn: why it is not a tool, why its hit test takes no tolerance, and what its editor cannot promise |
| [`gesture`] | press / drag / release, the clear that must not happen on a press, Escape's abort, and the one rubber band's two intents |
| [`moving`] | which move verb each rung reaches, the canvas→page delta, and the ghost's honesty rule |
| [`handles`] | eight grips plus move, and the cursor over each |
| [`textsel`] | selecting **text**: why it needs no capability, why it is offered exactly where content selection is not, and the single pass that makes the highlight and the copy one value |
| [`menus`] | the right-click: which of the two canvas menus opens, and the select-first rule that makes it about the thing you pointed at |
| [`overlay`] | what all of it looks like — and what rule 4 forbids it looking like |
| [`geometry`] | the pan and zoom-anchor arithmetic |
| [`pasteboard`] | **off-page reach** — which of the canvas's two interactive rectangles owns this frame's gesture |
| [`keys`] | Escape and Delete, and which of Escape's three claimants gets it |
| [`tool`] | select or hand, and the space bar that borrows the hand |
| [`zoom`] | **the anchor rule**, the two-frame handshake, and the five zoom paths that route through it |
| [`interact`](mod@interact) | **what the operator just did, and what happens as a result** — the pointer frame, the gesture application, the right-click, the keys, the re-resolve, the cursor |

Everything above is pure except this file, [`interact`](mod@interact),
[`overlay`], and [`moving`]'s one wiring function ([`moving::drag`], which
is the only thing there that touches the live object model — its rules are
pure). That is the point: `PROJECT_PLAN.md`'s split is driven by
testability, and the
selection invariants are exactly the kind of property a unit test can hold
and a running window cannot be trusted to demonstrate.

## What is in this file, and what is next door in [`interact`](mod@interact)

This file is **composition**. [`show`] and [`show_in`], the `ScrollArea`,
the strip of page rectangles and the raster or the state sentence drawn into
each, the fit resolved against this frame's viewport, the placeholder a
document with no pages gets, the [`CanvasGeometry`] the rulers are then
painted against, and the layout trace. It needs a live `Ui`, and it answers
one question: **where does everything go on screen?**

[`interact`](mod@interact) is **interaction**. The pointer frame, what a
press would land on, the gesture machine's outcome and its application, the
right-click, the keys, the re-resolve of the selection, the overlay draw and
the cursor. It needs this frame's input, and it answers the other question:
**what did the operator just do, and what happens as a result?**

The two change for different reasons — a page-display mode is a layout
question, a new tool is an interaction one — which is the seam rule R2
forced this file along when it reached 1,526 lines. The invariants that
belong to the second half travel with it, and are written up in
[`interact`](mod@interact)'s header rather than here: **selection survives
navigation**, and the two closed seams (where the selection lives, and the
one shared decomposition).

## Actions, not mutations — and the two documented exceptions

The project's strongest structural invariant is that **no code path runs
from a widget to a document**: everything an operator does becomes an
[`Action`] that is applied *after* the frame is drawn. It is why the old
GUI's undo log is coherent, and it is established here at S0 — with two
actions and one widget — because retrofitting it later is expensive.

[`show`] therefore takes `&mut OpenDoc` but is permitted to write
exactly three fields, none of them document state and none of them
expressible as an action. The first two are **frame bookkeeping about the
view**, and are impossible to defer for the same reason:

1. **`last_scroll_offset`** — the offset the scroll area settled on this
   frame. It is only readable *after* the area is built, and the next
   frame's pan needs it *before* the area is built. Storing it is what
   lets a pan track the hand instead of lagging it by a frame.
2. **`zoom_anchor`** — which page point a zoom step must hold still, and
   where. It has to span two frames because the new zoom is not known when
   the step is asked for: the zoom is an [`Action`], applied after the UI
   is built, and it *clamps*. Recording the inputs and solving next frame
   avoids predicting a clamp we do not control. [`zoom`] owns both ends —
   which point ([`zoom::anchor_point`]) and when to spend it
   ([`zoom::anchor_step`]).

The third is **`selection`**, which arrived here when seam 1 above was
closed, and it does not weaken the invariant: a selection *names* parts of
a document and changes nothing a save would write. It is settled during the
frame, from input that only exists during the frame, so deferring it would
make a click land a frame after the operator made it — the same argument as
the two above. The line is unmoved and visible in [`canvas_keys`]: Delete
removes nothing here, it raises [`VectorAction::DeleteSelection.into()`] carrying the
operand list, applied after the frame through the one funnel. Nothing that
touches `EditSession` runs from a widget.

A fourth value is derived rather than stored: [`crate::viewer::ViewState::apply_fit`]
is called inline, because a fit mode is a pure function of this frame's
viewport and turning it into an action would apply it one frame late —
the page would visibly lag every window resize.

## Input conventions, and why breaking them feels wrong


## The zoom anchor — decided once, in [`zoom`]

`DEFECTS.md`'s "Not defects" table records that *"zoom buttons pin the
page's top-left, not the centre or the cursor"*. The wheel path was fixed
at S0; Phase 3.1 closes the rest, and the rule that governs all five zoom
paths (wheel, in, out, actual size, and the two framing commands) lives in
[`zoom`]'s header and in [`zoom::anchor_point`] — **the pointer when it is
over the canvas, the viewport's centre when it is not**. This file no
longer decides an anchor; it arms one ([`zoom::arm_anchor`]) and consumes
one ([`zoom::consume_anchor`]).
