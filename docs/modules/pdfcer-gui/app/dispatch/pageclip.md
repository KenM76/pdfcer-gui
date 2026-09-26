# `app::dispatch::pageclip` — **cut, copy and paste whole pages**

`OPERATOR_REQUESTS.md` **O59**, item 2 — Ken: *"can you make sure we have
cut, copy, and paste available for everything and if not implement?"*

## Why these are their own commands and NOT `Ctrl+C`

This is the decision that shapes the whole module, and getting it the other
way round would have taken the clipboard away from the canvas permanently.

Every `pages.*` verb takes its operand from one rule, written down once in
`panels::pages::ops::operands`: **the picked sheets when there are any, the
current page when there are none.** That fallback is right for Delete,
Rotate and Extract — with nothing picked they act on the sheet you are
looking at, which is a defined answer rather than a disabled state.

It is fatal for a chord. A rung in `dispatch::clipboard`'s fork that asked
*"is there a page operand?"* would get **yes, always** — there is always a
current page — so `Ctrl+C` would copy a page instead of the shape the
operator had selected, for ever, and no state they could reach would give
the canvas its chord back.

⇒ So pages get **named commands** on the Pages tab and in the thumbnail
context menu, and the canvas keeps `Ctrl+C`. That is also what R8 asks for:
the capability exists because a command is registered, and it is reachable
by pointing at it rather than by knowing a rule.

Acrobat resolves the same collision by **focus** — `Ctrl+C` in its page
thumbnails copies pages. That is a legitimate answer and it is not available
here: this shell's thumbnails are a dock panel whose focus egui does not
model in a way a chord dispatcher can read, and inventing a focus notion to
serve one chord is a mechanism that would then own every other chord too.
Named and rejected rather than silently not done.

## Why copy is inline and cut raises an action

`copy_pages` is `&self` — it changes no document — so it runs here, writes
the clipboard, and raises nothing. That is `dispatch::textcopy`'s rule
exactly: *"the action funnel exists for work that changes a document or that
must not happen mid-frame, and a copy is neither."*

A **cut** does change one, so it is copy-then-`PageAction::DeletePages`:
the clip is captured first, and the existing delete arm — which already
resyncs the panel selection, clears the canvas and is one undo entry — does
the removal.

The engine ships `cut_pages`, which does both in one call, and it is
**not** used. Not an oversight: the clipboard lives in `egui::Memory` and
the action applier has no `egui::Context`, so a single-call cut could not
put its own clip anywhere. Copy-then-delete costs one extra page-tree walk
and keeps the undo entry count at one, which was the property the engine's
verb existed to guarantee.

## What the operator must be told, and when

Two disclosures, and they are at opposite ends of the gesture because they
answer different questions:

| when | what | why it cannot wait / cannot be earlier |
|---|---|---|
| **at the copy** | *"a form field was left behind"* | `PageClip::fields_dropped` — a field whose boxes straddle a copied and an uncopied sheet cannot travel, and the operator selected **pages**, not fields, so nothing they did says a field is about to go missing |
| **at the paste** | *"boxes arrived that nothing can fill"* | `InsertOutcome::orphaned_widgets` — a page's `/Annots` reaches its widgets and the `/AcroForm` that owns them does not travel, so they draw like fields and are dead |

The second is the one that **produces a document that looks right and
is not**, and it is invisible by construction: an orphaned widget draws
exactly like a live field. There is no screenshot that shows the difference,
so the status row is the only place it can be said — which is rule 4's
surviving half, again.
