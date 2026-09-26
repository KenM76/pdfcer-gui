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

## ★ Invariant 2, and it lives entirely in this file

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
