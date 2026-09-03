//! # `canvas::selection` — the selection STATE, and the invariant it exists to hold
//!
//! ## The two halves of this module, and the seam between them
//!
//! [`identity`] holds what a selection **is**: [`Selection`],
//! [`SelectionLevel`], [`ClickHit`] and [`EscapeOutcome`] — four `Copy` types
//! which between them cannot name a place on the screen. That file carries the
//! *"selection is an identity, not a position"* argument in full, because it is
//! an argument about the shape of a **type** and is answered by reading four
//! field declarations.
//!
//! This file holds what **accumulates** them: [`SelectionState`], the ladder it
//! walks, the `(page, epoch)`-keyed re-resolve, and every rule about what a
//! click, a marquee or an Escape means. Those are answered by reading
//! behaviour, which is why they are worth their own file and their own tests.
//!
//! All four identity types are re-exported here, so
//! `crate::canvas::selection::SelectionLevel` remains the path every caller
//! uses and the seam is invisible from outside the module.
//!
//! ## ★ The invariant, stated first because everything here is shaped by it
//!
//! `GUI_ROADMAP.md` Phase 1, from the operator's own words on 2026-08-13:
//!
//! > *"if I select a node or something for a tool, I should be able to pan
//! > and zoom out without losing my first selection."*
//! >
//! > **Navigation is not an edit. Panning, zooming, changing fit mode,
//! > rotating the view, switching page-display mode and changing ribbon tab
//! > must never alter the selection.**
//!
//! The roadmap names **three** ways the natural implementation loses it, each
//! of which looks reasonable in isolation. This module closes all three, and
//! each closure is a structural property rather than a promise:
//!
//! | # | The way it is lost | What closes it here |
//! |---|---|---|
//! | 1 | **Selection stored in screen coordinates.** Zoom changes the mapping, so the stored point stops naming the thing it named. | [`Selection`] holds **no coordinate of any kind**. It is `page + object + subpath + node`, four integers, none of which a zoom can touch. There is no constructor that takes a `Pos2`. This is the one closure that is a property of a *type* rather than of a method, which is why it lives in [`identity`] — see that file's header for the argument in full. |
//! | 2 | **Selection cleared by a click that was really a drag.** A gesture begins with a press; if press-on-empty clears, every drag that starts on blank paper destroys the selection. | Nothing in this module is called on a press. The clear is driven by [`SelectionState::click`], which [`crate::canvas::gesture`] raises only for a **completed click with no drag**. |
//! | 3 | **Selection invalidated by re-decomposition.** The provider rebuilds on page change and on edit; a rebuild triggered by zoom, or by a page change that is not a page change in the operator's sense, must not drop it. | [`SelectionState::resolve`] **re-resolves against the new decomposition** instead of discarding, and — the part that is easy to get wrong — it only validates entries **on the page the provider serves**. An entry for another page is left completely alone. |
//!
//! Row 3's second half is the one that makes the acceptance criterion pass:
//! *"select a node, zoom out three rungs, pan across the sheet, switch to
//! Continuous, come back, switch ribbon tab — the node is still selected and
//! still the entered level."* Going to another page builds a provider for
//! that page, and a `resolve` that pruned everything it could not find would
//! wipe the selection on the way past. Coming back would find nothing.
//!
//! ## Why the level is state and not derived
//!
//! [`SelectionLevel`] could be inferred from whether `subpath`/`node` are
//! `Some`. It is stored instead, because *"inside this object, nothing picked
//! yet"* is a real state — reached by entering an object at a point where no
//! subpath was close enough — and an inferred level would collapse it into
//! "not inside anything at all". The operator would then find Escape taking
//! two presses on one path and one on another, for no reason they could see.
//!
//! ## What this module deliberately does NOT do
//!
//! It never draws, never touches egui, never reads a pointer, and never
//! reaches a document. It is a state machine over four integers and a
//! provider trait, which is precisely why every invariant above can be
//! asserted in a unit test rather than hoped for in a running window.

/// What a selection **is** — the four `Copy` types the state below accumulates,
/// none of which can hold a coordinate. The pure half; see its header for why
/// "identity, not position" is a claim about a type rather than about a method.
// Clicking the things pdfcer itself put on the page — stamps, notes, shapes and
// ce dimensions. A sibling of `identity`, not a variant of it: an annotation is
// addressed by a STABLE `ObjId` where page content is addressed by a
// paint-order index, and the four ways the two differ are tabulated in its
// header.
pub mod annot;
pub mod identity;

pub use annot::{AnnotKind, AnnotSelection, AnnotTarget};
pub use identity::{ClickHit, EscapeOutcome, Selection, SelectionLevel};

use std::collections::BTreeSet;

use egui::Rect;

use crate::canvas::target::{CanvasTargetProvider, TargetId};

/// The whole of the canvas's selection state.
///
/// # ★ Where this lives, and why that is the whole of its document scoping
///
/// It is a field of `crate::app::state::OpenDoc` — the open document itself.
/// That is not filing: it is the mechanism, and it replaced one.
///
/// A selection is document-scoped state, so closing a document must forget it.
/// Until this stage the value lived in `egui::Memory`, which outlives
/// documents, so the canvas had to *detect* the change: a `DocumentToken`
/// built from the `Arc<EditSession>`'s allocation address mixed with the page
/// count, compared on every frame by a `sync_document` method that reset
/// everything when it moved. Both are now **deleted**, along with the
/// residual hazard they carried — an address is not an identity, and a reused
/// allocation with a matching page count would have carried a stale selection
/// into a new file, while holding an `Arc` or a `Weak` to make it a real
/// identity would have disabled editing outright (`Arc::get_mut` fails while
/// any other strong **or weak** reference exists).
///
/// What replaced them is `OpenDoc::new`'s own doc comment: *"opening a
/// document constructs a whole new `OpenDoc`, so a cached texture or a page
/// index can never refer to a page from a previous file."* A selection held
/// inside that structure inherits the guarantee by construction, on every
/// frame, at no cost, with nothing to compare. `panels::DocKey` and the
/// decomposition cache went the same way in the same stage, for the same
/// reason.
///
/// **A page change is still not a document change**, and never was — that is
/// invariant 3, and it is [`Self::resolve`]'s business, not this note's.
///
/// # Why it caches canvas-space outlines
///
/// Drawing the selection every frame needs each entry's bounds, and bounds
/// come from a decomposition. `decompose_page` resolves every `/Contents`
/// stream, inflates it, concatenates, tokenizes and walks the whole token
/// stream resolving fonts as it goes, with **no cache anywhere in
/// `pdfcer-core`** — so asking for it per frame is not an option.
///
/// Canvas-space bounds are the right thing to cache because they are
/// **zoom-independent**: canvas space is the page's device space at scale
/// 1.0, so a zoom or a pan changes where the outline is *drawn* and not what
/// it *is*. The cache is therefore keyed on `(page, edit epoch)` and survives
/// every navigation — which is the invariant again, from the drawing side.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectionState {
    /// The selected entries, in `(page, object, subpath, node)` order.
    entries: Vec<Selection>,
    /// The rung the operator has entered.
    level: SelectionLevel,
    /// Canvas-space outline rects for the entries on the resolved page, in
    /// the order they should be painted.
    outlines: Vec<(Selection, Rect)>,
    /// The `(page, edit epoch)` [`Self::outlines`] describes, or `None`
    /// before the first resolve.
    resolved_for: Option<(usize, u64)>,
    /// The selected **annotation**, if one is selected instead of content.
    ///
    /// # ★ Why it is a field here rather than a second selection elsewhere
    ///
    /// Because the two are **mutually exclusive**, and that has to be enforced
    /// somewhere rather than remembered everywhere. Putting it on
    /// `OpenDoc` beside this state would make "what is selected?" a question
    /// with two answers that could both be yes — which is exactly the *"second
    /// selection"* `panels::ObjectTreeUi::focus`' docs refuse, arriving through
    /// a field instead of through a type.
    ///
    /// Here, [`Self::select_annot`] and the content paths are the only writers
    /// and each clears the other. One canvas, one selection.
    ///
    /// # Why it needs no `resolved_for` twin
    ///
    /// [`Self::outlines`] is cached against `(page, epoch)` because content
    /// bounds cost a `decompose_page` — a full content-stream walk with no
    /// cache anywhere in `pdfcer-core`. An annotation's outline is its `/Rect`,
    /// four numbers in a dictionary, so it is re-read on the frame the
    /// selection is made and carried on the selection itself. See
    /// [`annot`]'s header table for the four ways the two differ.
    annot: Option<AnnotSelection>,
}

impl SelectionState {
    /// The selected entries, in document order.
    #[must_use]
    pub fn entries(&self) -> &[Selection] {
        &self.entries
    }

    /// Whether anything is selected — **content or annotation**.
    ///
    /// Both, deliberately: every caller of this is asking *"is there something
    /// for a verb to act on?"*, and answering only about content would leave a
    /// selected stamp reading as no selection at all. That is what would make
    /// the contextual Format tab hide itself over a selected annotation.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.annot.is_none()
    }

    /// The selected annotation, if the selection is one.
    #[must_use]
    pub const fn annot(&self) -> Option<&AnnotSelection> {
        self.annot.as_ref()
    }

    /// Select an annotation, **replacing** whatever was selected.
    ///
    /// Clearing the content selection is not a courtesy: it is the mutual
    /// exclusion this type owns. A build that left both set would draw two
    /// kinds of outline at once and leave `format.delete` with two plausible
    /// meanings, one of which removes page content the operator did not point
    /// at.
    ///
    /// The rung resets too, for [`Self::clear`]'s reason — a `Node`-rung state
    /// left behind an annotation selection would put the next content click
    /// straight into a rung the operator never entered.
    pub fn select_annot(&mut self, selection: AnnotSelection) {
        self.entries.clear();
        self.outlines.clear();
        self.resolved_for = None;
        self.level = SelectionLevel::Object;
        self.annot = Some(selection);
    }

    /// Drop an annotation selection, reporting whether there was one.
    ///
    /// Separate from [`Self::clear`] so a caller that is only retiring the
    /// annotation half — entering a mode that may not author markup, say —
    /// does not also destroy a content selection that mode still permits.
    pub fn clear_annot(&mut self) -> bool {
        self.annot.take().is_some()
    }

    /// **Drop the selection entirely**, reporting whether there was one.
    ///
    /// Back to the ground state — no entries, no outlines, the ladder at
    /// [`SelectionLevel::Object`] — which is what makes this different from
    /// [`Self::escape`]: escape *ascends one rung* and is the operator walking
    /// back up a structure they entered, while this is the selection ceasing to
    /// exist. Resetting the rung matters as much as emptying the list: a
    /// `Node`-rung state with nothing in it would put the next click's first
    /// selection straight into a rung the operator never entered.
    ///
    /// `resolved_for` is cleared too, so the empty state is not mistaken for
    /// one already resolved against a page and an epoch it no longer describes.
    ///
    /// The one caller is `PdfcerApp::on_mode_capabilities_changed`, entering a
    /// mode that cannot select page content — see there for why a selection is
    /// not "work" for the purposes of rule 1. It is deliberately **not** wired
    /// to any gesture: a click on empty paper narrows the selection through
    /// [`Self::click`]'s own rules, and a mis-aimed right-click is documented
    /// in [`crate::canvas::menus::select_under_right_click`] as something that
    /// must *not* destroy a set the operator spent five clicks building.
    ///
    /// # ★ It clears CONTENT only, and that is deliberate
    ///
    /// An annotation selection is governed by a different capability —
    /// `author_markup`, which **Review grants and Read does not**, where
    /// content selection needs `edit_content`, which only Edit grants. A mode
    /// change that lost one may keep the other, and Review is exactly that
    /// case: entering it from Edit must drop a selected path and keep a
    /// selected stamp.
    ///
    /// So the caller clears each half against its own predicate, and this one
    /// does not reach for [`Self::clear_annot`]. Folding them together here
    /// would be the "one on/off" the per-capability gate was built to avoid,
    /// reintroduced at the one place it is least visible.
    pub fn clear(&mut self) -> bool {
        if self.entries.is_empty() {
            return false;
        }
        self.entries.clear();
        self.outlines.clear();
        self.level = SelectionLevel::Object;
        self.resolved_for = None;
        true
    }

    /// How many entries are selected — the `sel=` the diagnostic trace
    /// reports, and the number `ui-verify` reads to tell a click that landed
    /// from one that did not.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// The rung the operator has entered.
    #[must_use]
    pub fn level(&self) -> SelectionLevel {
        self.level
    }

    /// The object the operator is inside, if any.
    ///
    /// Derived rather than stored, and safe to derive because every path that
    /// sets a level above [`SelectionLevel::Object`] also collapses the
    /// entries onto one object — a rung is a place *inside one thing*.
    #[must_use]
    pub fn entered_object(&self) -> Option<Selection> {
        (self.level != SelectionLevel::Object)
            .then(|| self.entries.first().copied())
            .flatten()
    }

    /// The canvas-space outlines to draw, with the entry each came from.
    ///
    /// Paired rather than a bare `Vec<Rect>` because the overlay needs to
    /// know which entry each box belongs to, and because [`Self::resolve`]
    /// drops entries the provider no longer knows — which breaks positional
    /// correspondence with [`Self::entries`].
    #[must_use]
    pub fn outlines(&self) -> &[(Selection, Rect)] {
        &self.outlines
    }

    /// The union of the current outlines, in canvas space — the box the
    /// resize grips are placed around.
    ///
    /// The union rather than the first entry's box, because a multi-select
    /// is one thing to act on: eight grips around one member of a set of
    /// five would say the gesture applies to that member alone.
    #[must_use]
    pub fn outline_union(&self) -> Option<Rect> {
        self.outlines
            .iter()
            .map(|(_, r)| *r)
            .reduce(|acc, r| acc.union(r))
    }

    /// The **page** paint-order indices selected on `page`, ascending — the
    /// operand list for a batched edit.
    ///
    /// Ascending and de-duplicated because `EditSession::delete_objects`
    /// resolves **every** index before planning anything, so a duplicate or a
    /// stale entry refuses the whole call rather than deleting the prefix
    /// that happened to resolve. Handing it a clean list is the difference
    /// between "delete refused" and "delete did half of what I asked".
    ///
    /// # ★★★ TARGETS INSIDE A FORM XOBJECT ARE NOT IN THIS LIST
    ///
    /// A selection can hold two kinds of thing —
    /// [`TargetId::Object`](crate::canvas::target::TargetId::Object), an index
    /// into the page's own paint order, and
    /// [`TargetId::Leaf`](crate::canvas::target::TargetId::Leaf), an index
    /// into the objects painted from inside a form XObject. **Only the first
    /// is an edit operand**, because every paint-order verb writes to the
    /// page's content stream and a leaf's token range indexes the form's.
    /// In range, wrong buffer, silent corruption — the engine's own reason for
    /// keeping the two lists apart, restated at the one funnel in this shell
    /// that feeds them to verbs.
    ///
    /// So a leaf is dropped here, and that is deliberate rather than a
    /// tolerated gap. It does mean **an empty return is not the same as an
    /// empty selection**: a caller that reports "nothing selected" on an empty
    /// list will contradict an outline the operator can see. Ask
    /// [`Self::leaf_indices_on`] before saying so — `canvas::moving` does, and
    /// declines with `Refusal::InsideForm` instead.
    #[must_use]
    pub fn object_indices_on(&self, page: usize) -> Vec<usize> {
        self.entries
            .iter()
            .filter(|e| e.page == page)
            .filter_map(|e| e.object.page_object_index())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// The indices into `PageObjects::leaves` selected on `page`, ascending
    /// and unique — the half [`Self::object_indices_on`] drops.
    ///
    /// ★ **This exists so a refusal can be worded.** Its one job is to let a
    /// caller tell *"you selected nothing"* from *"you selected something this
    /// verb cannot reach"*, which are the two states an operator most needs
    /// kept apart: the first is their mistake and the second is the program's
    /// limit. `RESUME.md` records four separate occasions where a limit
    /// reported as an absence cost weeks.
    ///
    /// Not an operand list. Nothing in `EditSession` takes one of these
    /// numbers, by design — see [`crate::canvas::target::TargetId`].
    #[must_use]
    pub fn leaf_indices_on(&self, page: usize) -> Vec<usize> {
        self.entries
            .iter()
            .filter(|e| e.page == page)
            .filter_map(|e| e.object.leaf_index())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Every target selected on `page`, both lists, ascending and unique.
    ///
    /// For a **readout** — the status line's *"what is selected"* — which must
    /// describe what the operator can see rather than what a verb can act on.
    /// Never hand a member of this to an edit verb; go through
    /// [`crate::canvas::target::TargetId::page_object_index`], which is the
    /// only thing that can say no.
    #[must_use]
    pub fn targets_on(&self, page: usize) -> Vec<TargetId> {
        self.entries
            .iter()
            .filter(|e| e.page == page)
            .map(|e| e.object)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// ★ The indices a **Delete** may act on for `page` — empty unless the
    /// operator is at the Object rung.
    ///
    /// # Why the rung guard lives here rather than at each call site
    ///
    /// Because there are now two call sites and they must not be able to
    /// disagree: the canvas's Delete/Backspace keys, and the ribbon's
    /// `format.delete` on the contextual Format tab (reached through
    /// `crate::app::PdfcerApp::dispatch_token`). A rule stated twice is a rule
    /// that drifts, and the drift here is destructive rather than cosmetic.
    ///
    /// # ★ And why the guard is not caution
    ///
    /// At the Part or Node rung the selection names a subpath or an anchor
    /// *inside* one object, while the only verb wired to it is
    /// `EditSession::delete_objects`, which removes **whole objects**. Deleting
    /// the enclosing object because the operator asked to delete one line of it
    /// is exactly the class of error that cannot be excused by "they can undo
    /// it": one measured CAD export holds an entire drawing view as a single
    /// path object with 1,194 subpaths, so the difference between the two
    /// readings is one line and the whole view.
    ///
    /// `pdfcer-core` has the verbs for the deeper rungs — `delete_subpath`,
    /// `delete_node` and `delete_text_run`, the last of which also needs the
    /// `ObjectModelProvider::text_run_delete_would_move_next` guard asked
    /// BEFORE the control is offered (R83). They are their own actions and
    /// their own change; refusing here is the honest interim.
    ///
    /// # Returns
    ///
    /// Ascending and de-duplicated — [`Self::object_indices_on`]'s contract,
    /// which is what `EditSession::delete_objects` needs in order to succeed
    /// rather than refuse the whole batch. Empty means *"nothing this verb may
    /// remove"*, which deliberately does **not** distinguish "nothing selected"
    /// from "selected at a rung with no delete verb": a caller that must tell
    /// those apart asks [`Self::level`], and the canvas does exactly that in
    /// order to trace the difference.
    #[must_use]
    pub fn deletable_objects_on(&self, page: usize) -> Vec<usize> {
        if self.level != SelectionLevel::Object {
            return Vec::new();
        }
        self.object_indices_on(page)
    }

    /// Apply a **completed click** — never a press.
    ///
    /// See invariant 2 in the module docs: a press that turns out to be a
    /// drag must leave the selection completely alone, so this is only ever
    /// reached from [`crate::canvas::gesture::GestureOutcome::Click`], which
    /// is raised on release and only when no drag happened.
    ///
    /// # The rules, and why each one
    ///
    /// - **Plain click, hit, at the Object rung** — replace the selection.
    /// - **Plain click, miss** — clear. Clicking empty paper deselects; that
    ///   is what every editor does, and the alternative strands an operator
    ///   with no way back to "nothing selected" except a key they have not
    ///   discovered.
    /// - **Shift+click, hit** — toggle that entry's membership, leaving the
    ///   rest alone. Toggle rather than add, so shift-clicking a selected
    ///   object is its own undo.
    /// - **Shift+click, miss** — unchanged. There is nothing to toggle, and
    ///   clearing here would make an over-shoot destroy a set the operator
    ///   spent five clicks building.
    /// - **Double-click, hit** — descend one rung into the object under the
    ///   pointer (the operator's stated model: *"double-click to get to the
    ///   next level down"*). A double-click at the Node rung changes nothing:
    ///   there is nothing below a point.
    /// - **Plain click while inside an object, hitting nothing in it** —
    ///   leave the rung and apply the ordinary Object-rung rule. Clicking
    ///   away is how every editor exits a group; staying inside until Escape
    ///   strands an operator who has forgotten they descended, which is the
    ///   failure a depth model must avoid above all.
    pub fn click(&mut self, page: usize, hit: ClickHit, shift: bool, double: bool) {
        // ★ A content click drops an annotation selection — HERE, in the type
        // that owns the exclusion, not at the call site.
        //
        // `canvas::interact` also clears it when a click misses every
        // annotation, and that is a different case: there, the click may go on
        // to mean *text* and never reach this function at all. Relying on that
        // one alone would leave the invariant owned by a caller — and an
        // invariant a caller maintains is one the next caller will not.
        //
        // Unconditional, including for a shift-extend: extending a content
        // selection while a stamp is selected still means the stamp is no
        // longer what is selected.
        self.annot = None;
        if double {
            self.descend(page, hit);
            return;
        }
        match self.level {
            SelectionLevel::Object => self.click_at_object_rung(page, hit, shift),
            SelectionLevel::Part | SelectionLevel::Node => self.click_inside(page, hit, shift),
        }
    }

    /// **The Node tool's click** — direct selection, with no descent ritual.
    ///
    /// # ★★ What this replaces, and why it is a separate entry point
    ///
    /// [`Self::click`] implements a *ladder*: a click selects an object, a
    /// double-click descends to its part, another descends to a node. That
    /// model is fine and it is what `move_node` and `move_subpath` are
    /// addressed through — but until 2026-08-19 it was **the only way to reach
    /// an anchor**, with nothing on screen at any stage saying a deeper rung
    /// existed. The operator's report:
    ///
    /// > *"How do I get to see the end points of an object and select them to
    /// > drag and move? This doesn't work either."*
    ///
    /// He is right, and the fix is the one every vector editor already uses:
    /// **the tool is the rung.** With the Node tool armed there is no state to
    /// descend through, so there is no way to be somewhere you did not choose.
    ///
    /// It is a *separate function* rather than a flag inside `click` because
    /// the two make different decisions at every branch — this one never
    /// ascends, never needs `entered_object`, and treats a click on a different
    /// object as "show me that one's anchors" rather than as "leave here". A
    /// boolean threaded through `click_at_object_rung` and `click_inside` would
    /// have made both harder to read and neither easier to test.
    ///
    /// # What it does
    ///
    /// - **Click on an anchor** → that anchor alone is selected, at the Node
    ///   rung, ready to drag.
    /// - **Click on a shape but not on an anchor** → the object is entered at
    ///   the Part rung with its nearest subpath, so **every anchor of that
    ///   subpath appears**. That is the step the ladder had no gesture for and
    ///   it is the one the operator was missing.
    /// - **Shift-click an anchor** → adds it, or removes it if it was already
    ///   in the set. `move_nodes` carries the whole set as one command.
    /// - **Click empty paper** → clears, which is the universal convention and
    ///   the one [`Self::marquee`] already follows.
    pub fn click_direct(&mut self, page: usize, hit: ClickHit, shift: bool) {
        // A content click drops an annotation selection, here, in the type that
        // owns the exclusion — exactly as `click` does and for the reason its
        // comment gives: an invariant a caller maintains is one the next caller
        // will not.
        self.annot = None;

        let Some(object) = hit.object else {
            // ★ Shift over empty paper clears too, and that is deliberate.
            // Shift means "add to what I have", and there is nothing there to
            // add; preserving the selection would make an aimless Shift-click a
            // no-op the operator cannot distinguish from a missed anchor.
            self.entries.clear();
            self.level = SelectionLevel::Object;
            return;
        };

        let entry = Selection {
            page,
            object,
            subpath: hit.part,
            node: hit.node,
        };

        // Shift only ever *extends within the same object*. Extending across
        // objects would build an operand list `move_nodes` cannot accept — it
        // addresses anchors within one object — and the refusal would arrive
        // after the operator had watched an outline slide.
        let same_object = self
            .entries
            .first()
            .is_some_and(|e| e.object == object && e.page == page);

        if shift && same_object && hit.node.is_some() {
            if let Some(at) = self.entries.iter().position(|e| *e == entry) {
                self.entries.remove(at);
                // Never leave the set empty at the Node rung: an empty set with
                // `level == Node` is the inconsistent state `normalise` exists
                // to prevent, and it would make the next plain click ambiguous.
                if self.entries.is_empty() {
                    self.entries.push(Selection {
                        page,
                        object,
                        subpath: hit.part,
                        node: None,
                    });
                    self.level = SelectionLevel::Part;
                }
            } else {
                self.entries.push(entry);
                self.level = SelectionLevel::Node;
            }
            self.normalise();
            return;
        }

        self.entries = vec![entry];
        self.level = if hit.node.is_some() {
            SelectionLevel::Node
        } else if hit.part.is_some() {
            // ★ The Part rung, NOT the Object rung, and this single line is most
            // of the fix. It is what makes the anchors appear on the very first
            // click — `painting::draw_anchors` draws the entered subpath's
            // anchors from the Part rung up, so entering it *is* showing them.
            SelectionLevel::Part
        } else {
            SelectionLevel::Object
        };
        self.normalise();
    }

    /// Replace or extend the selection with a marquee's enclosed set.
    ///
    /// Always resolves to the **Object** rung, and ascends if the operator
    /// was inside one. A rubber-band names a region of the page, and a region
    /// contains objects; there is no sensible reading of "every subpath of
    /// some other object that this box happens to cover".
    ///
    /// Plain replaces, `Shift` adds. An empty plain marquee therefore clears,
    /// which is the Inkscape convention and is **not** the failure invariant
    /// 2 is about: that one is about a *press*, and this runs on release,
    /// after a real enclosure test. Panning is the middle button and never
    /// reaches here at all.
    /// **Take a band's hits OUT of the selection** — `OPERATOR_REQUESTS.md`
    /// O104.
    ///
    /// The operator, 2026-09-03: *"I can't unselect things once I have selected
    /// them for redaction."* [`Self::marquee`] could replace or extend and had
    /// no third answer, so once several objects were picked the only way to
    /// drop one was to shift-click it precisely — which on a CAD sheet of
    /// overlapping strokes is often not practical.
    ///
    /// ★ An empty `hits` is a no-op rather than a clear, and the asymmetry with
    /// [`Self::marquee`] is deliberate. A band that encloses nothing means
    /// "replace the selection with nothing" when it is a plain band — that is
    /// how every editor cancels a selection — but "remove nothing from the
    /// selection" when it is a subtracting one. Clearing there would make a
    /// mis-aimed Ctrl-band destroy the very selection the operator was trying
    /// to refine, which is the opposite of what they asked for.
    ///
    /// ★★ The level is left alone. Subtracting is a change to WHICH objects are
    /// picked, never to how deep the selection has descended, and resetting it
    /// to `Object` would silently throw away an operator's descent into a form.
    pub fn marquee_remove(&mut self, page: usize, hits: &[TargetId]) {
        if hits.is_empty() {
            return;
        }
        let doomed: Vec<Selection> = hits
            .iter()
            .map(|&object| Selection::object(page, object))
            .collect();
        self.entries.retain(|e| !doomed.contains(e));
        self.normalise();
    }

    pub fn marquee(&mut self, page: usize, hits: &[TargetId], shift: bool) {
        let found: Vec<Selection> = hits
            .iter()
            .map(|&object| Selection::object(page, object))
            .collect();
        if shift {
            self.entries.extend(found);
        } else {
            self.entries = found;
        }
        self.level = SelectionLevel::Object;
        self.normalise();
    }

    /// ★★★ **Select one object outright**, because the program just put it
    /// there.
    ///
    /// # The complaint this closes
    ///
    /// The operator, 2026-08-26: *"if I add an image I Expect to click on it to
    /// resize but dragging doesn't resize."*
    ///
    /// He was right about the symptom and it was not the resize. A driven check
    /// had already proved that an image which **is** selected resizes from a
    /// corner grip (`resize-commit grip=SouthEast sx=0.6810 sy=0.5899`) and
    /// moves from a body drag. The image simply arrived **unselected** — so his
    /// first press landed on unselected paper, `gesture::meaning` read it as a
    /// marquee, and he watched a rubber band instead of a resize.
    ///
    /// # Why this is a convention rather than a convenience
    ///
    /// Every one of the eight applications surveyed for `HOW_IT_SHOULD_WORK.md`
    /// leaves a newly placed or pasted object **selected**, with its handles up.
    /// It is what makes "place it, then get it right" a single continuous act
    /// instead of a place, a hunt and a click. Nothing does otherwise.
    ///
    /// # Always the Object rung, and always replacing
    ///
    /// The **Object** rung because the operator has just created a whole thing,
    /// not entered one — the subpath and node rungs are places you descend to
    /// deliberately, and arriving inside a freshly placed image would be a
    /// state nobody asked for.
    ///
    /// **Replacing** rather than adding, even though a placement is arguably an
    /// addition: what was selected before is what the operator was working on
    /// *before* they placed this, and a two-object selection whose members were
    /// chosen minutes apart is not a set anybody intended to transform
    /// together.
    pub fn select_placed(&mut self, page: usize, object: TargetId) {
        self.select_only(page, object, "placed");
    }

    /// ★★ **Select exactly one object, from somewhere that is not the canvas.**
    ///
    /// The shared body of [`Self::select_placed`] and of the Objects panel's
    /// row click, and it exists as one function because the two are the same
    /// act: *"this, and only this, is now the thing being worked on."*
    ///
    /// `why` names the origin and reaches only the trace. It is a
    /// `&'static str` rather than an enum because nothing branches on it —
    /// the moment something does, it should become one.
    ///
    /// # ★★★ Why the Objects panel writes the SELECTION and not a focus
    ///
    /// It used to write `PanelsState::focus`, a second notion of *"the thing I
    /// am working on"* that the canvas knew nothing about and that only the
    /// Properties panel read. The audit of 2026-08-26 named that as the root of
    /// the operator's *"when I have an object selected like text the Tool tab
    /// doesn't switch to giving me the editable stuff for that object"*: there
    /// were three parallel answers to one question — the armed tool, the panel
    /// focus and the canvas selection — with no bridge between them and none of
    /// them authoritative.
    ///
    /// One notion, written from both ends. A row click selects on the canvas; a
    /// canvas click is what the panel describes. Neither can now disagree with
    /// the other, because there is nothing left to disagree with.
    pub fn select_only(&mut self, page: usize, object: TargetId, why: &'static str) {
        self.entries = vec![Selection::object(page, object)];
        self.level = SelectionLevel::Object;
        self.normalise();
        crate::diag::trace(move || {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            // ★ The list is named as well as the index. `object=7` was
            // unambiguous while a page had one index space; it has two now,
            // and a trace that cannot tell `objects[7]` from `leaves[7]` is a
            // trace that cannot be read back — which is how a wrong aim goes
            // six days without failing.
            let list = if object.is_leaf() { "leaf" } else { "object" };
            format!(
                "selection-set page={page} {list}={} via={why}",
                object.raw()
            )
        });
    }

    /// Every selected **anchor** on one object of one page, object-scoped,
    /// ascending and unique.
    ///
    /// # ★ Why this exists as its own accessor
    ///
    /// Because a multi-node selection has been *representable* since the Node
    /// rung landed — [`Self::pick_within`] adds a Shift-clicked anchor as its
    /// own entry, and [`Self::entries`] holds them all — and **nothing read it
    /// that way**. `canvas::moving::subject` asked [`Self::entered_object`],
    /// which is the FIRST entry, so an operator could Shift-click four anchors,
    /// watch four highlight, drag, and move one.
    ///
    /// A capability that the data model supports and no consumer reads is the
    /// hardest kind of gap to see: nothing is missing, nothing fails, and the
    /// unit tests of both halves pass. Giving it a name with a doc comment is
    /// what makes the next consumer ask the right question.
    ///
    /// # Why it filters on the object as well as the page
    ///
    /// `move_nodes` addresses anchors **within one object**, so a set spanning
    /// two objects is not one command. It cannot arise today — the Node rung is
    /// entered inside a single object and `click_inside` ascends the moment a
    /// click leaves it — but a caller that assumed otherwise would build an
    /// operand list the engine would refuse, and the refusal would arrive after
    /// the operator had watched an outline slide.
    #[must_use]
    pub fn selected_nodes_on(&self, page: usize, object: TargetId) -> Vec<usize> {
        self.entries
            .iter()
            .filter(|e| e.page == page && e.object == object)
            .filter_map(|e| e.node)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Ascend one rung, or clear, or decline the key. See [`EscapeOutcome`].
    ///
    /// **One press, one rung** — decision 025's L1. The old shell shipped
    /// Escape as "clear everything", so an operator who descended two rungs
    /// to reach a line and pressed Escape once found themselves back at the
    /// page. Collapsing the ladder in one press is exactly as wrong as
    /// requiring three presses to clear a single selection.
    pub fn escape(&mut self) -> EscapeOutcome {
        match self.level.ascend() {
            Some(SelectionLevel::Part) => {
                for entry in &mut self.entries {
                    entry.node = None;
                }
                self.level = SelectionLevel::Part;
                self.normalise();
                EscapeOutcome::LeftLevel(SelectionLevel::Part)
            }
            Some(SelectionLevel::Object) => {
                for entry in &mut self.entries {
                    entry.subpath = None;
                    entry.node = None;
                }
                self.level = SelectionLevel::Object;
                self.normalise();
                EscapeOutcome::LeftLevel(SelectionLevel::Object)
            }
            // `ascend()` never returns `Node`; the arm exists so adding a
            // fourth rung is a compile error here rather than a silent
            // fall-through to "clear the selection".
            Some(SelectionLevel::Node) => EscapeOutcome::Nothing,
            None if !self.entries.is_empty() => {
                self.entries.clear();
                self.outlines.clear();
                EscapeOutcome::ClearedSelection
            }
            None => EscapeOutcome::Nothing,
        }
    }

    /// ★ **Re-resolve against a fresh decomposition** — invariant 3.
    ///
    /// Called every frame; does real work only when `(page, epoch)` has
    /// moved, because that is the only time the answer can have changed. A
    /// zoom, a pan, a fit-mode change, a ribbon-tab change and a window
    /// resize all leave both halves of that key untouched, so they cost one
    /// comparison and change nothing — which is the invariant, enforced by
    /// the shape of the code rather than by a rule somebody has to remember.
    ///
    /// # Only the resolved page is validated, and that is the load-bearing part
    ///
    /// An entry on **another** page is left exactly as it was. The provider
    /// serves one page (`panels::objects::provider`, "Single-page by
    /// design"), so it has nothing to say about the others, and a `resolve`
    /// that pruned everything it could not find would wipe the selection the
    /// moment the operator paged away — and find nothing on the way back.
    /// That is the acceptance criterion, and it is this `if`.
    ///
    /// # What is dropped, and silently
    ///
    /// An entry whose object the provider no longer knows, i.e. one an edit
    /// removed. Dropping it is not a fact the operator needs disclosed: they
    /// deleted it. Keeping it would leave a selection naming a hole, and the
    /// next Delete would refuse the whole batch because
    /// `EditSession::delete_objects` resolves every index before planning.
    /// # `None` means "this page has no object model", not "nothing is selected"
    ///
    /// A page whose content streams will not decode has no decomposition, and
    /// the honest response is to draw no outlines while keeping the
    /// selection — the two states are different, and conflating them would
    /// make an undecodable page silently deselect. The `(page, epoch)` key is
    /// still recorded, so the failed decomposition is **not retried every
    /// frame**: the failure is deterministic (same bytes, same code), which is
    /// the same argument `PanelsState::provider_built` and
    /// `settle_and_rasterize`'s render-error hold both make.
    pub fn resolve(&mut self, targets: Option<&dyn CanvasTargetProvider>, page: usize, epoch: u64) {
        if self.resolved_for == Some((page, epoch)) {
            return;
        }
        self.resolved_for = Some((page, epoch));
        let Some(targets) = targets else {
            self.outlines.clear();
            return;
        };

        // Drop only entries ON THIS PAGE that no longer resolve.
        self.entries
            .retain(|e| e.page != page || targets.bounds(page, e.object).is_some());
        if self.entries.is_empty() {
            self.level = SelectionLevel::Object;
        }

        self.outlines = self
            .entries
            .iter()
            .filter(|e| e.page == page)
            .filter_map(|e| Some((*e, self.outline_rect(targets, e)?)))
            .collect();
    }

    /// Whether [`Self::resolve`] would do any work for `(page, epoch)`.
    ///
    /// The canvas asks **before** building a decomposition, because building
    /// one is the expensive half: `decompose_page` inflates, concatenates,
    /// tokenizes and walks every content stream on the page with no cache
    /// anywhere in `pdfcer-core`. A zoom, a pan, a fit change and a ribbon-tab
    /// change all leave both halves of the key untouched, so on the
    /// overwhelming majority of frames this is `false` and nothing is built
    /// at all.
    #[must_use]
    pub fn needs_resolve(&self, page: usize, epoch: u64) -> bool {
        self.resolved_for != Some((page, epoch))
    }

    /// The canvas-space rect to outline for one entry: the **part's** box
    /// once the operator is inside one, the object's box otherwise.
    ///
    /// Falling back to the object's box when a part has no bounds is
    /// deliberate — the alternative is drawing nothing for a selection that
    /// exists, and a correct action with no feedback is indistinguishable
    /// from a broken one.
    fn outline_rect(&self, targets: &dyn CanvasTargetProvider, entry: &Selection) -> Option<Rect> {
        // ★ A leaf has no page paint-order index, so it has no *part* box
        // either — the part rung is not offered for one. Its object box is,
        // and that is what gets outlined: `bounds` answers for both lists.
        entry
            .object
            .page_object_index()
            .and_then(|object| {
                entry
                    .subpath
                    .and_then(|part| targets.part_bounds(entry.page, object, part))
            })
            .or_else(|| targets.bounds(entry.page, entry.object))
    }

    /// A plain or shift click while at the Object rung.
    fn click_at_object_rung(&mut self, page: usize, hit: ClickHit, shift: bool) {
        match (shift, hit.object) {
            (false, Some(object)) => self.entries = vec![Selection::object(page, object)],
            (false, None) => self.entries.clear(),
            (true, Some(object)) => {
                let entry = Selection::object(page, object);
                if let Some(at) = self.entries.iter().position(|e| *e == entry) {
                    self.entries.remove(at);
                } else {
                    self.entries.push(entry);
                }
            }
            (true, None) => {}
        }
        self.normalise();
    }

    /// A plain or shift click while inside an object.
    ///
    /// Three outcomes, in precedence order: re-pick at the current rung; fall
    /// back one rung and re-pick there; or leave the object entirely and
    /// behave like an ordinary Object-rung click. The middle case is what
    /// stops an operator being stranded at a rung whose targets they keep
    /// missing — at the Node rung, a click that misses every anchor but lands
    /// on a part ascends to that part rather than doing nothing.
    fn click_inside(&mut self, page: usize, hit: ClickHit, shift: bool) {
        let Some(entered) = self.entered_object() else {
            // No entry to be inside of: the level and the entries disagreed,
            // which `normalise` prevents. Recover rather than panic.
            self.level = SelectionLevel::Object;
            self.click_at_object_rung(page, hit, shift);
            return;
        };
        let same_object = hit.object == Some(entered.object) && page == entered.page;

        if same_object && self.level == SelectionLevel::Node && hit.node.is_some() {
            self.pick_within(entered, hit.part.or(entered.subpath), hit.node, shift);
            return;
        }
        if same_object && let Some(part) = hit.part {
            // Either a re-pick at the Part rung, or the Node rung falling
            // back one rung onto a part.
            self.level = SelectionLevel::Part;
            self.pick_within(entered, Some(part), None, shift);
            return;
        }
        // The click left the object. Ascend and treat it as an ordinary
        // Object-rung click, which also covers "clicked a different object"
        // and "clicked empty paper".
        self.level = SelectionLevel::Object;
        self.click_at_object_rung(page, hit, shift);
    }

    /// Select a part or a node inside the entered object.
    fn pick_within(
        &mut self,
        entered: Selection,
        part: Option<usize>,
        node: Option<usize>,
        shift: bool,
    ) {
        let entry = Selection {
            page: entered.page,
            object: entered.object,
            subpath: part,
            node,
        };
        if shift {
            if let Some(at) = self.entries.iter().position(|e| *e == entry) {
                self.entries.remove(at);
            } else {
                self.entries.push(entry);
            }
        } else {
            self.entries = vec![entry];
        }
        self.normalise();
    }

    /// Descend one rung into whatever is under a double-click.
    ///
    /// A double-click on a **different** object enters that object rather
    /// than descending inside the current one: PDF path objects do not nest,
    /// so carrying a part or node index across would address an index in a
    /// different object's space.
    fn descend(&mut self, page: usize, hit: ClickHit) {
        // ★★★ **A LEAF DESCENDS TOO, as of 2026-09-01** —
        // `OPERATOR_REQUESTS.md` O70.
        //
        // A guard stood here for one day, refusing to descend into anything
        // painted inside a form XObject, and its reasoning was sound while it
        // lasted: the two deeper rungs were addressed by a page paint-order
        // index, `canvas::input::probe` answered `(None, None)` for a leaf, and
        // descending anyway would have set the Part rung with nothing
        // addressable in it — `canvas::painting` declining to draw anchors and
        // `pressing::grabbable` withholding the outline, so the operator's
        // second double-click would make the selection box VANISH and offer
        // nothing in its place.
        //
        // All three of those changed together, which is the only order in which
        // any of them should have: `provider::geometry` answers where a leaf's
        // subpaths and anchors are, `probe` asks it, the anchors draw, and the
        // drag routes to `pdfcer-core` Pass 188.0's `*_in_form` verbs. The rung
        // is addressable, so the ladder descends.
        let Some(object) = hit.object else {
            // A double-click is also a click, and a click on empty paper
            // leaves. Doing anything else here strands the operator.
            self.level = SelectionLevel::Object;
            self.entries.clear();
            self.outlines.clear();
            return;
        };
        let entered = self.entered_object();
        let same_object = entered.is_some_and(|e| e.object == object && e.page == page);

        let (level, entry) = match (same_object, self.level) {
            // Already inside this object at the Part rung: descend to Node.
            // With no anchor within tolerance the rung is still entered —
            // "inside this part, nothing picked yet" is a real state, and
            // refusing to descend would make the gesture feel unreliable on
            // exactly the curves whose anchors are hard to hit.
            (true, SelectionLevel::Part) => (
                SelectionLevel::Node,
                Selection {
                    page,
                    object,
                    subpath: hit.part.or_else(|| entered.and_then(|e| e.subpath)),
                    node: hit.node,
                },
            ),
            // Nothing is below a point.
            (true, SelectionLevel::Node) => return,
            // Entering an object (possibly a different one) from the top.
            _ => (
                SelectionLevel::Part,
                Selection {
                    page,
                    object,
                    subpath: hit.part,
                    node: None,
                },
            ),
        };
        self.level = level;
        self.entries = vec![entry];
        self.normalise();
    }

    /// Restore the two structural rules the rest of the module relies on.
    ///
    /// 1. **Entries are ordered and unique.** Document order, so the outlines
    ///    paint in a stable sequence rather than re-stacking on every
    ///    shift-click; unique, so a batched edit is handed a clean operand
    ///    list.
    /// 2. **A rung above `Object` means exactly one object is entered.** A
    ///    rung is a place *inside one thing*, and [`Self::entered_object`]
    ///    derives that from the first entry. Anything that would leave the
    ///    two disagreeing collapses to the Object rung instead — recovering
    ///    is better than asserting, because the state is reachable from a
    ///    marquee arriving while inside an object and the honest response is
    ///    to step out.
    fn normalise(&mut self) {
        self.entries.sort_unstable();
        self.entries.dedup();
        if self.entries.is_empty() {
            self.level = SelectionLevel::Object;
            self.outlines.clear();
            return;
        }
        if self.level != SelectionLevel::Object {
            let first = self.entries[0];
            if self
                .entries
                .iter()
                .any(|e| e.object != first.object || e.page != first.page)
            {
                self.level = SelectionLevel::Object;
                for entry in &mut self.entries {
                    entry.subpath = None;
                    entry.node = None;
                }
                self.entries.sort_unstable();
                self.entries.dedup();
            }
        }
        // The outlines describe the entries; any change to the entries makes
        // them stale, and a stale outline is a box drawn around something the
        // operator no longer has selected.
        self.resolved_for = None;
    }
}

// The selection algebra's assertions. Split out under R2; see its header for
// why the tests were the seam and the code was not.
#[cfg(test)]
mod tests;
