# `pagedrag` — a page drag in flight, wherever it started and wherever it
ends

One value, held in [`egui::Memory`], read by four surfaces that have no
other way to reach each other:

| surface | what it does with it |
|---|---|
| [`crate::panels::pages`] | starts it on a tile press; resolves a gap and a caret while it is in flight; ends it on release |
| [`crate::app::doctabs`] | springs a document tab open when the pointer dwells on it, so a drag can cross from one document to another |
| [`crate::canvas::pagedrop`] | resolves a gap between pages on the page view and draws the same caret |
| [`crate::app::status`] | says, in page numbers, where the drop would land |

## Why `egui::Memory` and not a field on `PdfcerApp`

Because the drag has to **survive a document switch**, and switching
documents calls `PanelsState::forget_document`, which is
`*self = Self::default()`. The Pages panel's own reorder drag lives on
`PagesUi` — correctly, because it can never outlive the document it is
reordering — and putting a *cross-document* drag in the same place would
mean the spring-loaded tab that makes the feature possible also destroys the
drag that needed it.

`canvas::textedit::pen` reaches the same answer from the other direction:
it lives in `egui::Memory` so that any surface can reach it through
`ui.ctx()` with no plumbing. **Move it, do not plumb it.** Four surfaces in
three module trees is more plumbing than that rule was written about, not
less.

The trade is stated plainly: memory-held state is reachable from anywhere,
which is exactly its value and exactly its hazard. The mitigation is that
**this module is the only code that names the key**. Nothing else calls
`data_mut` for it, so "who can write this?" has a grep-able answer.

## Why the operand set is captured at PRESS and not resolved at release

The opposite of what the Pages panel's own reorder does, and the difference
is the document switch again. `PagesUi::drag` holds only an **origin** and
resolves its operands at release, through
`ops::operands(&selection, current, page_count)`, so that a drag reflects
the selection as it stands rather than a second copy of it that could
disagree.

That reasoning holds exactly as long as the selection is still there at
release. A cross-document drag activates another document on the way, which
clears the Pages panel's selection — so resolving at release would resolve
against the *target's* selection, or against nothing. The operand set is
therefore captured, and captured **with the slot it came from**, which is
the pair that makes it meaningful later.

## A drag between documents COPIES, and Shift makes it a move

Stated here because this is the module every reader of the feature reaches
first, and stated as a **default rather than a rule**: the modifier below
overrides it.

The unmodified gesture copies. The argument is in
[`crate::text::doctabs::drag_landing_other`] and it is about undo, not
about caution: a move is two edits in two documents with one undo stack
each, and there is no ordering of them that makes one Ctrl+Z mean *"undo
what I just did"*. Windows Explorer copies between volumes for the same
reason.

**Holding Shift asks for the move anyway**, because an operator who wants
the pages out of the source is entitled to say so, and Shift is the key
that has meant *move* on this desktop since the mid-nineties.
[`crate::pagedrag::wants_move`] samples it live at the drop rather than
latching it at the press, so the caption changes under the operator's
hand while the key is held
([`crate::text::doctabs::drag_landing_move`], which says *REMOVED* and
names the source), and afterwards
[`crate::text::doctabs::moved_out_of`] states the undo consequence in
words. ⇒ **The disclosure is what makes the move offerable at all** — the
undo argument above is still true of it, so the operator is told, before
release and again after it.

Within one document the drag is the reorder it always was, a reorder is one
undoable command, and no modifier applies — there is nothing for it to
select between.

## Item notes

### `fn landing_shown_key`

## Why there are two slots and one rotation, rather than a shared flag

Two surfaces can resolve a landing — the Pages panel's grid and the page
view — and only one of them can have the pointer inside it, so at most one
writes per frame. The hard case is **neither**: the pointer is over the
ribbon, or a dock splitter, or off the window. Nobody writes, and nobody is
in a position to *clear* either, because "the pointer is not in my region"
is a thing every surface can say about itself and none can say about the
others. A surface that cleared on its own behalf would erase the answer the
other one had just written.

So the clear is not a surface's job. [`begin_frame`] rotates: whatever was
written last frame becomes what the caption reads, and the write slot goes
empty for this frame's surfaces to fill. One writer for the rotation, at a
known point, before anything draws.

That the caption is therefore **one frame late** is not a cost this design
introduced. It is the same one-frame lateness `PagesUi::drag_landing` was
documented as having, for the same unavoidable reason: *a gap has no
position until the rows have been placed, and the rows are placed below the
header.*

### `fn ending_a_drag_clears_where_it_would_have_landed`

The failure this closes is one `panels::pages` already names: a caret
that survives the gesture that produced it is a caret nobody can get
rid of. Here it would additionally make the status row describe a drop
that had already happened.

### `fn a_gap_maps_onto_an_insert_position`

The ends matter more than the middle: `End` survives the document
changing length between the gesture and the edit, and `Before(count)`
does not.
