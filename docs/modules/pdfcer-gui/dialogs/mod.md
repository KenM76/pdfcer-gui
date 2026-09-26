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

## Item notes

### `fn retire`

`open` is what the dialog's own `show` returned — *"should I still be on
screen?"* — and `answered` is whether it is holding a decision its owner has
not collected yet. A dialog is retired only when **both** say no: it is off
screen *and* it has nothing left to hand over.

# WHY THIS IS NOT `!open`

`!open`, expressed at each call site as
`if …map(|d| d.show(ctx)) == Some(false) { self.slot = None; }`, is exactly
right for eleven of the thirteen dialogs: they act through `actions` while
they draw, so a closed one has nothing left in it.

The two **confirmation** windows are different in kind, and the difference
is the whole defect. `unsaved` and `signature` deliberately do NOT act. They
*park* an answer and let `crate::app::PdfcerApp` perform it —
`resume_after_unsaved` and `resume_after_signature`, both later in the same
frame — because the acts in question (closing a document, writing over the
operator's own file) are the two most destructive things this shell does and
must have exactly one route each. A window that could call `save_in_place`
would be a second route.

So for those two, `show` returning `false` and the dialog being *finished*
are different facts. Pressing the button sets the answer, which is what
makes `show` answer `false` — so a `!open` branch destroys the dialog **and
the answer inside it** before the drain three call frames later can look,
and `take_signature_answer` finds an empty slot and returns `None`.

⇒ The observable result, and what `an_invalidating_save_is_warned_about`
drives the binary to check: the signature warning opens, holds the save,
draws its proceed button, takes the click, **closes** — and traces no
`signature-confirmed` and writes no file. A signed document cannot be saved
at all by any route the guard covers, which is worse than having no guard:
it stops the save and never lets it through.

Neither half is wrong on its own, which is why no unit test can see it. The
dialog returns its answer when asked; the drain performs whatever it is
given; the defect lives entirely in the **lifetime between them**, and a
lifetime is not a value any assertion over either half can name. That is the
same shape `PROJECT_PLAN.md` §4 built the driving harness for.

# The invariant this creates, stated where it can be checked

> **Every caller of [`DialogsState::show`] must drain the parked answers in
> the same frame.**

There is one caller — `crate::app::frame` — and it drains both, immediately
after. A retained-because-answered dialog therefore lives for zero frames:
it is emptied and dropped by `take_*_answer` before anything can draw it
again. A caller that did not drain would see the window redraw for as long
as it ignored it, which is a loud failure rather than a silent one, and that
direction was chosen deliberately over discarding the answer.

### `fn close_document_scoped`

One place, so a document-scoped dialog added later cannot be forgotten
by whichever of the close paths its author did not think of.
Application-scoped dialogs are deliberately absent — see
[`Self::show`].
