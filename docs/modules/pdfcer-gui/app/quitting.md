# `app::quitting` — **closing the program without losing anybody's work**


> *"when I close the program it should prompt to save changes if there are
> any, and it should do what other programs do — switch focus to the document
> that is being prompted for, and cycle through each unsaved document while
> it prompts, but also have a save all button that saves all changed
> documents."*

## What this module owns, and what it borrows

`crate::dialogs::unsaved` owns the **question** and asks it about one
document: it focuses that tab first, counts edits from the last *save* rather
than from zero, and refuses to proceed on a save that did not happen. This
module owns the **cycle** that puts that question to every dirty document in
turn, and the reading of `eframe`'s close request that starts it.

⚠ Nothing else on the exit path asks anything. `App::on_exit` flushes a
layout debounce and that is all it does, so a close request that reaches the
viewport unheld ends the process with every unsaved document still unsaved —
from the window's ✕ or from `Alt+F4`, in one keystroke.

## The cycle, and why it is derived rather than remembered

[`Quitting`] holds **one boolean**. Everything else is re-derived from the
document set on every frame:

1. a close is requested, and something is dirty → **cancel the close**, set
   the flag;
2. while the flag is set, find the **first dirty slot**, activate it, and ask
   about it;
3. an answer either cleans that document or closes it, so the next scan finds
   the next one;
4. nothing dirty left → **close for real**;
5. Cancel at any point clears the flag and the program stays open.

A remembered queue would be a second model of the document set, and it
would go stale the moment a save, a discard or a close changed one — which is
exactly what every answer here does. Re-deriving cannot drift, and the cost
is a scan of at most a handful of slots on the frames where a modal is up.

Step 2 is the operator's second requirement and it is not decoration: a
modal asking *"save changes?"* over a document you cannot see is asking about
a file you have to guess at. `crate::dialogs::unsaved`'s own `PendingIntent`
already documents that rule for the single-tab case; this applies the same
one to the quit cycle.

## Why Cancel abandons the whole quit, not one question

Because that is what the operator meant by it, and it is what Word, VS Code
and Notepad++ all do. The alternative — Cancel skips this document and asks
about the next — leaves the program in a state where some documents have been
closed and the operator asked for none of it.

⇒ Cancel is the **only** answer that undoes work already done in the cycle,
and it undoes it by not having done any: nothing is closed until the last
question is answered… except that it is, because a Discard closes its
document as it goes. That is the honest limit of this design and it is stated
rather than hidden — see [`Quitting::stand_down`].

## Item notes

### `impl crate`

Everything above this point is the *rule*, expressed over a slot count and a
dirty predicate so it can be tested without an application; everything below
is the application applying it.

### `fn scan`

The functions under test take a closure precisely so this can exist —
building real `Status::Open` values needs a parsed document, and what is
being tested is the **ordering and counting**, not the dirty predicate,
which is `save::has_unsaved_edits` and is tested where it lives.

### `fn the_cycle_takes_the_leftmost_dirty_document_first`

Lowest slot first — left to right in the tab strip, which is the order
the operator reads them in. Any other order would make the cycle feel
arbitrary, and "arbitrary" is what a modal must never feel while it is
asking about destroying work.

### `fn a_clean_set_has_no_first_dirty`

The case that must not regress into a spurious modal: an operator who
has saved everything and presses ✕ should get an immediate exit, not a
dialog with nothing in it. `first_dirty` answering `None` is what makes
the whole cycle skip.

### `fn save_all_is_for_more_than_one`

With one dirty document the two buttons are the same act, and a second
button that means the same thing is one the operator has to stop and
think about — on a modal that is standing between them and their work.

### `fn the_cycle_starts_down_and_cancel_puts_it_back`

Pinned because `running` defaulting to `true` would make the
application try to quit on its first frame, which is the one failure
mode of this design that would be spectacular rather than subtle.

### `fn stand_down`

Called on Cancel, and it is the answer with the honest caveat: any
document the operator already chose to *discard* in this cycle is
already closed, and cancelling does not bring it back. That matches
every editor in the class — a discard is an answer, not a step — but it
is worth saying out loud rather than leaving somebody to discover it.

### `fn first_dirty`

The whole of the cycle's ordering: lowest tab position first, which is
left-to-right in the strip and is the order an operator reads them in.

Takes a **predicate** rather than `&PdfcerApp`, so it can be tested without
an application — and a predicate rather than the `Status` itself because
`Status` is deliberately not `Clone` (it owns an `EditSession`). What this
needs to know is *"is slot n dirty"*, which is one bool.

### `fn dirty_count`

Read for one decision only: whether to offer *Save all*. A cycle of one does
not need it, and a button that does the same as the one beside it is a
button an operator has to think about.

### `fn is_dirty`

Delegates to `save::has_unsaved_edits`, which is the **one** expression
of this question — the same one `dialogs::unsaved::ask_for` consults before
deciding whether to ask at all. A second expression of *"is this document
dirty"* anywhere in the crate is a defect by construction: the two eventually
disagree, and the shape of the disagreement is a modal that does not appear
or one that appears over a clean file.

### `fn step_quit_cycle`

Called once per frame, after both dialog drains. See
[`crate::app::quitting`] for why the cycle is derived from the document
set rather than remembered as a queue.

# The states, and the order they are checked in

1. **Cancelled** — the operator answered Cancel, so stand down and stay
   open. Checked first, because every state below would otherwise act on
   a cycle that has just been abandoned.
2. **A close was requested and nothing is dirty** — let it through. No
   dialog, no cancelled close, no flicker.
3. **A close was requested and something is dirty** — cancel the close
   and begin the cycle.
4. **The cycle is running and something is dirty** — activate that
   document and ask about it.
5. **The cycle is running and nothing is dirty** — close, for real.

# Why the close is cancelled rather than pre-empted

`egui` reports the request and closes at the end of the frame unless
something says otherwise. There is no "ask first" hook, so the sequence
has to be *let it be requested, cancel it, then re-request it when the
questions are answered*. `ViewportCommand::CancelClose` is that, and it
must be sent on **the same frame** the request is read.

### `fn ask_unsaved_for_quit`

A thin wrapper rather than a parameter on `DialogsState::ask_unsaved`,
because every other caller is about **one** document and should keep
saying so without being edited.

### `fn save_every_dirty_document`

`OPERATOR_REQUESTS.md` O102's fourth requirement — *"a save all button
that saves all changed documents"* — and the thing that makes the quit
cycle bearable: without it, somebody with six dirty documents answers six
questions.

# Returns

`false` if any attempted write failed, so the caller can abandon the
resume. A document with **no file is not attempted and is not a
failure**: it needs a destination, which is a question only the operator
can answer, and the cycle asks about those individually afterwards.

# Why it activates each slot before writing it

Because `save::save_in_place` takes the **active** document, and the
application's own invariant is that `status` is the active one with the
rest parked. Reaching into a parked slot to write it would be a second
way to save, and the two would eventually disagree about what a save
does — the signature question, the receipt, the epoch. Activating first
means every document in the batch is saved by exactly the path a single
save uses.

The originally-active slot is restored at the end, so an operator who
cancels the rest of the cycle is looking at the document they were
looking at when they pressed the button.
