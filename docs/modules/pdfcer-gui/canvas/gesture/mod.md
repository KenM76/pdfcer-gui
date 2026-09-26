# `canvas::gesture` — press, drag, release, and the clear that must not happen on a press

This file is the **state machine**: one [`PointerFrame`] in, one
[`GestureOutcome`] out, and a single `Option` of carried state between
frames. What a press *means* — the [`DragKind`] it produces, the
[`MarqueeIntent`] a rubber band carries to its release, and the precedence
that decides between them when more than one meaning is available — is a
pure decision function and lives next door in `meaning.rs`
(`canvas::gesture::meaning`). It is re-exported from here, so
`canvas::gesture::press_kind` and `canvas::gesture::DragKind` still name it
from this module and no caller has to know about the split.

The division is by subject, not by size: `meaning.rs` answers *"what does
this press mean?"* from `(tool, grip, zoom_armed, capabilities)` with no
state at all, and is where marquee-select-versus-marquee-zoom and the
per-mode gate on a press are documented. This file answers *"what is
happening to that meaning now?"* across the frames of one gesture — the
press that decides nothing, the drag in flight, the release that commits,
and the Escape or interruption that abandons.

## Invariant 2, and it lives entirely in this file

**A selection is cleared by a completed click with no drag, never by a
press** — `GUI_ROADMAP.md`'s selection rule. A pan, a marquee and a move
all begin with a press on the canvas, so a press that cleared would make
every one of those gestures start by destroying its own operand.

[`GestureState::update`] returns [`GestureOutcome::Idle`] on the press
frame — always, unconditionally, whatever the press landed on. A press
records where the gesture began and does nothing else. Only a *completed*
interaction produces an outcome, and only [`GestureOutcome::Click`] can
reach [`crate::canvas::selection::SelectionState::click`].

The distinction is egui's to make and it already makes it correctly:
`Response::clicked()` is true for a press-and-release that did **not**
exceed the drag threshold, and `drag_started`/`dragged`/`drag_stopped` are
true for one that did. The two are mutually exclusive on one interaction.
What this module adds is the guarantee that **nothing else is consulted**
— in particular not `is_pointer_button_down_on`, which is true on the
press frame and is exactly how the defect above gets written.

## Primary button only, and why every canvas gesture must say so

`Response::drag_started()` is button-agnostic: it is true for a middle-
and a right-drag as well as a left one. That is harmless until the middle
button means something — and here it means **pan**. A pan read as a
selection gesture would make dragging across a drawing replace the
selection, or, once a move verb is wired, silently rewrite the page.

So the canvas reads `..._by(PointerButton::Primary)` and this module never
sees any other button. The right button is excluded for the same reason:
it opens the canvas context menu, which `canvas::menus` reads from the
`Response` directly and never through this machine.

## Marquee versus pan: settled by the button and the tool, not by a heuristic

A drag starting on empty canvas would be ambiguous between pan and
marquee-select if the two shared a button. They do not: `canvas/mod.rs`
switches egui's button-agnostic drag-to-scroll **off** and pans against the
scroll offset on the **middle** button, leaving the left button to the
selection marquee. Left drags marquee; middle drags pan; neither can be
mistaken for the other, and no distance threshold or modal state is
involved.

The hand tool and space-to-pan give the *primary* button a second
meaning — and the resolution keeps the same shape. The hand
tool is not a third `DragKind`: when [`crate::canvas::tool::active`] says
`Hand`, `canvas/mod.rs` hands this machine a **blank** [`PointerFrame`], so
a pan is not a gesture this module can see, let alone one it could confuse
with a marquee. One state machine, one meaning per frame, and the branch is
in one `if` at the boundary rather than a flag threaded through every arm.

## Item notes

### `const GESTURE_SLOT`

Its own slot: `trace_changed` keys on the slot, so sharing `canvas-press`'s
would make each line suppress the other's, and a suppressed line looks
exactly like the frame never happening.

### `fn a_press_alone_produces_no_outcome`

Whatever the press landed on, and whatever else the frame carries, the
press frame is `Idle`. Nothing downstream of this can clear a
selection, because nothing downstream of this is called.

### `fn a_press_that_becomes_a_drag_never_yields_a_click`

The whole sequence, frame by frame, as the roadmap describes it: press
on empty canvas, move, release. If any frame produced a `Click`, the
selection would be cleared by hit test — which is the defect.

### `fn a_zoom_marquee_is_the_same_band_with_the_other_intent`

Asserted by driving both intents through the identical frame sequence
and comparing the outcomes field by field: everything but `intent` must
match. That is the mechanical form of *"do not add a second rubber band
with different pixels"* — if the two ever diverged geometrically, the
canvas would be drawing two bands from one `draw_marquee`, and the
operator would see a zoom box that did not agree with the box that had
been selecting a moment earlier.

### `fn a_marquee_keeps_the_intent_it_started_with`

Modelled the way the machine actually experiences it: the caller
reports `Select` on every frame after the press, exactly as it would
once the arming flag had been cleared.

### `fn a_blank_frame_starts_nothing_however_hard_the_pointer_is_working`

The canvas hands this machine a blank `PointerFrame` while the hand
tool is active. This pins what "blank" is worth: whatever the pointer
is doing on screen, nothing starts, nothing draws, nothing commits.

### `fn a_markup_band_reports_its_endpoints_in_drag_order`

Asserted against a drag that goes **up and to the left**, because that
is the case a normalising implementation gets wrong: `from` would come
back as the smaller corner, which for this drag is the *head*.

### `fn escape_abandons_a_markup_drag_without_committing`

The existing cancellation test covers the three older kinds; this adds
the one where an un-cancelled release would write to the document. A
`Complete` here would be an annotation in the file that the operator
explicitly abandoned.

### `fn a_forbidden_press_starts_nothing_and_a_forbidden_click_reports_nothing`

The state-machine half of the gate. The click assertion is the
load-bearing one: a click is not a drag, so a build that gated only the
press would still select on every click — which is the single most
common gesture on the canvas.

### `fn a_drag_is_anchored_at_the_press_not_at_the_frame_it_was_recognised_on`

The regression test for the 94-point offset measured on a real drag —
see [`PointerFrame::press_origin`]. It is stated as a **magnitude**
against the press point rather than as "the band is on the page": a band
anchored at the recognised frame is on the page too, just in the wrong
place.

The fallback is asserted in the same test: a frame with no press origin
behaves exactly as it did before the field existed, so supplying it is
an accuracy improvement and never a behaviour change.

### `fn an_interrupted_drag_is_abandoned_rather_than_committed`

Focus loss, a dialog, the pointer leaving the window: egui stops
reporting the drag without ever reporting a stop. Committing on the
next frame that happens to look like a release would apply an edit the
operator never finished.

### `fn escape_abandons_a_drag_in_flight_without_committing`

The gesture ladder's escape hatch: a move drag that is halfway across
the page and clearly wrong must be abandonable without an undo. The
frame that carries the cancel produces `Cancelled` — never a
`Complete`, which is the outcome that would have rewritten the page.
