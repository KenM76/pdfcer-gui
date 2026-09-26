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

## Item notes

### `const CARET_PTS`

[`crate::panels::pages`]' `CARET_PTS` verbatim, and deliberately the same
number: the two are the same mark meaning the same thing on two surfaces of
one application, and a reorder caret that was thinner in one panel than the
other would read as a rendering artefact rather than as a deliberate mark.

### `const CARET_DIMMED`

[`crate::panels::pages`]' `CARET_DIMMED`, for its stated reason: **dimmed,
not hidden.** Drawing no caret over a landing that would not move anything
cannot be told apart from the panel having stopped tracking the pointer —
and the no-op landing is where *every* drag begins, because a row starts out
hovering over itself.

### `const CARET_REFUSED`

Fainter than [`CARET_DIMMED`], and a third state rather than a reuse of
the second, because the two facts have different remedies. *"This changes
nothing"* is answered by letting go somewhere else at leisure; *"pdfcer will
not do this"* is answered by aiming outside the branch, and an operator who
reads the two marks as one will keep trying the same drop.

It is a ratio of the same theme colour rather than a second colour, which is
the rule `paint_caret` inherits from the pages grid: one colour with a
stated relationship beats two colours that have to be kept in step.

### `const EDGE_BAND`

A quarter each, leaving the middle **half** to `Into`. The asymmetry is
deliberate and is the conventional weighting: re-parenting is the gesture an
operator aims at a row, and reordering is the one they aim at a *boundary*,
which they do by moving toward the edge they can see. Equal thirds make the
nesting band harder to hit than the two it sits between, which is backwards.

### `fn is_inside`

The test `EditError::OutlineMoveIntoOwnSubtree` guards, asked of the tree
the panel drew rather than of `/Parent` chains in the file. It walks the
**whole** subtree, collapsed branches included, because a collapsed branch
is still a branch — a drop into a hidden descendant would produce exactly
the `/Parent` cycle the engine refuses.

### `fn placement_word`

A word rather than `{:?}`, because `OutlinePlacement`'s `Debug` prints the
anchor inside the variant and a driven check reading `placement=` would then
be matching on a rendering of a struct. The two facts are traced as two
keys, so a check can assert the *kind* of landing without pinning the
engine's derive output.

### `fn anchor_number`

Zero is not a legal object number — §7.3.10 numbers objects from 1 — so it
cannot be confused with a real anchor, and it is the same stand-in
`OutlinePlacement`'s `None` means: the top level.

### `fn the_row_splits_into_before_into_and_after`

The one piece of arithmetic the whole gesture rests on. Both plausible
errors are pinned: bands that are equal thirds — which makes the nesting
band, the one an operator aims *at a row*, harder to hit than the two
beside it — and an off-by-one at the boundary that would make a drop on
the exact midpoint mean something different from a drop a pixel away.

### `fn the_subtree_bottom_is_the_last_descendant_not_the_last_child`

The fixture is deliberately shaped so the two wrong answers differ from
the right one and from each other: a chapter with two sections, the
second of which has a section of its own, followed by a second chapter.
*"The bottom of the row"* (10) and *"the bottom of the last child"* (30)
are both wrong; the answer is the bottom of the last **descendant** (40).

### `fn a_collapsed_rows_caret_is_at_its_own_bottom`

The §12.3.3 case: the branch exists in the document and not on the
screen, and the caret is a mark on the screen. A build that walked the
TREE here instead of the drawn rows would put the mark under rows that
are not there.

### `fn a_nodes_place_is_found_at_any_depth`

The nested case is the one that matters, and it is why the whole panel
addresses bookmarks by id: a walk that loses track of depth files an
item a level or two from where it was asked to go, and the outline it
produces still looks entirely plausible, so nothing downstream reports
it.

### `fn the_slot_a_bookmark_already_occupies_is_recognised_from_both_sides`

This is the assertion the caret's dimming rests on, and it is the one a
naive implementation gets wrong: comparing anchor ids alone catches
*"drop on yourself"* and misses both of the real cases —
**after the previous sibling** and **before the next sibling** are the
slot the bookmark is already in, named from either side.

The fixture can tell the answers apart: three siblings, so the middle
one has a real neighbour on each side and a genuine move available past
each of them.

### `fn the_three_landings_are_distinguishable`

Each one paints a different caret and produces a different act on
release — a move, a silence, and a sentence — and two that compared
equal would make the release arm choose the wrong one of the three.

### `fn a_refused_landing_is_fainter_than_one_that_merely_does_nothing`

A build that dimmed a refusal and a no-op equally would give the
operator one mark for two facts with two different remedies — and they
would keep repeating the drop that pdfcer will never accept.

### `fn the_two_lower_bands_differ_by_an_indent_and_not_by_a_height`

Driven through [`resolve_at`] with a real row list, because this is the
property an operator reads off the screen: crossing from the middle band
to the bottom band must move the mark **sideways**, by exactly one
indent, and not vertically. A build that used `FirstChild` for the
middle band would fail this by flinging the caret up the panel.
