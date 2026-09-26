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

### `const REGION_CARET`

`ui_rect_visible` rather than `ui_rect`, for the reason `diag.rs`'s own
header records: this is drawn inside a `ScrollArea`, and a mark scrolled out
of view must not keep publishing a rectangle a driven check would then aim
at.

### `const REGION_DISCLOSE_PREFIX`

Keyed by object number rather than by position, for
[`super::BookmarksUi::selected`]'s reason: an id survives an edit and a
position does not. A check that expands a row and then re-aims must name the
same bookmark, and the row it sits on will have moved.

### `struct VisibleRow`

# Why the walk collects these instead of resolving the drop as it goes

Two answers are unavailable at the moment a row is drawn:

* **the end of its subtree**, which is where the `After` and `LastChild`
  carets belong — its children have not been laid out yet;
* **whether the row is the last of its level**, which decides nothing here
  but would have to be re-derived by any caller that wanted it.

Collecting the rows and resolving afterwards makes both a lookup in a flat
list. It is the same shape `crate::panels::pages`' `visible`, `go` and
`tokens` already have: an answer only the layout pass is in a position to
give, carried out of it rather than acted on inside it.

# Why it holds an `ObjId` and not a `&OutlineItem`

So it can be built in a test. `OutlineItem` is `#[non_exhaustive]` and this
crate cannot construct one — the same constraint that split
[`super::tree`]'s walks in two — and every geometric decision in this module
is made from these five fields and nothing else.

### `fn band_at`

Pure, and separated from everything that needs a `Ui` so the boundary
arithmetic — the part with something to get wrong — is testable. A zero- or
negative-height rect answers [`Band::Into`]: a row with no height cannot
have an edge, and answering *"the middle"* keeps the caret on the row the
pointer is over rather than inventing a boundary.

### `fn subtree_bottom`

The run of consecutive rows after `index` whose level is deeper than
`rows[index]`'s. A collapsed or childless row has an empty run and answers
its own bottom edge, which is correct: there is nothing drawn between it and
the next row at its level.

It reads the **drawn** rows and not the tree, which is the whole point. A
collapsed chapter has forty items under it in the document and none of them
on screen, and the caret is a mark on the screen.

Returns `rows[index].rect.bottom()` for an index past the end, which cannot
happen from [`resolve_at`] and is the answer that keeps a caller from
panicking if it ever does.

### `enum Landing`

# Why "changes nothing" and "would be refused" are separate

They have different remedies and, on release, they do different things.

A no-op is the operator asking for the state they are already in. The honest
response is to raise nothing and say nothing: the dimmed caret already said
so **before** the press, which is this panel's whole posture, and
[`crate::panels::pages`]' release makes the identical call for its own
no-op. Raising it anyway would put *"nothing changed"* in the status bar for
a gesture the panel had already declined to promise anything about, evicting
a real disclosure to do it.

A refusal is the operator asking for something pdfcer will not do, and **a
refusal must be a sentence, never a silence** — they will otherwise read it
as *"the drag did not register"* or, worse, as the move having succeeded
somewhere they cannot see, which is a real state this feature can produce.
So it **is** raised, the engine refuses it by name
(`EditError::OutlineMoveIntoOwnSubtree`), and
`crate::app::actions::bookmarks` words it. See [`settle`] for why the
sentence comes from there and not from here.

### `struct DropTarget`

[`crate::panels::pages`]' `DropTarget` with a depth added. The caret is a
`Rect` for that type's stated reason: it is a **line**, its two endpoints
are all the layout pass knows, and carrying them in one value keeps the
geometry decision beside the rows and the appearance decision beside the
theme.

### `struct Location`

The three facts [`landing_for`] needs to answer *"would this move change
anything?"*, and the reason they travel together is that they are one
lookup: a second walk to fetch the index after a first fetched the parent
is a second walk that can disagree with the first about which node it found.

### `fn locate_in`

Generic over the tree for [`super::tree::find_in`]'s reason. `parent` is the
id of the list's owner — `None` at the top level, which is the outline
root and is deliberately *not* given an id here: `read_outline` reports the
root's children as its top-level items and never exposes the root itself, so
there is no id to use and `None` is the only honest spelling.

### `fn landing_for`

# This is a FORECAST of the engine's answer, not a second copy of it

`move_outline_item` decides both facts for itself: it answers
`OutlineMove::moved = false` for a placement the bookmark already occupies,
writing no objects and creating no undo entry, and it refuses
`EditError::OutlineMoveIntoOwnSubtree` unconditionally.

The shell asks anyway, and the reason is the caret: a mark that could only
be drawn *after* the release would be no use at all. This is the same
relationship `panels::properties::formfield::refuses_delete` has with
`EditSession::deletion_refusal` — a query the shell can answer from what it
can see, standing in front of a guard that remains the authority. Where the
two disagree the engine wins, and the operator reads
[`crate::text::panels::bookmarks::bookmark_move_no_change`] or
[`crate::text::panels::bookmarks::bookmark_move_declined_engine`].

It is **not** R171 duplication, because the two are asked at different times
about different things: this one asks *"what should the mark under the
pointer look like?"* and the engine asks *"what shall I write?"*. Only the
second may change a document.

# The arithmetic, in one place

| placement | changes nothing when |
|---|---|
| `Before { sibling }` | `sibling` is the item, or the item's **next** sibling |
| `After { sibling }` | `sibling` is the item, or the item's **previous** sibling |
| `LastChild { parent }` | the item is already that parent's last child |
| `FirstChild { parent }` | the item is already that parent's first child |

Each is *"the slot the item is in, named from the other side"*, which is
why a lone `==` on ids is not enough: `After` the previous sibling and
`Before` the next sibling are both the item's own slot, and both would
otherwise draw a live caret over a drop that does nothing.

### `fn resolve_at`

Pure, and every geometric decision in this module is in it. `indent` is the
theme's own indent width, so an `Into` caret sits exactly where the row it
describes would be drawn; `right` is the panel's right edge, so the mark
spans the list rather than stopping under the longest title.

`None` when the pointer is over no row — including the space below the last
one, which is deliberately **not** treated as *"the end of the top level"*.
[`crate::panels::pages`]' release makes the same choice and states the
reason: a landing an operator reached by missing is a landing they did not
choose. The end of the top level is reachable, precisely, from the bottom
band of the last top-level row.

### `fn resolve`

Nothing is resolved unless a drag is actually in flight — the caret is a
mark about a gesture, and a panel that computed one every frame would be
paying for a question nobody asked.

### `fn paint_caret`

# Rule 4: this is the cursor, not a mark on content

[`crate::panels::pages`]' `paint_caret` argument, unchanged: a drop caret is
in the class the rule permits by name — *"snap indicators, hover highlights,
rubber-bands and selection handles are the cursor and are welcome"*. It
draws nothing into a page, changes no title, and disappears the instant the
pointer is released.

# The colour is the theme's, never a literal

[`egui_shell::theme::Theme::canvas_selection_ink`], the same source the
pages caret and the current-page ring take, so a preset that changes the
accent changes all three together. **Not `visuals().selection.stroke`** —
that is `egui`'s selected-*widget* channel, and a mark drawn over content is
not a widget, so a theme that restyled selected list rows would silently
restyle this caret with them. The two dimmed states are `gamma_multiply`
ratios of the one colour rather than two more colours: one colour with a
stated relationship beats several that have to be kept in step.

### `fn settle`

# Why the release is read from raw pointer input

[`crate::panels::pages`]' `settle_drag` discipline and its reason,
unchanged: a drag that began on a row may end anywhere — over the panel
header, past the end of the list, outside the window entirely — and a
`Response` only reports releases inside the widget that produced it. Reading
the input means a drag **always** ends, which is the property that stops a
half-finished gesture surviving into the next frame as a caret nobody can
get rid of.

# Why it runs unconditionally, and what each ending does

Because a drag that has started has to be able to end. The four endings:

| released | raises | says |
|---|---|---|
| over no row | nothing | nothing — the operator let go over empty space, which is how a drag is abandoned |
| on a landing that changes nothing | nothing | nothing — the dimmed caret said so before the press |
| on the bookmark itself or inside it | [`BookmarkAction::Move`] | **a sentence**, from the engine's refusal |
| anywhere else | [`BookmarkAction::Move`] | the engine's report, afterwards |

# Why a landing this module has already judged impossible is still
raised

It looks wasteful and it is the only correct shape. **A refusal must be a
sentence, never a silence**, and the channel for a decline is
`crate::app::status::decline`, which is `pub(super)` inside `crate::app`
because a decline is written by the one dispatcher and read by the one bar.
A panel is outside that boundary.

The two ways round it are both worse than going through it. A `record_note`
from here would render the sentence under **`⚑ About your last edit:`**,
which `crate::text::status` forbids for a decline: nothing was edited, and
an operator told otherwise after a gesture that did nothing has been lied to
confidently. Widening the module would trade a real invariant for one call
site.

⇒ So the action is raised, `EditSession::move_outline_item` refuses it by
name, `crate::app::actions::bookmarks::move_to` records the decline from
**inside** the closure, and nothing is written: the engine's guard runs
before it plans anything, so there is no epoch bump and no undo entry.

And it puts the authority where the module header already says it is.
[`landing_for`] is a **forecast**, and its whole purpose is the caret. The
engine's guard decides what happens, exactly as it does for every other
refusal in this shell, and the two cannot drift into disagreeing about the
*outcome* because only one of them produces it.
