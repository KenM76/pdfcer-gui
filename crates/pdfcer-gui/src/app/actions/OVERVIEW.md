# `app::actions` — the one channel through which anything changes

## The invariant

**No code path runs from a widget to a document.** A widget that is clicked, a
key that is pressed, a wheel that is spun — none of them change anything
directly. Each produces an [`Action`]; the actions are collected while the frame
is drawn and applied *after* it, in one place, by [`PdfcerApp::apply`].

`PROJECT_PLAN.md` lists this first among the invariants that are not up for
renegotiation, and it is enforced from the first widget rather than retrofitted:
every widget written under the other discipline would have to be found and
rewritten, and the ones missed are exactly the ones that produce an incoherent
undo log.

## What the indirection buys

1. **A coherent undo log.** One operator gesture becomes one action becomes one
   command-log entry. A widget that mutated in place would have to remember to
   log, and the ones that forgot would be invisible holes in the history.
2. **The borrow checker stops fighting.** egui is immediate-mode: the document
   is being *read* to draw the very widget that wants to change it. Deferring
   the change to after the frame turns an aliasing problem into a queue.
3. **Order is explicit.** Two actions raised in one frame are applied in a
   defined order, in one readable function, rather than in whatever order the
   layout code happened to run.
4. **Every state change is greppable.** *"What can change the zoom?"* has a
   complete answer: the [`Action`] variants that touch it.

## Scope

View state (zoom, page navigation), document mutation, and which document is
open. Every variant that changes the document goes through [`vector_edit`], so
the cancel-mutate-bump-invalidate protocol is written once rather than once per
verb.

**There is no resize action, and its absence is deliberate.** `EditSession` has
the whole `move_*` family and no scale verb of any kind, so a `ResizeSelection`
would be an enum variant nothing could honour — the no-placeholders rule applied
to this enum. The canvas still *consumes* a grip drag so it cannot fall through
to a marquee, and commits nothing; see [`crate::canvas::handles`].

## Opening and closing are actions

[`Action::Open`] and [`Action::Close`] are about **which document is open**
rather than about the one that already is, which is why [`PdfcerApp::apply`]
matches them *before* the guard that refuses everything when nothing is open.

Both consult [`PdfcerApp::save_pending`], and that function's own docs carry the
rule, including why it answers `false` today and what will make it live. Do not
restate it here.

**The dialog is not the action.** `file.open` opens a native file picker, which
is a UI act that happens during dispatch; what goes through the funnel is its
*result*, a path. See [`crate::app::files`] for the picker and the diagnostics
seam that lets a scripted harness answer it without a human.

## `Action` is not `Copy`

[`Action::DeleteSelection`] carries a `Vec<usize>` — the paint-order indices to
remove — and it has to.

`EditSession::delete_objects` takes a **slice** and resolves every index before
planning anything, so an out-of-range index refuses the call rather than
deleting the prefix that happened to resolve. Deleting a multi-selection is
therefore **one** command. One `DeleteObject` action per selected object would
renumber the page between them — deleting object 5 and then object 3 deletes 5
and then whatever moved into slot 3 — so the batch cannot be decomposed.

`apply` cannot read the selection itself instead: the selection lives in the
canvas, `apply` has no `egui::Context`, and giving it one would make the action
funnel depend on the UI framework. Carrying the operands is also what an action
*is* — a complete statement of an operator's intent, resolvable after the frame
that raised it.
