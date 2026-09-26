# `dialogs` — the shell's stationary, screen-anchored surfaces

## What belongs here, and what does not

A **dialog** is a single transaction with a start and an end: it is opened
deliberately, it holds one job's worth of answers, and closing it forgets
them. A **panel** is somewhere an operator dips in and out of while
working, and it keeps its state across documents. The distinction decides
where a surface lives, and getting it wrong is not cosmetic — a print
configuration that persisted across documents would let a range typed for
one file silently apply to another.

## Every dialog here is screen-anchored, never page-anchored

Made in response to a specific operator objection: **controls whose position
is derived from the page move on every zoom and scroll.** A surface an operator is reading and
typing into must stay where they put their eyes. Each dialog therefore
anchors to the viewport rather than being positioned relative to the
canvas, and none of them is drawn inside the canvas's coordinate space.

## Where dialog state lives, and why it is one field

[`DialogsState`] is the whole dock-side surface of this module: one field
on `PdfcerApp`, one `open_*` call per dialog from the command dispatcher,
and one [`DialogsState::show`] call per frame. It follows
`crate::panels::PanelsState` exactly — same idiom, second instance, not a
new convention — and the reason it is a struct rather than a bare
`Option<PrintDialog>` is that the *next* dialog is then a change to this
file rather than to `app/mod.rs`, which is the file every parallel task
already contends over.

## Why a dialog does not push an `Action`

`crate::app::actions`' invariant is that **no code path runs from a widget
to a document**, and the four things it buys are all about *document*
state: a coherent undo log, an aliasing problem turned into a queue,
explicit ordering between changes, and a greppable answer to "what can
change this?".

A print changes no document state. It reads the document — the pages, the
edited view — and writes to a spooler, so it contributes nothing to the
undo log and has nothing to order against. Routing it through the funnel
would add an `Action` variant that `apply` could only answer by reaching
back into a dialog for the state it needs, which is the funnel pointing the
wrong way.

What the funnel's *reason* does still demand is that the irreversible work
not happen part-way through a layout pass, and [`print::PrintDialog`]
honours that in its own scope: the button sets a flag, and the spool runs
after the window's closure returns. See that field's documentation.

**A dialog that edits the document is a different case and must use the
funnel.** The properties dialog and the settings host will both raise
`Action`s; this note is about printing specifically, not about dialogs in
general.
