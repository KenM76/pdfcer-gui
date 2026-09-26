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

## Item notes

### `mod deleting`

Three engine verbs — `delete_subpath`, `delete_text_run`, `delete_node` —
had their MOVE twins wired and themselves called by nothing, so on a CAD
export a line could be entered, selected and dragged and could not be
removed. Its header carries the whole argument, including why one refusal is
pre-empted (R83) and the rest are left to the engine.

Pure — no egui, no pointer, no document — because both the Delete key and
the ribbon's `format.delete` ask it, and a destructive rule stated twice is
a rule that drifts.

### `mod overlays`

`egui_shell::theme::Overlays` is a generic role map because **R7** forbids
the shell learning what a ce dimension is; the roles are pdfcer's, exactly as
the ribbon manifest's command ids are. Its header carries the mapping
argument and the distinctness test the shell says the application owes.

### `mod pagedrop`

The operator's request of 2026-08-19: *"…or onto the canvas to add pages
and insert them in between the pages we've dragged to"*. The drag itself
lives in [`crate::pagedrag`], which is what lets a gesture that began in a
panel — possibly in another document — end here.

### `mod notepopup`

The operator, 2026-09-05: *"I could add a yellow sticky note but even in
read mode I don't think I could figure out how to read it."* He was right,
and the measurement was worse than the report: the only route to a comment
was the Comments panel, on the `markup` tab, which Read is not shown.

It lives on the **canvas** rather than on the ribbon precisely so that it
is mode-independent by construction — no future edit to a tab list can take
reading away from Read mode again. Its header carries the whole argument,
including why the pop-up is chrome rather than content under rule 4.

### `mod pasteboard`

O23's second half. The operator, 2026-09-10: *"how do I view and edit
objects that are off of the page? we added this feature but I didn't see
how to enable it."* There was nothing to enable: the pasteboard — the
viewport of scrollable slack [`geometry::content_extent`] puts on every side
of the strip — sensed hover and refused clicks, so a press out there never
became a gesture and an object dragged past the sheet edge was unreachable.

Its header carries the whole argument, including why this is a choice
between two responses rather than one widened page rect, and the two
clauses about a drag that crosses the sheet edge mid-gesture.

### `mod pick`

`OPERATOR_REQUESTS.md` O17. This is the replacement for Edit ▸ Content's
declare-your-intention-then-point model, and its header carries the whole
argument: why a filter belongs on the status bar rather than the ribbon,
why it is **subtractive only** (so `default()` reproduces today's behaviour
and R6 holds by construction), and why it composes with
[`crate::app::modes::capability::Capabilities`] as an `AND` rather than an
override.

Pure: no egui, no pointer, no document. Which is exactly why the popup that
drives it still has to be driven before any of it counts — R1.

### `mod previews`

Its header carries the convention it **reverses** by operator ruling —
`handledrag.rs`'s *"a preview shows the cursor, the render shows the
document"* — and the measurement that makes it possible: a rasterised
preview is a second away on a CAD sheet, and this never touches the
rasteriser.
The fourteen **pre-commit** slots one frame of the canvas might fill — the
marquee, the ghosts, the shape preview, the snap marker, the ink trail.

Extracted from [`interact`] under R2; its header carries the one argument
they all share (why each is its own value and not a variant of another) and
the Rule 4 reading that makes every one of them permitted.
