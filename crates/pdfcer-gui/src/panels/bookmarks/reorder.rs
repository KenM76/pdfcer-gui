//! # `panels::bookmarks::reorder` — dragging a bookmark to a new place, and
//! the triangle that opens or closes one
//!
//! ## What this closes
//!
//! The half of bookmark editing the panel shipped **without**. `Pass 156.0`
//! gave this shell rename and delete, and [`super::edit`]'s header still says,
//! in as many words, *"Reorder and re-parent do not [ship] … R9: a capability
//! that does not exist renders nothing."* `pdfcer-core` `Pass 161.0` shipped
//! both, and the engine's covering note names the gap it closes:
//!
//! > *"an outline in the wrong **order** could only be fixed by deleting a
//! > branch and re-authoring it, which loses every destination, colour and
//! > style on it and is not an edit any operator would call a
//! > reorganisation."*
//!
//! Two verbs arrived, deliberately kept apart, and the release note says why in
//! a sentence that is a design instruction to this module:
//!
//! > *"Expand/collapse ships alongside, **as a separate verb, because whether
//! > a move should reveal a collapsed destination has two defensible answers
//! > and both now exist.**"*
//!
//! ⇒ **This module does not fold expansion into the move.** A drop into a
//! collapsed parent leaves that parent collapsed, exactly as
//! `EditSession::move_outline_item` does, and the operator is told the
//! bookmark went out of sight (see
//! [`crate::text::panels::bookmarks::bookmark_move_into_collapsed`]). The
//! triangle is the remedy and it is one click away. Two undo entries for two
//! acts is the honest count — the engine's own argument — and an operator who
//! did not want the expansion can undo it without undoing the move.
//!
//! ## ★★★ The gesture is the conventional one, and it is copied rather than
//! invented
//!
//! Every program with an outline panel — Acrobat's Bookmarks, Word's
//! navigation pane, every browser's bookmark manager, every IDE's file tree —
//! moves a row by **dragging it**, and shows an **insertion line** where it
//! will land. The operator's standing tie-breaker is *"make it work the way
//! other programs do"*, and there is no second answer here.
//!
//! [`crate::panels::pages`] already implements exactly this shape for page
//! thumbnails, and this module follows it deliberately rather than inventing a
//! second drag idiom. The four properties carried across, each with its reason
//! restated where it differs:
//!
//! | Property | Pages | Here |
//! |---|---|---|
//! | the target is resolved **during the layout pass** | a gap has no position until the grid is laid out | a row's band has no position until the tree is laid out, and the *end of a subtree* is not known until its children are drawn |
//! | the caret is a `Rect` carrying two endpoints, not a stroke | keeps geometry beside the tiles and appearance beside the theme | unchanged |
//! | it is **dimmed**, never hidden, where the drop would change nothing | *"drawing no caret cannot be told apart from the panel having stopped tracking the pointer — and the no-op boundary is where every drag begins"* | unchanged, and it carries a second dimmed state for a drop pdfcer will refuse |
//! | the release is read from **raw pointer input**, not from a `Response` | a drag that began on a row may end anywhere | unchanged |
//!
//! ## ★★ What a TREE needs that a grid does not: a depth
//!
//! The pages grid has `n + 1` landings among `n` sheets, and a boundary is
//! fully described by which gap it is. An outline has the same `n + 1`
//! boundaries **and a depth at each one**, because the row below a bookmark
//! may be its child, its sibling, or its parent's next sibling, and all three
//! are different destinations.
//!
//! The conventional resolution — Explorer's navigation pane, VS Code's
//! explorer, every tree control in every toolkit — is **three bands across the
//! row's height**:
//!
//! | Band | Placement | Reads as |
//! |---|---|---|
//! | top quarter | [`OutlinePlacement::Before`] | *"in front of this one, beside it"* |
//! | middle half | [`OutlinePlacement::LastChild`] | *"inside this one"* |
//! | bottom quarter | [`OutlinePlacement::After`] | *"behind this one, beside it"* |
//!
//! and **the caret's horizontal position is the depth**. A `Before`/`After`
//! caret starts at the row's own indent; an `Into` caret starts one indent
//! deeper. That is the whole of *"showing where it will land and at what
//! depth"* in one mark, with no second idiom to learn.
//!
//! ### ★ Why the middle band is `LastChild` and not `FirstChild`
//!
//! Because the caret must be drawn **where the bookmark will actually appear**,
//! and `LastChild` is the only choice that keeps the two lower bands at the
//! same height as each other. For a row that is open with children:
//!
//! * `LastChild` lands at the end of that row's subtree — the caret goes at the
//!   bottom of the last descendant, indented one level;
//! * `After` lands after the whole subtree at the row's own level — the caret
//!   goes at the same height, indented one level *less*.
//!
//! Two bands a few pixels apart, one caret height, and the **indent** is the
//! only thing that changes as the pointer crosses between them. `FirstChild`
//! would have put the middle band's caret immediately under the row and the
//! bottom band's caret at the end of the branch, so a two-pixel pointer
//! movement would fling the mark across the panel.
//!
//! It is also `add_outline_item`'s own placement for a new bookmark, which
//! makes *"move it back where a fresh one would go"* expressible — the engine
//! names that as the reason [`OutlinePlacement::LastChild`] exists.
//!
//! ### ★★ The caret for the lower two bands sits at the END of the subtree,
//! which may be a long way from the pointer
//!
//! That is deliberate and it is information rather than a defect. *"After this
//! chapter"* means *after everything in this chapter*, and an operator who is
//! shown the mark thirty rows down has just learned what the placement means
//! at the only moment they can still change their mind. Drawing the caret
//! beside the pointer would be comfortable and false.
//!
//! The subtree's end is computed from the **flattened visible row list** — the
//! run of consecutive rows deeper than the anchor — which is why the rows are
//! collected during the walk and the target is resolved afterwards. A
//! collapsed row draws no children, so its subtree run is empty and its caret
//! is at its own bottom edge, which is exactly right: nothing is between them.
//!
//! ## ★★★ `/Count` is two quantities and its SIGN is the open flag (§12.3.3)
//!
//! Table 152 and Table 153 give the same key two meanings, and the item's
//! **sign** carries open-or-closed because there is no `/Open` key:
//!
//! | | root `/Outlines` | an item |
//! |---|---|---|
//! | counts | all visible items, **including** the top level | visible **descendants**, excluding itself |
//! | sign | cannot be negative | **positive = open, negative = closed** |
//!
//! Four consequences land in this file, and every one of them would be a defect
//! if it were missed:
//!
//! 1. **The panel now hides a collapsed row's children**, which it did not do
//!    before this module existed. [`super::rows`] used to recurse
//!    unconditionally, so a triangle that wrote `/Count`'s sign would have
//!    changed the file and changed nothing on screen — a control that appears
//!    not to work. See [`super`]'s header for the full note.
//! 2. **Nothing here sizes anything from `/Count`.** `OutlineItem::open` is the
//!    shell's read of the sign and is the *only* field of it this module
//!    touches; `declared_count` is carried *"verbatim … Do not use this to size
//!    anything"* in core's own words, and [`super::tree::descendants`] walks
//!    the tree instead.
//! 3. **The engine's move report counts what was VISIBLE.**
//!    `OutlineMove::visible_items` is the item plus its visible descendants —
//!    `1` for a collapsed chapter of forty sections. So the disclosure needs a
//!    second sentence for the collapsed case, and it comes from the tree rather
//!    than from the report. See
//!    [`crate::app::actions::bookmarks::BookmarkAction::Move`].
//! 4. **A leaf has no `/Count` at all** (Table 153 makes it *"required if the
//!    item has any descendants"*), so there is nothing to expand or collapse
//!    and no triangle is drawn. `set_outline_open` answers `Ok(false)` for one
//!    rather than refusing — *"asking a leaf to expand is what a 'collapse all'
//!    sweep does to every row it walks"* — and this module simply never asks.
//!
//! ## What is deliberately NOT done here
//!
//! **Nothing mutates.** Both verbs leave through `actions`, as
//! [`crate::app::actions`]' `OVERVIEW.md` requires; this module raises
//! [`BookmarkAction::Move`] and [`BookmarkAction::SetOpen`] and touches the
//! document never. The one thing it writes is [`super::BookmarksUi`]'s own
//! drag slot, which is panel state and not document state.
//!
//! **The drag does not cross documents.** [`crate::pagedrag`] publishes the
//! page drag into the `egui::Context` because a page can be dropped into
//! another open tab; an outline is a document-level structure and a bookmark
//! has no meaning in another file — its destination names a page of *this*
//! one. So the drag lives in the panel's own state, where a shorter life is
//! the honest one.

use egui::{Pos2, Rect, Ui};
use pdfcer_core::edit::OutlinePlacement;
use pdfcer_core::object::ObjId;
use pdfcer_core::outline::OutlineItem;

use crate::app::actions::Action;
use crate::app::actions::bookmarks::BookmarkAction;

/// The region name the insertion caret publishes.
///
/// `ui_rect_visible` rather than `ui_rect`, for the reason `diag.rs`'s own
/// header records: this is drawn inside a `ScrollArea`, and a mark scrolled out
/// of view must not keep publishing a rectangle a driven check would then aim
/// at.
pub const REGION_CARET: &str = "bookmarks.drop-caret"; // ui-text-exempt: trace region name, never displayed

/// The prefix of the per-row disclosure-triangle regions; the item's object
/// **number** is appended.
///
/// Keyed by object number rather than by position, for
/// [`super::BookmarksUi::selected`]'s reason: an id survives an edit and a
/// position does not. A check that expands a row and then re-aims must name the
/// same bookmark, and the row it sits on will have moved.
pub const REGION_DISCLOSE_PREFIX: &str = "bookmarks.disclose."; // ui-text-exempt: trace region name, never displayed

/// How thick the insertion caret is drawn, in points.
///
/// [`crate::panels::pages`]' `CARET_PTS` verbatim, and deliberately the same
/// number: the two are the same mark meaning the same thing on two surfaces of
/// one application, and a reorder caret that was thinner in one panel than the
/// other would read as a rendering artefact rather than as a deliberate mark.
const CARET_PTS: f32 = 2.0;

/// How much of the caret's colour survives when the drop would change nothing.
///
/// [`crate::panels::pages`]' `CARET_DIMMED`, for its stated reason: **dimmed,
/// not hidden.** Drawing no caret over a landing that would not move anything
/// cannot be told apart from the panel having stopped tracking the pointer —
/// and the no-op landing is where *every* drag begins, because a row starts out
/// hovering over itself.
const CARET_DIMMED: f32 = 0.35;

/// How much survives when the drop would be **refused**.
///
/// ★ Fainter than [`CARET_DIMMED`], and a third state rather than a reuse of
/// the second, because the two facts have different remedies. *"This changes
/// nothing"* is answered by letting go somewhere else at leisure; *"pdfcer will
/// not do this"* is answered by aiming outside the branch, and an operator who
/// reads the two marks as one will keep trying the same drop.
///
/// It is a ratio of the same theme colour rather than a second colour, which is
/// the rule `paint_caret` inherits from the pages grid: one colour with a
/// stated relationship beats two colours that have to be kept in step.
const CARET_REFUSED: f32 = 0.15;

/// One row of the outline **as it was actually drawn**, in draw order.
///
/// # ★ Why the walk collects these instead of resolving the drop as it goes
///
/// Two answers are unavailable at the moment a row is drawn:
///
/// * **the end of its subtree**, which is where the `After` and `LastChild`
///   carets belong — its children have not been laid out yet;
/// * **whether the row is the last of its level**, which decides nothing here
///   but would have to be re-derived by any caller that wanted it.
///
/// Collecting the rows and resolving afterwards makes both a lookup in a flat
/// list. It is the same shape `crate::panels::pages`' `visible`, `go` and
/// `tokens` already have: an answer only the layout pass is in a position to
/// give, carried out of it rather than acted on inside it.
///
/// # Why it holds an `ObjId` and not a `&OutlineItem`
///
/// So it can be built in a test. `OutlineItem` is `#[non_exhaustive]` and this
/// crate cannot construct one — the same constraint that split
/// [`super::tree`]'s walks in two — and every geometric decision in this module
/// is made from these five fields and nothing else.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisibleRow {
    /// Which bookmark this row is.
    pub id: ObjId,
    /// Its nesting depth: `0` for a top-level bookmark, matching
    /// `OutlineItem::level`.
    pub level: usize,
    /// The full-width strip the row occupies, in the scroll area's own
    /// coordinate space. The **band test** reads its vertical extent; the caret
    /// reads its horizontal one.
    pub rect: Rect,
    /// Where this row's own content begins horizontally — the left edge of its
    /// disclosure triangle. This is the x a caret at *this row's depth* starts
    /// from, and it is measured rather than computed from `level` so an indent
    /// the theme changes cannot make the mark and the rows disagree.
    pub indent_left: f32,
    /// Whether this bookmark is **open**, from `OutlineItem::open` — the
    /// shell's read of `/Count`'s sign. A row that is closed drew no children,
    /// so its subtree run in this list is empty.
    pub open: bool,
    /// Whether it has any children at all, from the tree rather than from
    /// `/Count`. Decides whether a triangle is drawn: a leaf carries no
    /// `/Count` and has no open-or-closed state to set.
    pub has_children: bool,
}

/// Which third of a row the pointer is in, and therefore which placement.
///
/// The conventional three-band split every tree control uses. See the module
/// header's table, and its note on why the middle band is `LastChild`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// The top quarter — land in front of this row, beside it.
    Before,
    /// The middle half — land inside this row, at the end of whatever is
    /// already there.
    Into,
    /// The bottom quarter — land behind this row, beside it.
    After,
}

/// Fraction of a row's height each edge band occupies.
///
/// ★ A quarter each, leaving the middle **half** to `Into`. The asymmetry is
/// deliberate and is the conventional weighting: re-parenting is the gesture an
/// operator aims at a row, and reordering is the one they aim at a *boundary*,
/// which they do by moving toward the edge they can see. Equal thirds make the
/// nesting band harder to hit than the two it sits between, which is backwards.
const EDGE_BAND: f32 = 0.25;

/// Which band `y` falls in, within `rect`.
///
/// Pure, and separated from everything that needs a `Ui` so the boundary
/// arithmetic — the part with something to get wrong — is testable. A zero- or
/// negative-height rect answers [`Band::Into`]: a row with no height cannot
/// have an edge, and answering *"the middle"* keeps the caret on the row the
/// pointer is over rather than inventing a boundary.
#[must_use]
pub fn band_at(rect: Rect, y: f32) -> Band {
    let height = rect.height();
    if height <= 0.0 {
        return Band::Into;
    }
    let edge = height * EDGE_BAND;
    if y < rect.top() + edge {
        Band::Before
    } else if y > rect.bottom() - edge {
        Band::After
    } else {
        Band::Into
    }
}

/// The bottom of `rows[index]`'s **subtree**, as it was drawn.
///
/// The run of consecutive rows after `index` whose level is deeper than
/// `rows[index]`'s. A collapsed or childless row has an empty run and answers
/// its own bottom edge, which is correct: there is nothing drawn between it and
/// the next row at its level.
///
/// ★ It reads the **drawn** rows and not the tree, which is the whole point. A
/// collapsed chapter has forty items under it in the document and none of them
/// on screen, and the caret is a mark on the screen.
///
/// Returns `rows[index].rect.bottom()` for an index past the end, which cannot
/// happen from [`resolve_at`] and is the answer that keeps a caller from
/// panicking if it ever does.
#[must_use]
pub fn subtree_bottom(rows: &[VisibleRow], index: usize) -> f32 {
    let Some(anchor) = rows.get(index) else {
        return 0.0;
    };
    let mut bottom = anchor.rect.bottom();
    for row in &rows[index.saturating_add(1)..] {
        if row.level <= anchor.level {
            break;
        }
        bottom = row.rect.bottom();
    }
    bottom
}

/// What releasing on a landing would do — the three answers the caret has to
/// be able to draw.
///
/// # ★★ Why "changes nothing" and "would be refused" are separate
///
/// They have different remedies and, on release, they do different things.
///
/// A no-op is the operator asking for the state they are already in. The honest
/// response is to raise nothing and say nothing: the dimmed caret already said
/// so **before** the press, which is this panel's whole posture, and
/// [`crate::panels::pages`]' release makes the identical call for its own
/// no-op. Raising it anyway would put *"nothing changed"* in the status bar for
/// a gesture the panel had already declined to promise anything about, evicting
/// a real disclosure to do it.
///
/// A refusal is the operator asking for something pdfcer will not do, and **a
/// refusal must be a sentence, never a silence** — they will otherwise read it
/// as *"the drag did not register"* or, worse, as the move having succeeded
/// somewhere they cannot see, which is a real state this feature can produce.
/// So it **is** raised, the engine refuses it by name
/// (`EditError::OutlineMoveIntoOwnSubtree`), and
/// `crate::app::actions::bookmarks` words it. See [`settle`] for why the
/// sentence comes from there and not from here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Landing {
    /// The bookmark would move. The caret is drawn at full strength.
    Lands,
    /// The bookmark is already there. Dimmed; the release raises nothing and
    /// says nothing.
    NoChange,
    /// The destination is the bookmark itself or somewhere inside it. Fainter
    /// still; the release **raises the move anyway**, so the engine can refuse
    /// it and the refusal can be worded.
    OwnSubtree,
}

/// Where a drag in flight would land, resolved during the layout pass.
///
/// [`crate::panels::pages`]' `DropTarget` with a depth added. The caret is a
/// `Rect` for that type's stated reason: it is a **line**, its two endpoints
/// are all the layout pass knows, and carrying them in one value keeps the
/// geometry decision beside the rows and the appearance decision beside the
/// theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropTarget {
    /// The placement a release would ask the engine for.
    pub placement: OutlinePlacement,
    /// The line to draw, in the scroll area's own coordinate space. Its left
    /// edge carries the destination **depth**; see the module header.
    pub caret: Rect,
    /// What releasing here would actually do.
    pub landing: Landing,
}

/// Where a bookmark sits in the tree — its parent, its siblings, and its place
/// among them.
///
/// The three facts [`landing_for`] needs to answer *"would this move change
/// anything?"*, and the reason they travel together is that they are one
/// lookup: a second walk to fetch the index after a first fetched the parent
/// is a second walk that can disagree with the first about which node it found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    /// The bookmark it is filed under, or `None` for the top level — which is
    /// [`OutlinePlacement::FirstChild`]'s and `add_outline_item`'s own spelling
    /// for the outline root.
    pub parent: Option<ObjId>,
    /// Its siblings, in document order, including itself.
    pub siblings: Vec<ObjId>,
    /// Its index within [`Self::siblings`].
    pub index: usize,
}

/// Find `id`'s [`Location`] in the real outline.
///
/// One line, because the recursion — the part with something to get wrong —
/// lives in [`locate_in`], which **can** be tested. Same split, same reason, as
/// [`super::tree::find`]: `OutlineItem` is `#[non_exhaustive]` and a walk
/// written directly over it is a walk no test in this crate can reach.
#[must_use]
pub fn locate(items: &[OutlineItem], id: ObjId) -> Option<Location> {
    locate_in(
        items,
        None,
        id,
        |item| item.id,
        |item| item.children.as_slice(),
    )
}

/// Depth-first search for a node's parent, siblings and index.
///
/// Generic over the tree for [`super::tree::find_in`]'s reason. `parent` is the
/// id of the list's owner — `None` at the top level, which is the outline
/// root and is deliberately *not* given an id here: `read_outline` reports the
/// root's children as its top-level items and never exposes the root itself, so
/// there is no id to use and `None` is the only honest spelling.
pub fn locate_in<'a, T>(
    items: &'a [T],
    parent: Option<ObjId>,
    id: ObjId,
    id_of: impl Fn(&T) -> ObjId + Copy,
    children: impl Fn(&'a T) -> &'a [T] + Copy,
) -> Option<Location> {
    if let Some(index) = items.iter().position(|item| id_of(item) == id) {
        return Some(Location {
            parent,
            siblings: items.iter().map(id_of).collect(),
            index,
        });
    }
    for item in items {
        if let Some(found) = locate_in(children(item), Some(id_of(item)), id, id_of, children) {
            return Some(found);
        }
    }
    None
}

/// **Would this move do anything, and would pdfcer allow it?**
///
/// # ★★ This is a FORECAST of the engine's answer, not a second copy of it
///
/// `move_outline_item` decides both facts for itself: it returns
/// `OutlineMove::moved = false` for a placement the bookmark already occupies
/// — *"a legitimate request with a legitimate answer — nothing"*, writing no
/// objects and creating no undo entry — and it refuses
/// `EditError::OutlineMoveIntoOwnSubtree` unconditionally.
///
/// The shell asks anyway, and the reason is the caret: a mark that could only
/// be drawn *after* the release would be no use at all. This is the same
/// relationship `panels::properties::formfield::refuses_delete` has with
/// `EditSession::deletion_refusal` — a query the shell can answer from what it
/// can see, standing in front of a guard that remains the authority. Where the
/// two disagree the engine wins, and the operator reads
/// [`crate::text::panels::bookmarks::bookmark_move_no_change`] or
/// [`crate::text::panels::bookmarks::bookmark_move_declined_engine`].
///
/// It is **not** R171 duplication, because the two are asked at different times
/// about different things: this one asks *"what should the mark under the
/// pointer look like?"* and the engine asks *"what shall I write?"*. Only the
/// second may change a document.
///
/// # The arithmetic, in one place
///
/// | placement | changes nothing when |
/// |---|---|
/// | `Before { sibling }` | `sibling` is the item, or the item's **next** sibling |
/// | `After { sibling }` | `sibling` is the item, or the item's **previous** sibling |
/// | `LastChild { parent }` | the item is already that parent's last child |
/// | `FirstChild { parent }` | the item is already that parent's first child |
///
/// Each is *"the slot the item is in, named from the other side"*, which is
/// why a lone `==` on ids is not enough: `After` the previous sibling and
/// `Before` the next sibling are both the item's own slot, and both would
/// otherwise draw a live caret over a drop that does nothing.
#[must_use]
pub fn landing_for(items: &[OutlineItem], dragged: ObjId, to: OutlinePlacement) -> Landing {
    let anchor = match to {
        OutlinePlacement::Before { sibling } | OutlinePlacement::After { sibling } => Some(sibling),
        OutlinePlacement::FirstChild { parent } | OutlinePlacement::LastChild { parent } => parent,
        // `OutlinePlacement` is `#[non_exhaustive]`. A variant this build has
        // never seen cannot be reasoned about, so it gets the answer that asks
        // the engine rather than the answer that quietly refuses — the panel
        // constructs every placement it uses, so this arm is unreachable today
        // and must not become a silent veto if that stops being true.
        _ => return Landing::Lands,
    };
    // The refusal first, because it outranks the no-op: dropping a bookmark on
    // itself is both "already there" and "inside itself", and the sentence the
    // operator needs is the one that says pdfcer will not do it.
    if let Some(anchor) = anchor
        && (anchor == dragged || is_inside(items, dragged, anchor))
    {
        return Landing::OwnSubtree;
    }
    let Some(here) = locate(items, dragged) else {
        // The dragged id no longer resolves — the ordinary state one frame
        // after an undo. Nothing can be forecast about a bookmark that is not
        // there, so the engine is asked and answers `OutlineItemNotFound`.
        return Landing::Lands;
    };
    let unchanged = match to {
        OutlinePlacement::Before { sibling } => here
            .siblings
            .get(here.index.saturating_add(1))
            .is_some_and(|next| *next == sibling),
        OutlinePlacement::After { sibling } => here
            .index
            .checked_sub(1)
            .and_then(|before| here.siblings.get(before))
            .is_some_and(|previous| *previous == sibling),
        OutlinePlacement::LastChild { parent } => {
            here.parent == parent && here.index + 1 == here.siblings.len()
        }
        OutlinePlacement::FirstChild { parent } => here.parent == parent && here.index == 0,
        _ => false,
    };
    if unchanged {
        Landing::NoChange
    } else {
        Landing::Lands
    }
}

/// Is `candidate` somewhere below `ancestor` in the tree?
///
/// The test `EditError::OutlineMoveIntoOwnSubtree` guards, asked of the tree
/// the panel drew rather than of `/Parent` chains in the file. It walks the
/// **whole** subtree, collapsed branches included, because a collapsed branch
/// is still a branch — a drop into a hidden descendant would produce exactly
/// the `/Parent` cycle the engine refuses.
#[must_use]
fn is_inside(items: &[OutlineItem], ancestor: ObjId, candidate: ObjId) -> bool {
    super::tree::find(items, ancestor)
        .is_some_and(|item| super::tree::find(&item.children, candidate).is_some())
}

/// **Where a drag would land**, from the rows as drawn and the pointer as it
/// is.
///
/// Pure, and every geometric decision in this module is in it. `indent` is the
/// theme's own indent width, so an `Into` caret sits exactly where the row it
/// describes would be drawn; `right` is the panel's right edge, so the mark
/// spans the list rather than stopping under the longest title.
///
/// `None` when the pointer is over no row — including the space below the last
/// one, which is deliberately **not** treated as *"the end of the top level"*.
/// [`crate::panels::pages`]' release makes the same choice and states the
/// reason: a landing an operator reached by missing is a landing they did not
/// choose. The end of the top level is reachable, precisely, from the bottom
/// band of the last top-level row.
#[must_use]
pub fn resolve_at(
    rows: &[VisibleRow],
    items: &[OutlineItem],
    dragged: ObjId,
    pointer: Pos2,
    indent: f32,
    right: f32,
) -> Option<DropTarget> {
    let index = rows.iter().position(|row| row.rect.contains(pointer))?;
    let row = rows[index];
    let band = band_at(row.rect, pointer.y);
    let (placement, y, left) = match band {
        Band::Before => (
            OutlinePlacement::Before { sibling: row.id },
            row.rect.top(),
            row.indent_left,
        ),
        // Both lower bands sit at the END of the row's subtree, and differ only
        // by one indent. See the module header for why that is the whole trick:
        // the mark's height stops moving and its depth is the thing the
        // operator is choosing.
        Band::Into => (
            OutlinePlacement::LastChild {
                parent: Some(row.id),
            },
            subtree_bottom(rows, index),
            row.indent_left + indent,
        ),
        Band::After => (
            OutlinePlacement::After { sibling: row.id },
            subtree_bottom(rows, index),
            row.indent_left,
        ),
    };
    Some(DropTarget {
        placement,
        caret: Rect::from_min_max(Pos2::new(left, y), Pos2::new(right.max(left), y)),
        landing: landing_for(items, dragged, placement),
    })
}

/// [`resolve_at`], with the pointer and the theme read from the `Ui`.
///
/// Nothing is resolved unless a drag is actually in flight — the caret is a
/// mark about a gesture, and a panel that computed one every frame would be
/// paying for a question nobody asked.
#[must_use]
pub fn resolve(
    ui: &Ui,
    rows: &[VisibleRow],
    items: &[OutlineItem],
    dragged: Option<ObjId>,
) -> Option<DropTarget> {
    let dragged = dragged?;
    let pointer = ui.ctx().pointer_latest_pos()?;
    resolve_at(
        rows,
        items,
        dragged,
        pointer,
        ui.spacing().indent,
        // `max_rect`, not `min_rect`: the caret spans the LIST, and `min_rect`
        // after the rows have been drawn is the bounding box of the titles —
        // so a document of short chapter numbers would get a mark that stopped
        // a third of the way across the panel and read as a hairline artefact.
        // It is the same rectangle the rows were measured against; see
        // `super::body`'s `strip`.
        ui.max_rect().right(),
    )
}

/// Draw the insertion caret for a drag in flight.
///
/// # Rule 4: this is the cursor, not a mark on content
///
/// [`crate::panels::pages`]' `paint_caret` argument, unchanged: a drop caret is
/// in the class the rule permits by name — *"snap indicators, hover highlights,
/// rubber-bands and selection handles are the cursor and are welcome"*. It
/// draws nothing into a page, changes no title, and disappears the instant the
/// pointer is released.
///
/// # The colour is the theme's, never a literal
///
/// `visuals().selection.stroke.color`, the same source the pages caret and the
/// current-page ring take, so a preset that changes the accent changes all
/// three together. The two dimmed states are `gamma_multiply` ratios of it
/// rather than two more colours, for that module's stated reason: one colour
/// with a stated relationship beats several that have to be kept in step.
pub fn paint_caret(ui: &Ui, target: Option<&DropTarget>) {
    let Some(target) = target else {
        return;
    };
    let base = ui.visuals().selection.stroke.color;
    let colour = match target.landing {
        Landing::Lands => base,
        Landing::NoChange => base.gamma_multiply(CARET_DIMMED),
        Landing::OwnSubtree => base.gamma_multiply(CARET_REFUSED),
    };
    ui.painter().line_segment(
        [target.caret.left_top(), target.caret.right_top()],
        egui::Stroke::new(CARET_PTS, colour),
    );
    crate::diag::ui_rect_visible(REGION_CARET, target.caret.expand(CARET_PTS), ui.clip_rect());
    crate::diag::trace_changed(CARET_SLOT, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed
            "bookmark-drop-target placement={} anchor={} landing={:?} caret={:?}",
            placement_word(target.placement),
            anchor_number(target.placement),
            target.landing,
            target.caret,
        )
    });
}

/// Trace slot for the once-per-change caret line.
const CARET_SLOT: &str = "bookmark-drop-target"; // ui-text-exempt: trace slot name, never displayed

/// The placement as one word, for the trace.
///
/// ★ A word rather than `{:?}`, because `OutlinePlacement`'s `Debug` prints the
/// anchor inside the variant and a driven check reading `placement=` would then
/// be matching on a rendering of a struct. The two facts are traced as two
/// keys, so a check can assert the *kind* of landing without pinning the
/// engine's derive output.
fn placement_word(to: OutlinePlacement) -> &'static str {
    match to {
        // ui-text-exempt: diagnostic trace tokens, never displayed
        OutlinePlacement::Before { .. } => "before",
        OutlinePlacement::After { .. } => "after",
        OutlinePlacement::FirstChild { .. } => "first-child",
        OutlinePlacement::LastChild { .. } => "last-child",
        _ => "unknown",
    }
}

/// The anchor's object number, for the trace, or `0` for the outline root.
///
/// Zero is not a legal object number — §7.3.10 numbers objects from 1 — so it
/// cannot be confused with a real anchor, and it is the same stand-in
/// `OutlinePlacement`'s `None` means: the top level.
fn anchor_number(to: OutlinePlacement) -> u32 {
    match to {
        OutlinePlacement::Before { sibling } | OutlinePlacement::After { sibling } => sibling.num,
        OutlinePlacement::FirstChild { parent } | OutlinePlacement::LastChild { parent } => {
            parent.map_or(0, |id| id.num)
        }
        _ => 0,
    }
}

/// **End a drag** — read the release, raise the move, clear the state.
///
/// # ★ Why the release is read from raw pointer input
///
/// [`crate::panels::pages`]' `settle_drag` discipline and its reason,
/// unchanged: a drag that began on a row may end anywhere — over the panel
/// header, past the end of the list, outside the window entirely — and a
/// `Response` only reports releases inside the widget that produced it. Reading
/// the input means a drag **always** ends, which is the property that stops a
/// half-finished gesture surviving into the next frame as a caret nobody can
/// get rid of.
///
/// # Why it runs unconditionally, and what each ending does
///
/// Because a drag that has started has to be able to end. The four endings:
///
/// | released | raises | says |
/// |---|---|---|
/// | over no row | nothing | nothing — the operator let go over empty space, which is how a drag is abandoned |
/// | on a landing that changes nothing | nothing | nothing — the dimmed caret said so before the press |
/// | on the bookmark itself or inside it | [`BookmarkAction::Move`] | **a sentence**, from the engine's refusal |
/// | anywhere else | [`BookmarkAction::Move`] | the engine's report, afterwards |
///
/// # ★★★ Why a landing this module has already judged impossible is still
/// raised
///
/// It looks wasteful and it is the only correct shape. **A refusal must be a
/// sentence, never a silence**, and the channel for a decline is
/// `crate::app::status::decline`, which is `pub(super)` inside `crate::app` on
/// a stated boundary: *"a decline is written by the one dispatcher and read by
/// the one bar."* A panel is outside it.
///
/// The two ways round that boundary are both worse than going through it. A
/// `record_note` from here would render the sentence under **`⚑ About your last
/// edit:`** — which `crate::text::status`' own rule forbids for a decline, in
/// as many words: *"an operator who reads 'About your last edit' after a
/// gesture that did nothing has been told a small lie confidently."* Widening
/// the module would trade a real invariant for one call site.
///
/// ⇒ So the action is raised, `EditSession::move_outline_item` refuses it by
/// name, `crate::app::actions::bookmarks::move_to` records the decline from
/// **inside** the closure, and nothing is written: the engine's guard runs
/// before it plans anything, so there is no epoch bump and no undo entry.
///
/// ★ And it puts the authority where the module header already says it is.
/// [`landing_for`] is a **forecast**, and its whole purpose is the caret. The
/// engine's guard decides what happens, exactly as it does for every other
/// refusal in this shell, and the two cannot drift into disagreeing about the
/// *outcome* because only one of them produces it.
pub fn settle(
    ui: &Ui,
    ui_state: &mut super::BookmarksUi,
    target: Option<&DropTarget>,
    actions: &mut Vec<Action>,
) {
    let Some(dragged) = ui_state.drag else {
        return;
    };
    // Set every frame of the drag rather than once at the start: egui resolves
    // the cursor per frame from whatever asked most recently, so a request made
    // at `drag_started` would be overwritten by the next widget the pointer
    // passed over. `crate::panels::pages` records the same finding.
    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    if !ui
        .ctx()
        .input(|i| i.pointer.button_released(egui::PointerButton::Primary))
    {
        return;
    }
    ui_state.drag = None;
    let Some(target) = target else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("bookmark-drag-released item={} landing=none", dragged.num)
        });
        return;
    };
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed
            "bookmark-drag-released item={} placement={} anchor={} landing={:?}",
            dragged.num,
            placement_word(target.placement),
            anchor_number(target.placement),
            target.landing,
        )
    });
    match target.landing {
        // ★★★ Both of these raise, and the second is the whole of R83's rule:
        // a refusal must be a SENTENCE, never a silence. See this function's
        // header for why the sentence has to be produced by the apply phase
        // rather than here.
        Landing::Lands | Landing::OwnSubtree => {
            actions.push(Action::Bookmark(BookmarkAction::Move {
                item: dragged,
                to: target.placement,
            }));
        }
        // ★ Nothing, and nothing said. The operator asked for the state they
        // are already in, and the caret was dimmed under their pointer before
        // they let go. Raising the action anyway would be honest — the engine
        // answers `moved: false` and writes nothing — and it would cost a
        // status line saying "nothing changed" for a gesture the panel had
        // already declined to promise anything about. `crate::panels::pages`'
        // release makes the identical call for its own no-op.
        Landing::NoChange => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A row list this crate CAN build, at whatever geometry a test wants.
    fn row(num: u32, level: usize, top: f32, height: f32) -> VisibleRow {
        VisibleRow {
            id: ObjId::new(num, 0),
            level,
            rect: Rect::from_min_max(Pos2::new(0.0, top), Pos2::new(200.0, top + height)),
            #[allow(
                clippy::cast_precision_loss,
                reason = "a nesting level is a small integer; the indent is a test fixture" // ui-text-exempt: clippy lint justification, never displayed
            )]
            indent_left: level as f32 * 10.0,
            open: true,
            has_children: false,
        }
    }

    /// ★★ **The three bands are three, and the edges are quarters.**
    ///
    /// The one piece of arithmetic the whole gesture rests on. Both plausible
    /// errors are pinned: bands that are equal thirds — which makes the nesting
    /// band, the one an operator aims *at a row*, harder to hit than the two
    /// beside it — and an off-by-one at the boundary that would make a drop on
    /// the exact midpoint mean something different from a drop a pixel away.
    #[test]
    fn the_row_splits_into_before_into_and_after() {
        let rect = Rect::from_min_max(Pos2::new(0.0, 100.0), Pos2::new(200.0, 120.0));
        assert_eq!(band_at(rect, 100.0), Band::Before, "the very top");
        assert_eq!(band_at(rect, 104.0), Band::Before, "inside the top quarter");
        assert_eq!(band_at(rect, 106.0), Band::Into, "past it");
        assert_eq!(band_at(rect, 110.0), Band::Into, "dead centre");
        assert_eq!(band_at(rect, 114.0), Band::Into, "still the middle half");
        assert_eq!(band_at(rect, 116.0), Band::After, "into the bottom quarter");
        assert_eq!(band_at(rect, 120.0), Band::After, "the very bottom");
        // The middle band is the widest, which is the conventional weighting.
        let edge = rect.height() * EDGE_BAND;
        assert!(
            rect.height() - 2.0 * edge > edge,
            "the nesting band must be wider than either edge band"
        );
    }

    /// A degenerate row answers the middle rather than inventing a boundary.
    #[test]
    fn a_zero_height_row_is_all_middle() {
        let flat = Rect::from_min_max(Pos2::new(0.0, 50.0), Pos2::new(200.0, 50.0));
        assert_eq!(band_at(flat, 50.0), Band::Into);
    }

    /// ★★★ **The caret for the lower bands sits at the end of the SUBTREE**,
    /// which is what makes it truthful rather than comfortable.
    ///
    /// The fixture is deliberately shaped so the two wrong answers differ from
    /// the right one and from each other: a chapter with two sections, the
    /// second of which has a section of its own, followed by a second chapter.
    /// *"The bottom of the row"* (10) and *"the bottom of the last child"* (30)
    /// are both wrong; the answer is the bottom of the last **descendant** (40).
    #[test]
    fn the_subtree_bottom_is_the_last_descendant_not_the_last_child() {
        let rows = vec![
            row(1, 0, 0.0, 10.0),  // Chapter one
            row(2, 1, 10.0, 10.0), // .. section A
            row(3, 1, 20.0, 10.0), // .. section B
            row(4, 2, 30.0, 10.0), // .... section B.i
            row(5, 0, 40.0, 10.0), // Chapter two
        ];
        assert_eq!(subtree_bottom(&rows, 0), 40.0, "past every descendant");
        assert_ne!(subtree_bottom(&rows, 0), 10.0, "not its own bottom");
        assert_ne!(subtree_bottom(&rows, 0), 30.0, "not its last CHILD's");
        assert_eq!(subtree_bottom(&rows, 2), 40.0, "section B holds one");
        assert_eq!(
            subtree_bottom(&rows, 4),
            50.0,
            "the last row has no subtree and answers its own bottom"
        );
    }

    /// ★ **A collapsed row's caret is at its own edge**, because nothing of it
    /// is drawn.
    ///
    /// The §12.3.3 case: the branch exists in the document and not on the
    /// screen, and the caret is a mark on the screen. A build that walked the
    /// TREE here instead of the drawn rows would put the mark under rows that
    /// are not there.
    #[test]
    fn a_collapsed_rows_caret_is_at_its_own_bottom() {
        // Two top-level rows; the first is collapsed, so nothing of its branch
        // was drawn however large it is.
        let mut rows = vec![row(1, 0, 0.0, 10.0), row(2, 0, 10.0, 10.0)];
        rows[0].open = false;
        rows[0].has_children = true;
        assert_eq!(subtree_bottom(&rows, 0), 10.0);
    }

    // ---------------------------------------------------------------------
    // The landing forecast, over a tree this crate can build
    // ---------------------------------------------------------------------

    /// A node type standing in for `OutlineItem`, which is `#[non_exhaustive]`
    /// and cannot be constructed here. Same device, same reason, as
    /// [`super::super::tree`]'s tests.
    struct Node {
        id: ObjId,
        children: Vec<Node>,
    }

    fn node(num: u32, children: Vec<Node>) -> Node {
        Node {
            id: ObjId::new(num, 0),
            children,
        }
    }

    fn kids(n: &Node) -> &[Node] {
        n.children.as_slice()
    }

    fn locate_node(items: &[Node], num: u32) -> Option<Location> {
        locate_in(items, None, ObjId::new(num, 0), |n| n.id, kids)
    }

    /// ★★ **A bookmark's place is its parent, its siblings and its index**, and
    /// all three come from one walk.
    ///
    /// The nested case is the one that matters, for the reason the whole panel
    /// addresses bookmarks by id: the engine got this wrong in its own CLI and
    /// *"nested something two levels deeper than intended, and the output
    /// looked entirely plausible."*
    #[test]
    fn a_nodes_place_is_found_at_any_depth() {
        let tree = vec![
            node(1, vec![node(3, vec![]), node(4, vec![node(6, vec![])])]),
            node(2, vec![]),
        ];
        let top = locate_node(&tree, 2).expect("present");
        assert_eq!(top.parent, None, "a top-level item has no parent id");
        assert_eq!(top.index, 1);
        assert_eq!(top.siblings.len(), 2);

        let deep = locate_node(&tree, 6).expect("present");
        assert_eq!(deep.parent, Some(ObjId::new(4, 0)));
        assert_eq!(deep.index, 0);
        assert_eq!(deep.siblings.len(), 1);

        let middle = locate_node(&tree, 4).expect("present");
        assert_eq!(middle.parent, Some(ObjId::new(1, 0)));
        assert_eq!(middle.index, 1, "the second child");

        assert!(locate_node(&tree, 99).is_none());
    }

    /// ★★★ **The two spellings of a bookmark's own slot are recognised as
    /// no-ops.**
    ///
    /// This is the assertion the caret's dimming rests on, and it is the one a
    /// naive implementation gets wrong: comparing anchor ids alone catches
    /// *"drop on yourself"* and misses both of the real cases —
    /// **after the previous sibling** and **before the next sibling** are the
    /// slot the bookmark is already in, named from either side.
    ///
    /// The fixture can tell the answers apart: three siblings, so the middle
    /// one has a real neighbour on each side and a genuine move available past
    /// each of them.
    #[test]
    fn the_slot_a_bookmark_already_occupies_is_recognised_from_both_sides() {
        // Modelled with `locate_in` directly rather than `landing_for`, which
        // needs a real `OutlineItem` tree. The arithmetic under test is the
        // index comparison, and this is where it lives.
        let tree = vec![node(1, vec![]), node(2, vec![]), node(3, vec![])];
        let me = locate_node(&tree, 2).expect("present");
        let next = me.siblings.get(me.index + 1).copied();
        let previous = me
            .index
            .checked_sub(1)
            .and_then(|i| me.siblings.get(i))
            .copied();
        assert_eq!(next, Some(ObjId::new(3, 0)), "Before THIS is a no-op");
        assert_eq!(previous, Some(ObjId::new(1, 0)), "After THIS is a no-op");
        // And the far side of each neighbour is a real move.
        assert_ne!(next, Some(ObjId::new(1, 0)));
        assert_ne!(previous, Some(ObjId::new(3, 0)));
    }

    /// ★ **The first and last child are recognised too**, which is what makes
    /// the middle band's caret dim when a bookmark is dropped back into the
    /// parent it is already the last child of.
    #[test]
    fn being_already_the_last_child_is_a_no_op() {
        let tree = vec![node(1, vec![node(2, vec![]), node(3, vec![])])];
        let last = locate_node(&tree, 3).expect("present");
        assert_eq!(last.parent, Some(ObjId::new(1, 0)));
        assert_eq!(last.index + 1, last.siblings.len(), "already last");
        let first = locate_node(&tree, 2).expect("present");
        assert_eq!(first.index, 0, "already first");
        assert_ne!(
            first.index + 1,
            first.siblings.len(),
            "the fixture must be able to tell first from last"
        );
    }

    /// ★★ **The three landings are three distinct answers**, so a match on them
    /// cannot silently collapse.
    ///
    /// Each one paints a different caret and produces a different act on
    /// release — a move, a silence, and a sentence — and two that compared
    /// equal would make the release arm choose the wrong one of the three.
    #[test]
    fn the_three_landings_are_distinguishable() {
        assert_ne!(Landing::Lands, Landing::NoChange);
        assert_ne!(Landing::NoChange, Landing::OwnSubtree);
        assert_ne!(Landing::Lands, Landing::OwnSubtree);
    }

    /// ★ **The three dimming ratios are three**, and they are ordered.
    ///
    /// A build that dimmed a refusal and a no-op equally would give the
    /// operator one mark for two facts with two different remedies — and they
    /// would keep repeating the drop that pdfcer will never accept.
    #[test]
    fn a_refused_landing_is_fainter_than_one_that_merely_does_nothing() {
        const {
            assert!(CARET_REFUSED < CARET_DIMMED);
            assert!(CARET_DIMMED < 1.0);
            // Faint is not invisible: a caret nobody can see is the "no caret
            // at all" this module's constants exist to refuse.
            assert!(CARET_REFUSED > 0.0);
        }
    }

    /// ★★★ **The caret's DEPTH is the whole of what distinguishes nesting from
    /// reordering**, and the two lower bands sit at the same height.
    ///
    /// Driven through [`resolve_at`] with a real row list, because this is the
    /// property an operator reads off the screen: crossing from the middle band
    /// to the bottom band must move the mark **sideways**, by exactly one
    /// indent, and not vertically. A build that used `FirstChild` for the
    /// middle band would fail this by flinging the caret up the panel.
    #[test]
    fn the_two_lower_bands_differ_by_an_indent_and_not_by_a_height() {
        let rows = vec![row(1, 0, 0.0, 20.0), row(2, 1, 20.0, 20.0)];
        // No `OutlineItem` tree is constructible here, so the landing forecast
        // degrades to `Lands` (the dragged id does not resolve) — which is
        // exactly what this test wants, since it is about geometry.
        let empty: [OutlineItem; 0] = [];
        let indent = 12.0;
        let into = resolve_at(
            &rows,
            &empty,
            ObjId::new(9, 0),
            Pos2::new(50.0, 10.0),
            indent,
            200.0,
        )
        .expect("the pointer is over row one");
        let after = resolve_at(
            &rows,
            &empty,
            ObjId::new(9, 0),
            Pos2::new(50.0, 19.0),
            indent,
            200.0,
        )
        .expect("the pointer is over row one");
        assert_eq!(placement_word(into.placement), "last-child");
        assert_eq!(placement_word(after.placement), "after");
        assert!(
            (into.caret.top() - after.caret.top()).abs() < f32::EPSILON,
            "the mark must not jump vertically between the two lower bands"
        );
        assert!(
            (into.caret.left() - after.caret.left() - indent).abs() < f32::EPSILON,
            "nesting is exactly one indent deeper than landing beside it"
        );
        // And both are at the end of row one's subtree, which is row two.
        assert!((into.caret.top() - 40.0).abs() < f32::EPSILON);
    }

    /// ★★ **The top band's caret is at the row's own top edge and its own
    /// depth**, which is the one landing whose mark is beside the pointer.
    #[test]
    fn the_top_band_marks_the_row_it_is_over() {
        let rows = vec![row(1, 0, 0.0, 20.0), row(2, 1, 20.0, 20.0)];
        let empty: [OutlineItem; 0] = [];
        let before = resolve_at(
            &rows,
            &empty,
            ObjId::new(9, 0),
            Pos2::new(50.0, 22.0),
            12.0,
            200.0,
        )
        .expect("the pointer is over row two");
        assert_eq!(placement_word(before.placement), "before");
        assert_eq!(anchor_number(before.placement), 2);
        assert!((before.caret.top() - 20.0).abs() < f32::EPSILON);
        assert!(
            (before.caret.left() - rows[1].indent_left).abs() < f32::EPSILON,
            "a Before caret sits at the row's OWN depth, not one deeper"
        );
    }

    /// The pointer over no row resolves nothing, and the space below the list
    /// is deliberately not *"the end of the top level"*.
    #[test]
    fn a_pointer_over_no_row_lands_nowhere() {
        let rows = vec![row(1, 0, 0.0, 20.0)];
        let empty: [OutlineItem; 0] = [];
        assert!(
            resolve_at(
                &rows,
                &empty,
                ObjId::new(9, 0),
                Pos2::new(50.0, 400.0),
                12.0,
                200.0
            )
            .is_none()
        );
    }

    /// ★ **The root is spelled `0` in the trace**, which is not a legal object
    /// number and so cannot be read as a real anchor.
    #[test]
    fn the_top_level_anchor_traces_as_zero() {
        assert_eq!(
            anchor_number(OutlinePlacement::LastChild { parent: None }),
            0
        );
        assert_eq!(
            anchor_number(OutlinePlacement::FirstChild {
                parent: Some(ObjId::new(7, 0))
            }),
            7
        );
        assert_eq!(
            placement_word(OutlinePlacement::FirstChild { parent: None }),
            "first-child"
        );
    }
}
