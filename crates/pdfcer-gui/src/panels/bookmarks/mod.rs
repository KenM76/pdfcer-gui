//! # `panels::bookmarks` — the document's outline, as navigation
//!
//! Salvaged from the old shell's `panels_structure.rs`, unchanged in
//! substance. **This is the only one of the six panels that can act on the
//! document at all**: it pushes [`Action::GoToPage`], which is the one thing
//! stage S3's action enum can carry that a panel wants.
//!
//! "Bookmarks", not "Outline": the PDF specification calls the structure an
//! outline (§12.3.3) and every other reader calls the things in it
//! bookmarks. The operator-facing word is the one operators use; the spec's
//! word stays in the code and the doc comments.
//!
//! # Why the tree is read fresh each frame rather than cached
//!
//! [`pdfcer_core::outline::read_outline`] takes an object graph, not `&mut
//! self`, so it can run inside the draw closure — and the outline is a
//! property of the document that page edits can change (deleting a page can
//! leave a bookmark pointing nowhere). A cache would need invalidating on
//! every edit and undo, which is a correctness problem traded for a parse of
//! a structure that is a few hundred items at most.
//!
//! Measure before trading back. Note the contrast with
//! [`crate::panels::objects`], whose decomposition **is** cached: that one
//! walks every content stream on the page and there is no cache anywhere in
//! `pdfcer-core`. The two panels differ because the work does, not because
//! one of them was optimised and the other forgotten.
//!
//! # A bookmark with no destination is NOT an error
//!
//! Three distinct states, and collapsing them would mislead:
//!
//! | State | Row | Why |
//! |---|---|---|
//! | points at a page pdfcer resolved | full-strength label, tooltip names the page | the only one worth a click |
//! | a **heading** with no destination at all | weak label, tooltip says so | legal, common, groups its children |
//! | a destination pdfcer could not resolve | weak label, tooltip says so | the document meant something and pdfcer could not follow it |
//!
//! Only the third is a problem. Rendering the second and third alike would
//! send an operator hunting for damage in a perfectly ordinary document; not
//! showing the third at all would hide a real defect.
//!
//! ## ★★ The two unnavigable kinds stopped being DISABLED WIDGETS on
//! 2026-08-29, and the distinction is unchanged
//!
//! This table used to read *"disabled"* in the second column, and the rows were
//! `add_enabled(false, …)` buttons. That was right while a row's only job was
//! navigation — R83, never offer a control for something that cannot work — and
//! it quietly cost something the paragraph below already claimed: a disabled
//! `egui::Button` reports no click at all, so a **heading** could never be
//! selected, and could therefore never be the parent for an add, although this
//! file said it was *"the likeliest one"*.
//!
//! A row now has four jobs — navigate, select, drag, expand — and three of them
//! work perfectly on a heading. So the row is a live control and it is
//! **navigation alone** that is withheld: no [`Action::GoToPage`] is raised,
//! the label is drawn weak, and the tooltip says which of the two kinds it is.
//! R83 is satisfied more precisely than before, because the thing withheld is
//! now the thing that cannot work rather than everything that shares a widget
//! with it.
//!
//! Neither unnavigable kind is a navigation *affordance*. Both are still drawn,
//! because a heading's children hang off it and omitting the parent would show
//! them at the wrong depth, silently misrepresenting the document's structure.
//!
//! [`pdfcer_core::outline::Destination`] is `#[non_exhaustive]` with six
//! variants; only `Page { page_index, .. }` is navigable and the match below
//! says so by naming it and treating everything else as unresolved. That is
//! deliberate: a variant added to core must default to *"pdfcer could not
//! follow this"*, never to a guess.
//!
//! # ★★★ The list honours `/Count`'s SIGN, and it did not until 2026-08-29
//!
//! A **collapsed** bookmark's children are no longer drawn. `read_outline`
//! resolves the whole tree whatever the sign says — `OutlineItem::children` is
//! populated for a closed item exactly as for an open one — and this panel used
//! to recurse into all of it unconditionally.
//!
//! That was defensible while there was no way to change the sign: the panel
//! showed more than the document asked for, which is more than it was owed and
//! less than a lie. It stopped being defensible the moment
//! [`reorder`]'s disclosure triangle arrived, because a control that writes
//! `/Count`'s sign into the file and changes **nothing on screen** is a control
//! that appears not to work.
//!
//! Three sentences elsewhere in this panel became literally true with the
//! change, having been true only of the file before it:
//!
//! * [`crate::text::panels::bookmark_add_under_collapsed`] — *"the new one will
//!   not appear until you expand it"*;
//! * [`crate::text::panels::bookmarks::bookmark_move_into_collapsed`], its
//!   counterpart for the move;
//! * [`edit`]'s subtree warning, which is about a branch the operator cannot
//!   see and now genuinely cannot see.
//!
//! ★ **The count above the list is a different number from the number of rows,
//! and that is correct.** `outline.diagnostics.items` counts every item pdfcer
//! read at every level, collapsed branches included — the document's real size.
//! The rows are what is visible. They agreed before this change and are allowed
//! to differ now, because the summary is about the document and the list is
//! about the screen.
//!
//! # The truncation disclosure sits ABOVE the list
//!
//! An operator who scrolls a short list and stops has already drawn a
//! conclusion by the time a footnote would reach them. Same reasoning as the
//! Signatures caveat and the Fonts coverage note; three panels, one rule.
//!
//! # Indentation is keyed by object id, not by index
//!
//! `ui.indent` takes an id source, and two siblings at the same index in
//! different subtrees would collide in egui's id space — which shows up as
//! the wrong row responding to a hover. The item's `ObjId` (`num`,
//! `generation`) is unique across the document, so it cannot.

/// ★ Writing a bookmark — the half this panel did not have until
/// `EditSession::add_outline_item` shipped on 2026-08-19.
///
/// Its header carries the `/Count` trap the engine called *"the entire
/// difficulty of the feature"*: a bookmark added under a **collapsed** parent
/// does not change the document's total, so a surface reporting a diff reports
/// zero for a correct save — and, more to the point for an operator, the
/// bookmark is genuinely not visible until the parent is expanded.
pub mod add;
/// ★ Cut, copy and paste of a bookmark and everything filed under it — O59
/// item 3, and the one operation in this panel Acrobat cannot do between two
/// files at all.
mod clip;

/// ★ Renaming a bookmark, and removing one with everything under it - the half
/// this panel did not have until `EditSession::set_outline_title` and
/// `EditSession::delete_outline_item` shipped on 2026-08-28.
///
/// Its header carries the two decisions a reader must not have to re-derive:
/// why the delete is **undoable rather than confirmed** (one press is one
/// engine command, so `Ctrl+Z` restores the whole subtree, and the sentence an
/// operator needs is *"this takes the eleven underneath"* rather than *"are you
/// sure?"*), and - since 2026-08-29 - where reorder and re-parent went when
/// they arrived, which is [`reorder`] and not another button in that block.
pub mod edit;
/// ★★★ Moving a bookmark by **dragging** it, and the triangle that opens or
/// closes one - the half this panel did not have until
/// `EditSession::move_outline_item` and `EditSession::set_outline_open` shipped
/// on 2026-08-29 as `pdfcer-core` `Pass 161.0`.
///
/// Its header carries the three things a reader must not re-derive: why the
/// gesture is copied from [`crate::panels::pages`] rather than invented, why a
/// **tree** needs a depth at each landing where a grid needs only a gap - and
/// the three-band split that supplies it - and why expansion is a **separate
/// verb** rather than a flag on the move, which is the engine's own instruction:
/// *"whether a move should reveal a collapsed destination has two defensible
/// answers and both now exist."*
pub mod reorder;
/// The two questions this panel asks of an outline - *where is this id?* and
/// *how many bookmarks are under this one?* - in the one place they can be
/// tested.
///
/// Split out of [`add`] when [`edit`] needed both. Its header carries why both
/// walks are generic over the tree (`OutlineItem` is `#[non_exhaustive]` and
/// this crate cannot build one, so a recursion written over it directly is a
/// recursion no test here can reach) and why the subtree count reads the
/// **tree** rather than `/Count`.
pub mod tree;

/// The panel's state, between frames.
///
/// ★ **Moved here from [`add`] on 2026-08-28**, when [`edit`] arrived. It was
/// never the add row's private state - the row it holds is the row the whole
/// panel is pointed at - and leaving it in `add` would have made the rename and
/// remove controls reach through the module that writes new bookmarks to find
/// the one they act on. `crate::panels::PanelsState` names the type and not its
/// path, so the move is invisible to every caller.
///
/// ★ **The selected bookmark is an `ObjId`, not a path through the tree.**
/// `OutlineItem::id` carries it for exactly this, and its own doc says why:
/// *"identity is what a GUI needs and the tree cannot otherwise supply ...
/// selecting a bookmark ... keys off the object, not off a path through the
/// tree that any edit invalidates."*
///
/// An index into the walk would name a different bookmark after every add,
/// which is the hazard the engine hit **in its own CLI** - *"the indices shift
/// after every add ... I got this wrong myself while driving the command and
/// nested something two levels deeper than intended, and the output looked
/// entirely plausible."*
#[derive(Default)]
pub struct BookmarksUi {
    /// What has been typed into the **new bookmark's** title field.
    ///
    /// Distinct from [`Self::rename`], which is a draft over an existing
    /// bookmark's name. Two fields rather than one because they answer
    /// different questions and are live at the same time: an operator may be
    /// half-way through naming a new bookmark when they decide to rename the
    /// one they had selected, and a shared buffer would swap one into the
    /// other.
    pub(super) title: String,
    /// ★ **The row the operator last clicked**, or `None` for none.
    ///
    /// One field, three meanings, all of them true of the row that was pointed
    /// at - which is what makes the overload honest rather than a shortcut:
    ///
    /// | read by | means |
    /// |---|---|
    /// | [`add`] | the parent a new bookmark is filed under; `None` is the top level |
    /// | [`edit`] | the bookmark being renamed |
    /// | [`edit`] | the bookmark being removed, with its subtree |
    ///
    /// The one seam worth knowing: `None` is a **meaningful** answer for the
    /// add (file it at the top level) and an **absent** one for the other two
    /// (nothing is selected, so R9 says draw nothing). So pressing *Move to top
    /// level* in the add row also takes the rename and remove controls away.
    /// That is correct - nothing is selected - and it is stated in [`edit`]'s
    /// header rather than left to be discovered.
    pub(super) selected: Option<pdfcer_core::object::ObjId>,
    /// The rename draft, **paired with the bookmark it was typed for**.
    ///
    /// The pairing is the point: a half-typed name must not follow the operator
    /// to a different bookmark. A draft whose id does not match the selected
    /// row is stale, and [`Self::rename_draft_for`] re-seeds from the document
    /// instead of offering it.
    pub(super) rename: Option<(pdfcer_core::object::ObjId, String)>,
    /// ★★ **The bookmark currently being dragged**, or `None` for no drag in
    /// flight.
    ///
    /// # Why it lives here rather than in the `egui::Context`
    ///
    /// [`crate::pagedrag`] publishes the *page* drag into the context, and its
    /// header carries the reason: a page can be dropped into another open
    /// document, so the drag has to outlive the panel that started it and be
    /// readable by a panel that did not.
    ///
    /// A bookmark cannot. An outline is a document-level structure (§12.3.3)
    /// reached from the catalogue's `/Outlines`, and a bookmark's destination
    /// names a page of **this** document — carrying one into another file would
    /// author a bookmark pointing at a page that is not there. So the gesture
    /// begins and ends inside one panel, and the shortest life that can hold it
    /// is the honest one. A drag that could be read from anywhere is a drag
    /// somebody will eventually read from somewhere.
    ///
    /// # It is an `ObjId`, like everything else this panel holds
    ///
    /// Same rule, same reason as [`Self::selected`]: an id survives an edit and
    /// a position does not. A drag that stored *"the fourth row"* would be
    /// holding a number the drop it is about to perform invalidates — which is
    /// the hazard `OutlinePlacement`'s own doc comment names for this exact
    /// surface: *"A shell that reads a panel, lets the operator drag a row, and
    /// then calls with the index it read has a race with its own undo stack."*
    pub(super) drag: Option<pdfcer_core::object::ObjId>,
}

impl std::fmt::Debug for BookmarksUi {
    /// The drafts' **lengths**, not their text: a bookmark's name is the
    /// operator's own words about their drawing, and this reaches a trace file
    /// a harness keeps. `panels::properties::info` makes the same choice for
    /// `/Info`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BookmarksUi")
            .field("title_len", &self.title.len())
            .field("selected", &self.selected)
            .field(
                "rename_len",
                &self.rename.as_ref().map(|(_, text)| text.len()),
            )
            .field("drag", &self.drag)
            .finish()
    }
}

impl BookmarksUi {
    /// Record the row the operator clicked.
    ///
    /// Clears any rename draft held for a *different* bookmark on the way
    /// through, which is belt-and-braces beside [`Self::rename_draft_for`]'s
    /// staleness test: the draft is re-seeded on read anyway, and dropping it
    /// here means a stale name does not sit in memory being not-shown.
    pub fn select(&mut self, id: pdfcer_core::object::ObjId) {
        if self.rename.as_ref().is_some_and(|(held, _)| *held != id) {
            self.rename = None;
        }
        self.selected = Some(id);
    }

    /// Forget the selected row.
    ///
    /// Raised by the add row's *Move to top level* - where it means *"file the
    /// next one at the top"* - and by [`edit`] the instant a removal is raised,
    /// so the block does not spend one frame describing a bookmark that has
    /// gone. See that call site for why one frame matters.
    pub fn clear_selection(&mut self) {
        self.selected = None;
        self.rename = None;
    }

    /// The rename draft for `item`, seeded from the document when it is stale.
    ///
    /// *Stale* means **held for a different bookmark** - see [`Self::rename`].
    ///
    /// ★ **The draft does NOT follow the document while it is being typed**,
    /// deliberately, and that differs from `panels::properties::info`'s
    /// epoch-reseed. The difference is what the two fields are: a metadata box
    /// commits on focus loss and is otherwise idle, so re-seeding it costs
    /// nothing; a rename box is typed into and then committed, and an epoch
    /// bump from an unrelated edit - placing a dimension, moving a page -
    /// would wipe a half-typed name mid-keystroke.
    ///
    /// The narrow cost is that undoing a rename leaves the old name in the box
    /// until the operator selects another bookmark and comes back. The button
    /// re-appears, because the draft now differs from the document, so the
    /// state is legible rather than wrong. Same trade, same wording, as
    /// `panels::dimension_groups::identity::rename_draft_for`.
    pub(super) fn rename_draft_for(&self, item: &pdfcer_core::outline::OutlineItem) -> String {
        match &self.rename {
            Some((id, text)) if *id == item.id => text.clone(),
            _ => item.title.clone(),
        }
    }

    /// Hold what is in the rename field, against the bookmark it belongs to.
    pub(super) fn set_rename_draft(&mut self, id: pdfcer_core::object::ObjId, text: String) {
        self.rename = Some((id, text));
    }

    /// Drop the rename draft so the next frame re-seeds from the document.
    pub(super) fn clear_rename_draft(&mut self) {
        self.rename = None;
    }
}

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::panels::PanelsState;
use crate::text::panels as t;

/// Draw the Bookmarks panel.
pub fn body(ui: &mut egui::Ui, doc: &OpenDoc, state: &mut PanelsState, actions: &mut Vec<Action>) {
    let outline = pdfcer_core::outline::read_outline(&doc.session.view());

    let total = outline.diagnostics.items;
    // The current page, so a driven click has an observable to check
    // against — the only oracle available when the operator is using the
    // machine and a screenshot harness would seize their screen.
    crate::diag::trace(|| {
        format!(
            "bookmarks-panel page={} items={total}",
            doc.view.page_index + 1
        )
    });
    ui.label(t::bookmarks_count(total));
    // The truncation disclosure sits ABOVE the list, not below it — see the
    // module docs.
    if outline.diagnostics.cycles_broken > 0
        || outline.diagnostics.depth_truncations > 0
        || outline.diagnostics.item_budget_exhausted
    {
        ui.label(egui::RichText::new(t::bookmarks_truncated()).small().weak());
    }
    if outline.items.is_empty() {
        ui.label(t::bookmarks_empty());
        // ★ NOT an early return any more. A document with no bookmarks is
        // exactly the one an operator most wants to add the first one to, and
        // returning here is what made this panel read-only-looking for its
        // whole life — the sentence said "none" and offered nothing.
        add::show(ui, doc, state.bookmarks_mut(), actions);
        return;
    }
    ui.separator();

    // Collected first, applied after — the actions-not-mutations discipline
    // at its smallest: the click is recorded while the tree is being walked
    // and turned into an `Action` once the walk is over.
    let mut harvest = Harvest::default();
    // ★★ The authoring row is drawn BEFORE the list, and that ordering is
    // the fix for a feature that shipped unreachable.
    //
    // A driven run on a 122-bookmark drawing found the panel body occupying
    // y=133..770 and this row laid out at y=899..923 — **below the bottom of
    // the panel**, with no way to reach it. The row drew. It published its
    // region. Every unit test passed. And `add_outline_item`, wired that
    // morning, could not be used on any document with a real outline, which is
    // every document somebody would want to add a bookmark to.
    //
    // The first attempt capped the list with a reserve, which moved the row
    // from y=899 to y=769 in a panel ending at 770 — still overflowing, by less.
    // That is the shape of a fix that is a **magic number**: it works at the
    // pane height it was tuned against and fails quietly at every other one.
    //
    // Putting the row first removes the arithmetic entirely. Nothing follows
    // the scroll area, so nothing can be pushed past the end of the panel, at
    // any pane height, with any size of outline. The rule generalises and is
    // worth stating: **a control that must always be reachable cannot be placed
    // after an unbounded `ScrollArea`.** Reserve-and-hope is not a second
    // option; it is the same defect with a tuning parameter.
    //
    // It also reads better, for the reason the Manage-groups window's Add
    // button was moved on the same pass: this row acts on the LIST — it files
    // the new bookmark under whichever row was last clicked — and a control's
    // position is a claim about what it acts on. Above the list it is making
    // that claim correctly, and the operator sees the destination before they
    // scroll rather than after.
    add::show(ui, doc, state.bookmarks_mut(), actions);
    // ★ The rename-and-remove block, and it is drawn ONLY when a row has been
    // clicked. That is R9 rather than tidiness: with nothing selected there is
    // no bookmark for either verb to name, so the controls would be offering a
    // capability that cannot act. They are absent, not greyed — greying is for
    // something *temporarily* unavailable that can explain itself, and "click a
    // row first" is already what the add row's parent hint says two lines up.
    //
    // Resolved here rather than inside `edit::show` so the whole block can be
    // skipped in one place, and so that module never has to consider an id that
    // no longer names anything — the ordinary state one frame after an undo of
    // a delete, and the state `add::show` above has already cleared.
    // ★ The drag is the one gesture in this panel with no widget to look at, so
    // it is the one that has to be written down. See
    // `crate::text::panels::bookmarks::bookmark_drag_hint`: R83 forbids
    // offering a control that cannot work, and its quieter twin is that a
    // gesture nobody is told about is a capability the program does not have.
    ui.weak(crate::text::panels::bookmarks::bookmark_drag_hint());
    ui.separator();

    // Read before the scroll area, because the walk needs it and the walk
    // cannot borrow `state` — `edit::show` above already holds it mutably for
    // part of this function, and the drag is one `Copy` id rather than a
    // borrow.
    let dragging = state.bookmarks_mut().drag;
    // Where the drag would land. Resolved INSIDE the scroll area, for
    // `reorder::VisibleRow`'s stated reason: a row's bands and the end of its
    // subtree have no position until the tree has been laid out.
    let mut target: Option<reorder::DropTarget> = None;
    egui::ScrollArea::vertical()
        .id_salt("bookmark-rows")
        .show(ui, |ui| {
            // The full-width strip a row occupies, captured once: the caret
            // spans the list rather than stopping under the longest title, and
            // a band test over the label alone would miss the pointer whenever
            // it was to the right of a short name.
            // ★★★ THE PER-SELECTION CONTROLS LIVE IN HERE, and they used to live
            // above this scroll area. That shipped two controls an operator
            // could not click.
            //
            // The dock gives a panel body a FIXED rectangle and no scrolling of
            // its own — `egui-shell`'s dock says so in as many words, and the
            // corollary is that the body is expected to create one. This body
            // created one around its LIST and left everything above it laid out
            // in whatever space the panel happened to have.
            //
            // Measured 2026-08-29 with the panel body at `y = 159.3 .. 447.7`:
            //
            //     ui-rect name=bookmarks.delete rect=[[0.0 500.3] - [55.3 524.3]]
            //     ui-rect name=bookmark-copy    rect=[[0.0 528.3] - [37.2 552.3]]
            //
            // ⇒ **Remove was 53 points below the bottom of its own panel and
            // Copy was 81.** Both were drawn, both published a rect, and neither
            // could be clicked. That is the shape `D:/dev/rag/egui/` records as
            // *panels that shipped unreachable in real builds with every gate
            // green* — and it survived because it is invisible to any test that
            // does not select a bookmark first. Nothing did, until the clipboard
            // work needed to.
            //
            // ★ Inside the scroll area rather than given a scroll area of their
            // own: two sibling scrollers in one narrow panel is two scrollbars
            // and two places the operator's wheel might go. One region that
            // scrolls is what every other panel here does.
            //
            // ★★ ABOVE the rows, not below, because they are about the row that
            // is already selected — putting them under a list of forty
            // bookmarks would mean scrolling past the list to act on something
            // at the top of it.
            let selected = state
                .bookmarks_mut()
                .selected
                .and_then(|id| tree::find(&outline.items, id));
            if let Some(item) = selected {
                // Cloned because `state` is borrowed mutably by `edit::show` and the
                // item is borrowed out of `outline`, which `state` does not own. One
                // `OutlineItem` per frame in which a bookmark is selected, against
                // restructuring the whole panel to read the outline twice.
                let item = item.clone();
                edit::show(ui, &item, state.bookmarks_mut(), actions);
                // ★ Copy and Cut sit with the other verbs that act on the SELECTED
                // bookmark, under the same heading, because that is the question the
                // operator is answering when they are looking at this block.
                clip::copy_row(ui, doc, &item, actions);
            }
            // ★★ PASTE IS OUTSIDE THE `if`, and that is the whole difference between
            // it and the two above.
            //
            // Copy and Cut act on a selected bookmark, so with none selected there is
            // nothing for them to act on and R9 says draw nothing. A paste has no
            // operand on the tree at all -- it reads the CLIPBOARD -- and pasting at
            // the top level of a document with nothing selected is not merely legal, it
            // is the ordinary case for putting a copied chapter into an empty outline.
            //
            // ⇒ Gating it on the selection would have made the one thing this feature
            // exists for -- carrying a chapter's bookmarks into another drawing --
            // reachable only by first selecting a bookmark in the document that has
            // none.
            let selected_for_paste = state
                .bookmarks_mut()
                .selected
                .and_then(|id| tree::find(&outline.items, id))
                .cloned();
            clip::paste_row(ui, doc, selected_for_paste.as_ref(), actions);

            let strip = (ui.max_rect().left(), ui.max_rect().right());
            rows(ui, &outline.items, strip, dragging, &mut harvest);
            target = reorder::resolve(ui, &harvest.rows, &outline.items, dragging);
            // Painted AFTER the rows and INSIDE the scroll area, which is what
            // puts it over them rather than under — egui paints in call order —
            // and what keeps it in the coordinate space it was measured in.
            reorder::paint_caret(ui, target.as_ref());
        });

    let ui_state = state.bookmarks_mut();
    // ★★★ **A DRAG DOES NOT SELECT THE ROW IT BEGAN ON, AND THAT IS A MEASURED
    // DEFECT RATHER THAN A PREFERENCE.**
    //
    // It did, for one build. `panels::pages`' tile selects on `drag_started`,
    // on a rule worth keeping — *a gesture's verbs must apply to the tile the
    // operator pointed at* — and the same line here also put the row's name in
    // the *Selected bookmark* block, so an operator who let go over nothing
    // could still see what they had been carrying. Both arguments are good.
    //
    // The block is **drawn above the list**, and it is only drawn when
    // something is selected. So selecting on press GROWS THE PANEL ABOVE THE
    // ROWS, mid-gesture. Driven, 2026-08-29,
    // `a_bookmark_can_be_dragged_and_a_branch_collapsed`, from the trace:
    //
    // ```text
    // before the press   bookmark-row id=64 row=[[0 542] - [274 566]]
    // after the press    bookmark-row id=64 row=[[0 729] - [260 753]]   +187
    // ```
    //
    // **One hundred and eighty-seven points**, and the strip narrowed by
    // fourteen as a scroll bar appeared with it. The row the operator was
    // aiming at slid a third of a panel's height out from under the pointer at
    // the instant they committed to the drag, and the drop then landed on empty
    // space above the list — `bookmark-drag-released … landing=none`, a gesture
    // that did nothing with no explanation, which is this project's founding
    // defect shape.
    //
    // ⇒ That is R128's feedback loop, and this is its **third** instance in
    // this codebase: `bottom_panel_height_...` in the egui RAG was the first,
    // `panels::pages`' drag caption the second, measured at 49 points and
    // recorded in that module's header with the same trace shape. The rule it
    // yields is more general than any of the three: **a surface may not change
    // size in response to a gesture that is aimed at it.**
    //
    // ★ Nothing is lost. `BookmarksUi::drag` carries the operand, captured at
    // the press, so the move acts on the row the operator pointed at exactly as
    // the pages rule requires. What is given up is the *Selected bookmark*
    // block naming the row in flight — which was a convenience, and which cost
    // the gesture it was decorating.
    //
    // ★ And selection is unchanged as a **click**: egui reports no `clicked()`
    // for a press that travelled, so a press-and-release without movement still
    // selects, and a drag does not. Those are two gestures with two meanings,
    // which is what every other outline panel does.
    if let Some(id) = harvest.started {
        ui_state.drag = Some(id);
    }
    // ★ Runs unconditionally and BEFORE the click is applied. A drag that has
    // started has to be able to end — see `reorder::settle` on why the release
    // is read from raw pointer input — and egui reports no `clicked()` for a
    // press that travelled, so the two cannot both fire for one gesture.
    reorder::settle(ui, ui_state, target.as_ref(), actions);
    if let Some(id) = harvest.picked {
        ui_state.select(id);
    }
    if let Some((item, open)) = harvest.disclosure {
        actions.push(Action::Bookmark(
            crate::app::actions::bookmarks::BookmarkAction::SetOpen { item, open },
        ));
    }
    if let Some((page, view)) = harvest.go {
        // ★★ One place turns a destination into moves, so the bookmarks panel
        // and anything else that navigates to one cannot disagree about what
        // `/XYZ` means. See `app::actions::destination`.
        crate::app::actions::destination::actions_for(page, &view, actions);
    }
}

/// Everything one walk of the outline has to carry back out of it.
///
/// # ★ Why a struct rather than five `&mut` parameters
///
/// [`rows`] is recursive, so every output it collects is threaded through every
/// level. Five out-parameters would be five places to transpose two of the same
/// type — and two of these *are* the same type
/// (`Option<pdfcer_core::object::ObjId>`: the row that was clicked and the row a
/// drag began on), which is exactly the pair a reader cannot check by eye at a
/// call site.
///
/// [`crate::panels::pages::grid_rows`] takes them loose and carries a
/// `clippy::too_many_arguments` waiver saying *"bundling them into a struct
/// would name a type whose only purpose is to be destructured immediately"*.
/// That argument holds there and does not hold here, and the difference is the
/// recursion: a bundle passed down four levels is written once, and four loose
/// parameters are written at every level.
#[derive(Default)]
struct Harvest {
    /// The page a click asked to go to, 0-based.
    /// The page a click asked for, **with the view that came with it**.
    ///
    /// ★ A pair rather than a page, since 2026-09-01. Carrying only the page
    /// is what made every bookmark on a drawing sheet arrive in the same place.
    go: Option<(usize, pdfcer_core::outline::DestView)>,
    /// ★ The row that was clicked, recorded as well as navigated to. A bookmark
    /// click means "take me there" first and always; making it ALSO mean "and
    /// this is the parent for the next one" is free, because both are true of
    /// the row the operator pointed at, and it saves a second selection gesture
    /// that would have to be taught.
    picked: Option<pdfcer_core::object::ObjId>,
    /// The row a drag began on this frame.
    started: Option<pdfcer_core::object::ObjId>,
    /// A disclosure triangle that was pressed, and the state it asked for.
    disclosure: Option<(pdfcer_core::object::ObjId, bool)>,
    /// Every row that was actually drawn, in draw order. The input to
    /// [`reorder::resolve`]; see [`reorder::VisibleRow`] for why the drop is
    /// resolved from this list rather than as each row is laid out.
    rows: Vec<reorder::VisibleRow>,
}

/// How wide the disclosure triangle's slot is, in points.
///
/// ★ Reserved on a **leaf** as well, with `add_space`, so every title at one
/// level starts at one x. A tree whose rows step in and out by the width of a
/// triangle depending on whether they have children reads as a rendering fault,
/// and it is the first thing an eye notices in a list of names.
const DISCLOSURE_WIDTH_PTS: f32 = 14.0;

/// Draw one level of the outline, recursing into the children of the rows that
/// are **open**.
///
/// Indentation carries the structure. See the module docs on why the indent
/// is keyed by the item's object id rather than by its index.
///
/// # ★★★ It recurses only when the row is open, and it did not used to
///
/// This walk was unconditional for the whole life of the panel: every child of
/// every bookmark was drawn, whatever `/Count`'s sign said. That was defensible
/// while there was no way to change the sign — the panel showed the operator
/// everything, which is more than the document asked for and less than a lie.
///
/// It stopped being defensible the moment the disclosure triangle arrived. A
/// control that writes `/Count`'s sign into the file and changes **nothing on
/// screen** is a control that appears not to work, and the operator's next act
/// is to press it again. So the sign is now honoured here, and three other
/// sentences in this panel became literally true with it:
/// [`crate::text::panels::bookmark_add_under_collapsed`]'s *"the new one will
/// not appear until you expand it"*, its move counterpart, and
/// [`edit`]'s subtree warning, which was always about a branch the operator
/// could not see and can now genuinely not see.
///
/// ★ **The count above the list is unaffected and is a different number.**
/// `outline.diagnostics.items` is every item pdfcer read, at every level,
/// collapsed branches included — the document's real size — and the number of
/// rows drawn here is what is visible. They were the same before this change
/// and are now allowed to differ, which is correct: the panel's summary is
/// about the document and its list is about the screen.
///
/// # ★★ Every row is an enabled control now, and that is a widening
///
/// A row whose destination pdfcer cannot resolve used to be an
/// `add_enabled(false, …)` button, and a disabled `egui::Button` reports no
/// click at all. That was right when a row's only job was navigation — R83,
/// never offer a control for something that cannot work — and it quietly cost
/// something this panel's own comments claimed it had: a **heading** with no
/// destination could never be selected, so it could never be the parent for an
/// add, although this file said *"indeed it is the likeliest one, since a
/// heading is what an operator files things under."*
///
/// A row now has four jobs — navigate, select, drag, expand — and three of them
/// work perfectly on a heading. So the row is enabled and it is **navigation
/// alone** that is withheld: the click raises no [`Action::GoToPage`], the
/// label is drawn weak, and the tooltip says which of the two unclickable kinds
/// it is. The three-state distinction the module header sets out is unchanged;
/// what changed is that it is now carried by the label's colour and its words
/// rather than by a dead widget.
///
/// ★ `enabled=` in the row's trace line still means **navigable**, which is
/// what every reader of it assumes and what `tools/ui-verify`'s
/// `bookmark_edit` check skips on.
fn rows(
    ui: &mut egui::Ui,
    items: &[pdfcer_core::outline::OutlineItem],
    strip: (f32, f32),
    dragging: Option<pdfcer_core::object::ObjId>,
    harvest: &mut Harvest,
) {
    use pdfcer_core::outline::Destination;
    for it in items {
        // The page a click would reach, if any. Only a resolved page
        // destination is navigable — a named destination pdfcer could not
        // look up, or a remote file, is shown and not offered.
        // ★★★ **The view comes with the page** — 2026-09-01.
        //
        // This read `Some(Destination::Page { page_index, .. })`, and that `..`
        // is where the operator's zoom went: *"it just jumps us to the correct
        // page, but doesn't send us to the spot on the page the bookmark
        // actually points to."*
        //
        // On a drawing package every bookmark names a DETAIL — `/XYZ` or
        // `/FitR` on a shared sheet — so discarding the view reduced the whole
        // outline to a page list, and several bookmarks pointing at different
        // details all arrived in the same place.
        let target = match &it.destination {
            Some(Destination::Page {
                page_index, view, ..
            }) => Some((*page_index, view.clone())),
            _ => None,
        };
        let (enabled, tip) = match (&it.destination, &target) {
            (_, Some((p, _))) => (true, t::bookmark_row_tooltip(p + 1)),
            (None, _) => (false, t::bookmark_row_heading_tooltip().to_owned()),
            (Some(_), None) => (false, t::bookmark_row_unresolved_tooltip().to_owned()),
        };

        let label = if it.title.trim().is_empty() {
            // An untitled bookmark is legal and unclickable-looking. Its own
            // row still has to exist, or its children lose their parent and
            // appear at the wrong depth.
            t::bookmark_untitled().to_owned()
        } else {
            it.title.clone()
        };

        // ★ The label carries the three-state distinction now that the widget
        // no longer can: a row that cannot be jumped to is drawn weak, which is
        // the same signal a disabled button gave and the same one the Fonts
        // panel uses for a face it cannot act on. The tooltip says which kind.
        let label = if enabled {
            egui::RichText::new(label)
        } else {
            egui::RichText::new(label).weak()
        };

        let row = ui.horizontal(|ui| {
            disclosure(ui, it, harvest);
            // `click_and_drag`, not `click`. The row was click-only for the
            // whole life of this panel, which is why moving a bookmark was
            // impossible rather than merely awkward — the gesture every
            // operator tries first was not sensed at all. Same sentence, same
            // fix, as `panels::pages`' tile.
            ui.add(
                egui::Button::new(label)
                    .frame(false)
                    .sense(egui::Sense::click_and_drag()),
            )
            .on_hover_text(tip)
        });
        let resp = row.inner;
        // The full-width strip, for the band test and the caret. See
        // `reorder::VisibleRow::rect`: a test over the label alone would lose
        // the pointer the moment it moved right of a short title, which on an
        // outline of chapter numbers is most of the panel.
        let row_rect = egui::Rect::from_min_max(
            egui::pos2(strip.0, row.response.rect.top()),
            egui::pos2(strip.1.max(strip.0), row.response.rect.bottom()),
        );
        harvest.rows.push(reorder::VisibleRow {
            id: it.id,
            level: it.level,
            rect: row_rect,
            // Measured, not computed from `level`: an indent the theme changes
            // must move the caret and the rows together or the mark lies about
            // the depth.
            indent_left: row.response.rect.left(),
            open: it.open,
            has_children: !it.children.is_empty(),
        });
        crate::diag::trace(|| {
            // ★★★ TWO rectangles, and a harness needs both. `rect=` is the
            // **label**, which is what a check presses to select or to lift a
            // row — its meaning is unchanged from the day this line was
            // written, and `bookmark_edit` aims with it. `row=` is the
            // **full-width strip**, which is what a drop is tested against, and
            // its centre is deliberately somewhere no widget is: a check aiming
            // at a landing band must land on the row, not on the title.
            //
            // Keeping `rect=` as the label rather than widening it is the whole
            // of why the existing check still works. A key that changes meaning
            // in place is the shape of change that breaks a harness silently —
            // every line still parses, every field is still there, and the
            // clicks land somewhere else.
            //
            // ★ `id=` and `open=` were added with the drag. A check that aims
            // at a row needs to name WHICH bookmark it hit — `title=` is
            // ambiguous the moment a document has two chapters called
            // "Details" — and `open=` is the only evidence a trace can give
            // that the disclosure triangle did anything, because what it
            // changes is which OTHER lines exist.
            format!(
                "bookmark-row id={} level={} title={:?} page={:?} enabled={enabled} \
                 open={} children={} rect={:?} row={row_rect:?}",
                it.id.num,
                it.level,
                it.title,
                target.as_ref().map(|(p, _)| p + 1),
                u8::from(it.open),
                it.children.len(),
                resp.rect,
            )
        });
        if resp.clicked() {
            // ★★★ WHICH ROW WAS ACTUALLY PRESSED, and whether it navigates.
            //
            // Added 2026-09-02 to close a gap named in `CONTINUE.md`: the driven
            // check `a_bookmark_lands_on_the_detail_it_names` fails
            // intermittently inside a batch with `zoom 0.382 → 0.382`, and
            // **cannot tell its two causes apart** — the destination was not
            // applied, or the click landed on a different row (or on none). Both
            // leave the zoom where it was, and the check's own message has to
            // hedge between them.
            //
            // A trace of the press settles it in one line. `bookmark-row`
            // already reports where every row IS; this reports which one was
            // hit, which is the other half and the one no rectangle can supply.
            //
            // ★ `navigates=` beside the title, because a heading with no
            // destination is a perfectly good row to click and changes nothing
            // — so "the right row was pressed and the zoom did not move" is only
            // a defect when the row had somewhere to go.
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "bookmark-pick id={} title={:?} navigates={} page={:?}",
                    it.id.num,
                    it.title,
                    u8::from(target.is_some()),
                    target.as_ref().map(|(p, _)| p + 1),
                )
            });
            // The id is recorded whether or not the row navigates. A heading
            // with no destination is unclickable-looking and is still a
            // perfectly good PARENT — indeed it is the likeliest one, since a
            // heading is what an operator files things under. That sentence was
            // aspirational until the row stopped being a disabled widget; see
            // this function's header.
            harvest.picked = Some(it.id);
            if let Some(p) = target.clone() {
                harvest.go = Some(p);
            }
        }
        // `drag_started_by(Primary)`, not `drag_started()`. egui's plain
        // predicate is true for the middle button as well, and a right-press
        // that wandered a few pixels before releasing would start a move the
        // operator meant as something else. `panels::pages` records the same.
        //
        // ★ A second drag cannot start while one is in flight. Without the
        // guard, dragging over the list would arm a new drag on every row the
        // pointer crossed — egui reports `drag_started` from whichever widget
        // the press is attributed to — and the bookmark that finally moved
        // would be whichever row the pointer happened to be over.
        if dragging.is_none() && resp.drag_started_by(egui::PointerButton::Primary) {
            harvest.started = Some(it.id);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "bookmark-drag-started item={} level={}",
                    it.id.num, it.level
                )
            });
        }

        // ★★★ Only when the row is OPEN. See this function's header: a
        // triangle that wrote `/Count`'s sign and left the list unchanged would
        // be a control that appears not to work.
        if it.open && !it.children.is_empty() {
            ui.indent(("bookmark", it.id.num, it.id.generation), |ui| {
                rows(ui, &it.children, strip, dragging, harvest);
            });
        }
    }
}

/// Draw the disclosure triangle, or reserve its width on a leaf.
///
/// # ★★ A leaf gets no triangle, and that is R83 rather than tidiness
///
/// §12.3.3 Table 153 makes `/Count` *"required if the item has any
/// descendants"*, so an item without them carries none and has no
/// open-or-closed state to set. `EditSession::set_outline_open` answers
/// `Ok(false)` for one rather than refusing — *"asking a leaf to expand is what
/// a 'collapse all' sweep does to every row it walks, and refusing would make
/// the sweep's caller filter first for no gain"* — so a triangle on a leaf
/// would be a control that reaches the engine and correctly does nothing.
/// Never offer a control for something that cannot work.
///
/// The width is reserved anyway. See [`DISCLOSURE_WIDTH_PTS`].
///
/// # ★ The hover text says the state is saved into the document
///
/// The one genuinely surprising fact about this control, and the reason it is
/// disclosed **before** the press rather than after: every other tree an
/// operator has used treats expand and collapse as a window setting, and here
/// it is a byte in the file. See
/// [`crate::text::panels::bookmarks::bookmark_expand_tooltip`].
fn disclosure(ui: &mut egui::Ui, item: &pdfcer_core::outline::OutlineItem, harvest: &mut Harvest) {
    use crate::text::panels::bookmarks as bt;

    if item.children.is_empty() {
        ui.add_space(DISCLOSURE_WIDTH_PTS);
        return;
    }
    let (glyph, tip) = if item.open {
        (
            bt::bookmark_expanded_glyph(),
            bt::bookmark_collapse_tooltip(),
        )
    } else {
        (
            bt::bookmark_collapsed_glyph(),
            bt::bookmark_expand_tooltip(),
        )
    };
    let response = ui
        .add(
            egui::Button::new(glyph)
                .frame(false)
                .min_size(egui::vec2(DISCLOSURE_WIDTH_PTS, 0.0)),
        )
        .on_hover_text(tip);
    // `ui_rect_visible` rather than `ui_rect`: this is inside a `ScrollArea`,
    // and a triangle scrolled out of view must not keep publishing a rectangle
    // a driven check would then click on. `diag.rs`'s header records the
    // false-failure that rule exists for, and `bookmark_edit` paid for it again
    // on 2026-08-29 by aiming three thousand points below the panel.
    crate::diag::ui_rect_visible(
        &format!("{}{}", reorder::REGION_DISCLOSE_PREFIX, item.id.num),
        response.rect,
        ui.clip_rect(),
    );
    if response.clicked() {
        harvest.disclosure = Some((item.id, !item.open));
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // ★ `open=` is the state being ASKED FOR, not the one the row is
            // in, so a check reads the request rather than having to invert it.
            format!(
                "bookmark-disclosure item={} open={} children={}",
                item.id.num,
                u8::from(!item.open),
                item.children.len(),
            )
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panels::objects::test_support::engine_fixture;
    use pdfcer_core::outline::Destination;

    /// **A resolved bookmark's page index is 0-based and already resolved by
    /// core; the tooltip prints it 1-based.**
    ///
    /// The off-by-one that would otherwise be invisible: `page_index` is
    /// *"ALREADY 0-based into `pages`"* per `pdfcer-core`'s consumer map, and
    /// [`Action::GoToPage`] takes the same 0-based index — so the raw value
    /// travels, and the `+ 1` happens only where a human reads it.
    ///
    /// Getting that backwards produces a panel that navigates one page past
    /// every bookmark, which looks like a document defect.
    #[test]
    fn a_resolved_destination_navigates_zero_based_and_prints_one_based() {
        let path = engine_fixture("outline/basic-tree.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let outline = pdfcer_core::outline::read_outline(&doc);

        let resolved: Vec<usize> = outline
            .flatten()
            .into_iter()
            .filter_map(|it| match &it.destination {
                Some(Destination::Page { page_index, .. }) => Some(*page_index),
                _ => None,
            })
            .collect();
        assert!(
            !resolved.is_empty(),
            "the fixture must have at least one resolvable destination, or this \
             test proves nothing"
        );
        for page_index in resolved {
            // What the panel would push …
            let action = Action::GoToPage(page_index);
            assert_eq!(action, Action::GoToPage(page_index));
            // … and what it would print, which is one higher.
            let tip = t::bookmark_row_tooltip(page_index + 1);
            assert!(
                tip.contains(&(page_index + 1).to_string()),
                "the tooltip must name the human page number: {tip}"
            );
        }
    }

    /// **Every non-page destination is treated as unresolved, including ones
    /// this build has never seen.**
    ///
    /// `Destination` is `#[non_exhaustive]`, so core can add a variant
    /// without this crate changing. The match must therefore *fail closed*:
    /// anything that is not a resolved page is a row pdfcer declines to
    /// offer, never a row it guesses at.
    ///
    /// Asserted against a real fixture whose destinations pdfcer genuinely
    /// cannot resolve, using the same expression the panel uses, so the two
    /// cannot come apart. Constructing `Destination` values by hand would
    /// prove only that `matches!` works.
    #[test]
    fn any_destination_that_is_not_a_resolved_page_is_not_navigable() {
        let navigable = |d: &Option<Destination>| matches!(d, Some(Destination::Page { .. }));
        // A heading has no destination at all, and is not navigable.
        assert!(!navigable(&None));

        let path = engine_fixture("outline/broken-dests.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let outline = pdfcer_core::outline::read_outline(&doc);
        let items = outline.flatten();
        assert!(!items.is_empty(), "the fixture must have bookmarks");

        let unresolvable = items
            .iter()
            .filter(|it| it.destination.is_some() && !navigable(&it.destination))
            .count();
        assert!(
            unresolvable > 0,
            "this fixture exists to carry destinations pdfcer cannot map to a \
             page; if none survive, the test proves nothing about failing closed"
        );
    }

    /// The three row states carry three different tooltips, and the two
    /// unclickable ones say which kind they are.
    ///
    /// A heading and an unresolved destination are both disabled rows. If
    /// they read the same, an operator cannot tell a perfectly ordinary
    /// document from one whose outline is damaged.
    #[test]
    fn the_two_disabled_row_kinds_explain_themselves_differently() {
        let heading = t::bookmark_row_heading_tooltip();
        let unresolved = t::bookmark_row_unresolved_tooltip();
        assert_ne!(heading, unresolved);
        assert!(heading.contains("heading"), "{heading}");
        assert!(unresolved.contains("could not resolve"), "{unresolved}");
    }

    /// An untitled bookmark still gets a row.
    ///
    /// Its children hang off it; omitting the parent would show them at the
    /// wrong depth and silently misrepresent the document's structure.
    #[test]
    fn an_untitled_bookmark_has_a_label_rather_than_being_skipped() {
        assert!(!t::bookmark_untitled().trim().is_empty());
        // Whitespace-only titles take the placeholder too — a title of three
        // spaces is an invisible row, which is the same defect as no row.
        for title in ["", " ", "\t\n"] {
            assert!(
                title.trim().is_empty(),
                "the panel's emptiness test is `trim().is_empty()`; this pins the \
                 inputs it must catch"
            );
        }
    }
}
