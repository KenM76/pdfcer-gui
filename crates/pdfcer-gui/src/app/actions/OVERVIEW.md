# `app::actions` — the one channel through which anything changes

## The invariant this module exists to enforce

**No code path runs from a widget to a document.** A widget that is
clicked, a key that is pressed, a wheel that is spun — none of them
change anything directly. Each produces an [`Action`], the actions are
collected while the frame is drawn, and they are applied *after* it, in
one place, by [`PdfcerApp::apply`].

`SALVAGE.md` calls this *"the single best structural decision in the old
GUI"*, and `PROJECT_PLAN.md` §3 lists it first among the invariants that
are "not up for renegotiation". It is established here at stage S0, with
a handful of actions and one widget, **because retrofitting it is
expensive**: every widget written under the other discipline has to be
found and rewritten, and the ones that are missed are exactly the ones
that produce an incoherent undo log later.

## Why it is worth the indirection

Four things fall out of it, none of which can be had cheaply otherwise:

1. **A coherent undo log.** One operator gesture becomes one action
   becomes one command-log entry. A widget that mutated in place would
   have to remember to log, and the ones that forgot would be invisible
   holes in the history. (S0 has no undo — but S4's undo is only
   possible because the funnel already exists.)
2. **The borrow checker stops fighting.** egui is immediate-mode: the
   document is being *read* to draw the very widget that wants to change
   it. Deferring the change to after the frame turns an aliasing problem
   into a queue.
3. **Order becomes explicit.** Two actions raised in one frame are
   applied in a defined order, in one readable function, rather than in
   whatever order the layout code happened to run.
4. **Every state change is greppable.** "What can change the zoom?" has
   a complete answer: the [`Action`] variants that touch it.

## Scope

Zoom, page navigation, and — from stage S4 — **the actions that change the
document**: [`Action::DeleteSelection`], the three move verbs
([`Action::MoveSelection`], [`Action::MoveSubpath`], [`Action::MoveNode`])
and, from Phase 6, [`Action::CommitMarkup`].
Those are why this module has a mutation path, and the path is short and
deliberately in one place; see [`vector_edit`], which every one of them goes
through so the cancel-mutate-bump-invalidate protocol is written once rather
than four times.

**There is no resize action, and its absence is deliberate.**
`EditSession` has the whole `move_*` family and no scale verb of any kind,
so a `ResizeSelection` here would be an enum variant nothing could honour —
which is the same "no placeholders" rule this enum's own doc comment states
two paragraphs down. The canvas still *consumes* a grip drag, so it cannot
fall through to a marquee, and commits nothing; see
[`crate::canvas::handles`].

## ★ Opening and closing are actions now, and this header said they would be

Until `file.open` existed, this paragraph read: *"Opening a document is
deliberately not an action at S0: it happens once, from `argv`, before the
event loop starts. It becomes one the moment there is an Open command, and
the `apply` gate that blocks an open while a save is pending lands with
it."*

Both halves have now happened. [`Action::Open`] and [`Action::Close`] are
the first two variants that are about **which document is open** rather
than about the one that already is, and they are the reason [`PdfcerApp::apply`]
no longer begins by refusing everything when nothing is open — see the
★ comment at the top of its body, which is the whole of why the two are
matched *before* that guard rather than inside it.

The gate is [`PdfcerApp::save_pending`], consulted by both, and its own doc
comment carries the rule. It answers `false` in this build because there
is no save path at all — so no confirmation dialog is built for a
condition that cannot occur, and the rule has one home rather than being
rediscovered by whoever adds the save.

**The dialog is not the action.** `file.open` opens a native file picker,
which is a UI act that happens during dispatch; what goes through the
funnel is its *result*, a path. See [`crate::app::files`] for the picker,
the diagnostics seam that lets a scripted harness answer it without a
human, and why the two are separated at exactly that line.

## ★ `Action` is no longer `Copy`, and that is a decision rather than an accident

[`Action::DeleteSelection`] carries a `Vec<usize>` — the paint-order
indices to remove — and it has to, for a reason that is not about
convenience:

`EditSession::delete_objects` takes a **slice** and resolves every index
before planning anything, *"so an out-of-range one refuses the call rather
than deleting the prefix that happened to resolve"*. Deleting a
multi-selection therefore has to be **one** command. Emitting one
`DeleteObject` action per selected object would renumber the page between
them — deleting object 5 and then object 3 deletes 5 and then whatever
moved into slot 3 — so the batch cannot be decomposed.

The alternative to carrying the list would be for `apply` to read the
selection itself. It cannot: the selection lives in the canvas, `apply`
has no `egui::Context`, and giving it one would make the action funnel
depend on the UI framework. Carrying the operands is also simply what an
action *is* — a complete statement of an operator's intent, resolvable
after the frame that raised it.
