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
