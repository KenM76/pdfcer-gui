# `panels::bookmarks::reorder` — dragging a bookmark to a new place, and
the triangle that opens or closes one

## What this closes

Without a move verb an outline in the wrong **order** can only be repaired
by deleting a branch and re-authoring it, which loses every destination,
colour and style on it and is not an edit any operator would call a
reorganisation.

`EditSession::move_outline_item` and `EditSession::set_outline_open` are two
verbs and are **deliberately kept apart**, because whether a move should
reveal a collapsed destination has two defensible answers and the engine
declines to pick one for its callers.

⇒ **This module does not fold expansion into the move.** A drop into a
collapsed parent leaves that parent collapsed, exactly as
`EditSession::move_outline_item` does, and the operator is told the
bookmark went out of sight (see
[`crate::text::panels::bookmarks::bookmark_move_into_collapsed`]). The
triangle is the remedy and it is one click away. Two undo entries for two
acts is the honest count — the engine's own argument — and an operator who
did not want the expansion can undo it without undoing the move.

## The gesture is the conventional one, and it is copied rather than
invented

Every program with an outline panel — Acrobat's Bookmarks, Word's
navigation pane, every browser's bookmark manager, every IDE's file tree —
moves a row by **dragging it**, and shows an **insertion line** where it
will land. The operator's standing tie-breaker is *"make it work the way
other programs do"*, and there is no second answer here.

[`crate::panels::pages`] already implements exactly this shape for page
thumbnails, and this module follows it deliberately rather than inventing a
second drag idiom. The four properties carried across, each with its reason
restated where it differs:

| Property | Pages | Here |
|---|---|---|
| the target is resolved **during the layout pass** | a gap has no position until the grid is laid out | a row's band has no position until the tree is laid out, and the *end of a subtree* is not known until its children are drawn |
| the caret is a `Rect` carrying two endpoints, not a stroke | keeps geometry beside the tiles and appearance beside the theme | unchanged |
| it is **dimmed**, never hidden, where the drop would change nothing | drawing no caret cannot be told apart from the panel having stopped tracking the pointer, and the no-op boundary is where every drag begins | unchanged, and it carries a second dimmed state for a drop pdfcer will refuse |
| the release is read from **raw pointer input**, not from a `Response` | a drag that began on a row may end anywhere | unchanged |

## What a TREE needs that a grid does not: a depth

The pages grid has `n + 1` landings among `n` sheets, and a boundary is
fully described by which gap it is. An outline has the same `n + 1`
boundaries **and a depth at each one**, because the row below a bookmark
may be its child, its sibling, or its parent's next sibling, and all three
are different destinations.

The conventional resolution — Explorer's navigation pane, VS Code's
explorer, every tree control in every toolkit — is **three bands across the
row's height**:

| Band | Placement | Reads as |
|---|---|---|
| top quarter | [`OutlinePlacement::Before`] | *"in front of this one, beside it"* |
| middle half | [`OutlinePlacement::LastChild`] | *"inside this one"* |
| bottom quarter | [`OutlinePlacement::After`] | *"behind this one, beside it"* |

and **the caret's horizontal position is the depth**. A `Before`/`After`
caret starts at the row's own indent; an `Into` caret starts one indent
deeper. That is the whole of *"showing where it will land and at what
depth"* in one mark, with no second idiom to learn.

### Why the middle band is `LastChild` and not `FirstChild`

Because the caret must be drawn **where the bookmark will actually appear**,
and `LastChild` is the only choice that keeps the two lower bands at the
same height as each other. For a row that is open with children:

* `LastChild` lands at the end of that row's subtree — the caret goes at the
  bottom of the last descendant, indented one level;
* `After` lands after the whole subtree at the row's own level — the caret
  goes at the same height, indented one level *less*.

Two bands a few pixels apart, one caret height, and the **indent** is the
only thing that changes as the pointer crosses between them. `FirstChild`
would have put the middle band's caret immediately under the row and the
bottom band's caret at the end of the branch, so a two-pixel pointer
movement would fling the mark across the panel.

It is also `add_outline_item`'s own placement for a new bookmark, which
makes *"move it back where a fresh one would go"* expressible — the engine
names that as the reason [`OutlinePlacement::LastChild`] exists.

### The caret for the lower two bands sits at the END of the subtree,
which may be a long way from the pointer

That is deliberate and it is information rather than a defect. *"After this
chapter"* means *after everything in this chapter*, and an operator who is
shown the mark thirty rows down has just learned what the placement means
at the only moment they can still change their mind. Drawing the caret
beside the pointer would be comfortable and false.

The subtree's end is computed from the **flattened visible row list** — the
run of consecutive rows deeper than the anchor — which is why the rows are
collected during the walk and the target is resolved afterwards. A
collapsed row draws no children, so its subtree run is empty and its caret
is at its own bottom edge, which is exactly right: nothing is between them.

## `/Count` is two quantities and its SIGN is the open flag (§12.3.3)

Table 152 and Table 153 give the same key two meanings, and the item's
**sign** carries open-or-closed because there is no `/Open` key:

| | root `/Outlines` | an item |
|---|---|---|
| counts | all visible items, **including** the top level | visible **descendants**, excluding itself |
| sign | cannot be negative | **positive = open, negative = closed** |

Four consequences land in this file, and every one of them would be a defect
if it were missed:

1. **A collapsed row draws no children**, and [`super::rows`] has to honour
   that for the triangle to mean anything: a walk that recursed
   unconditionally would let a triangle write `/Count`'s sign, change the
   file and change nothing on screen — a control that appears not to work.
   See [`super`]'s header for the full note.
2. **Nothing here sizes anything from `/Count`.** `OutlineItem::open` is the
   shell's read of the sign and is the *only* field of it this module
   touches; `declared_count` is the file's own number carried verbatim and
   is not a count of anything this shell may size from, so
   [`super::tree::descendants`] walks the tree instead.
3. **The engine's move report counts what was VISIBLE.**
   `OutlineMove::visible_items` is the item plus its visible descendants —
   `1` for a collapsed chapter of forty sections. So the disclosure needs a
   second sentence for the collapsed case, and it comes from the tree rather
   than from the report. See
   [`crate::app::actions::bookmarks::BookmarkAction::Move`].
4. **A leaf has no `/Count` at all** — Table 153 requires the key only of an
   item that has descendants — so there is nothing to expand or collapse and
   no triangle is drawn. `set_outline_open` answers `Ok(false)` for a leaf
   rather than refusing, so a future *collapse all* may ask every row it
   walks; this module simply never asks.

## What is deliberately NOT done here

**Nothing mutates.** Both verbs leave through `actions`, as
[`crate::app::actions`]' `OVERVIEW.md` requires; this module raises
[`BookmarkAction::Move`] and [`BookmarkAction::SetOpen`] and touches the
document never. The one thing it writes is [`super::BookmarksUi`]'s own
drag slot, which is panel state and not document state.

**The drag does not cross documents.** [`crate::pagedrag`] publishes the
page drag into the `egui::Context` because a page can be dropped into
another open tab; an outline is a document-level structure and a bookmark
has no meaning in another file — its destination names a page of *this*
one. So the drag lives in the panel's own state, where a shorter life is
the honest one.
