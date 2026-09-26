# canvas::interact — what the operator just did, and what happens as a result

The **interaction** half of the canvas. [`super`] is the composition half:
it settles *where everything goes* — the scroll area, the strip of page
rectangles, the fit against this frame's viewport, the one screen⟷page map
— and hands the settled facts here in a single [`Frame`]. This file answers
the other question, and only that one: **what did the operator just do, and
what happens as a result?**

Everything a canvas gesture means passes through one function, [`interact`]:
reading the frame's pointer, deciding what a press would land on, advancing
the gesture machine, building a decomposition *only* if something needs one,
applying the outcome, the right-click, the keys, the re-resolve, the draw
and the cursor. Its own header carries the order of those steps and the
argument for that order — including the two orderings that are load-bearing
rather than incidental (Escape read at step 1 and honoured at step 6; the
right-click applied before the resolve). The sections below carry what is
true of the file as a whole.

## Why this is a separate file

Rule R2 (no `.rs` file over 1,500 lines) forced the split at 1,526, and the
seam was already drawn by the two subjects: **composition needs a live `Ui`
and changes when the layout changes** — a new page-display mode, a ruler
reservation, a strip; **interaction needs this frame's input and changes
when a gesture changes** — a new tool, a new outcome, a new key. They change
for different reasons, so they are two files. [`super`]'s header carries the
table of every other `canvas::*` module.

## Actions, not mutations — this is the file that has to hold the line

Nothing here runs from a widget to a document. Every verb an operator
reaches through the canvas leaves as an [`Action`] pushed onto `actions` and
applied after the frame, in one place. Delete is the sharp case and it is
visible in [`keys::canvas_keys`]: it removes nothing, it raises
[`VectorAction::DeleteSelection.into()`] carrying the operand list. The zoom a released
region marquee asks for is an [`Action`] too, and so is every move a drag
commits.

The one document field this file writes back is the **selection**, which is
the third of the three bookkeeping fields [`super::show`] is permitted to
write — see [`super`]'s "Actions, not mutations" section for the whole
argument, and [`interact`]'s own docs for why the selection is taken out of
the document by value and put back at the bottom.

## ★ Selection survives navigation — the invariant this stage is accountable for

`GUI_ROADMAP.md` Phase 1 states it and names three ways it is lost.
[`selection`](super::selection)'s header carries the full table; the
wiring's share of it is visible right here in [`interact`]:

- the selection is **read, resolved and stored** on every frame, and the
  resolve does no work unless `(page, edit epoch)` moved — so a zoom, a
  pan, a fit change, a view rotation, a page-display change and a ribbon-tab
  change all cost one comparison and change nothing;
- the only thing that can clear it is [`gesture::GestureOutcome::Click`],
  which is raised on a **completed click with no drag**;
- a decomposition is built **only when a gesture needs one or the epoch
  moved** — never per frame, and never merely because the view changed.

**A move does not threaten the invariant, and that is measured rather than
hoped.** `move_*` rewrites operator operands in place, adds and removes no
operator, and therefore renumbers nothing — `pdfcer-core`'s
`object_identity_across_edits.rs` decomposes, edits and decomposes again to
prove it. So a committed move leaves every `Selection` untouched and only
the *outlines* have to catch up, which the epoch bump already handles. The
`delete_*` family is the one that renumbers. See [`moving`]'s header for the
full table.

## ★ The two seams that were wiring, and how they were closed

Both were recorded here as *"one-line changes once the field they want
exists"*. The field exists; both are closed. They are written up rather
than deleted because each closed a live hazard, and the next person to
consider reopening one should have to read what it cost.

1. **The selection was held in `egui`'s `Memory`.** It is document-scoped
   state, and `Memory` outlives documents — so the canvas had to *detect*
   a document change with a `DocumentToken` built from the
   `Arc<EditSession>`'s **allocation address** mixed with the page count,
   compared once per frame. That is the same address-as-identity key
   `panels::DocKey` was deleted for earlier in this stage, with the same
   ABA hazard.

   It now lives on [`crate::app::state::OpenDoc::selection`], and
   `DocumentToken` and `SelectionState::sync_document` are **deleted**.
   `OpenDoc::new`'s own argument does the work: constructing fresh state
   per document is what makes *"a cached value can never refer to a
   previous file"* true by construction, on every frame, with nothing to
   compare. The canvas still owns the selection in the sense that matters —
   it is the only writer — and the *edit* a Delete leads to is still an
   [`Action`] applied after the frame.

2. **The decomposition was built transiently, per gesture.** The Objects
   panel's cache moved onto `OpenDoc` earlier in this stage; until this
   change the canvas could not reach it, so `build_targets` called
   `ObjectModelProvider::build` for itself on every click and every marquee
   release — a **second** decomposition of a page the panel had already
   decomposed, which is the *"two decompositions quietly diverge"* failure
   decision 011 names.

   [`interact`] now reads [`crate::app::state::OpenDoc::page_objects`], and
   `build_targets` is deleted. One decomposition per `(page, epoch)`, shared
   by the panel, the Properties row, the `objects n=` trace and the hit
   test — the same value, not two values that agree. [`show`](super::show)
   needs no provider argument, because the document it already takes
   carries it.

   The gating is kept and still matters: the cache is asked for **only**
   when a gesture needs a hit test or when `(page, epoch)` has moved, so a
   zoom or a pan with the Objects panel closed still decomposes nothing.
   Drawing needs no provider at all — the outlines are cached in canvas
   space, which is zoom-independent.
