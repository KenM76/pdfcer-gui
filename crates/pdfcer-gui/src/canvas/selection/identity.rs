//! # `canvas::selection::identity` — a selection is four integers, never a position
//!
//! ## Which half of `canvas::selection` this file is
//!
//! [The parent module](super) holds the mutable state that *accumulates*
//! selections — [`SelectionState`](super::SelectionState), the ladder it walks,
//! the `(page, epoch)`-keyed re-resolve, and every rule about what a click
//! means. This file holds the **vocabulary that state is made of**:
//! [`Selection`], [`SelectionLevel`], [`ClickHit`] and [`EscapeOutcome`].
//!
//! Every type here is `Copy`, none of them owns anything, none of them has a
//! method that mutates, and — the point of the split — **none of them can name
//! a place on the screen**. The parent re-exports all four, so
//! `crate::canvas::selection::SelectionLevel` is still the path every caller
//! uses; the file boundary is for the reader, not for the type system.
//!
//! ## ★ Selection is an identity, not a position
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
//! The roadmap names **three** ways the natural implementation loses that, each
//! of which looks reasonable in isolation; [the parent module](super)'s header
//! tabulates all three against what closes them. **The first is closed here,
//! and it is the only one of the three that is closed by construction rather
//! than by behaviour:**
//!
//! > **Selection stored in screen coordinates.** Zoom changes the mapping, so
//! > the stored point stops naming the thing it named.
//!
//! [`Selection`] holds **no coordinate of any kind**. It is
//! `page + object + subpath + node`, four integers, none of which a zoom can
//! touch. There is no constructor that takes a `Pos2`.
//!
//! Having that as a property of a *type* rather than as a rule somebody
//! maintains is the whole reason these four declarations are worth a file of
//! their own. A method can be edited to keep a cached point "in step" with the
//! view and still read as reasonable in review — that is precisely how the
//! defect arrives. A field that does not exist cannot be kept in step with
//! anything. Everything that could go wrong with the selection's *shape* is
//! therefore visible on one screen, and the other two failure modes — a press
//! that turned out to be a drag, and a re-decomposition that discards instead
//! of re-resolving — are behavioural, so they are closed by
//! [`SelectionState`](super::SelectionState) and argued there.
//!
//! ## Why paint-order index is the identity, and what it does not survive
//!
//! `Selection::object` is a [`TargetId`], which is the object's index into
//! `PageObjects::objects` — **paint order**. It is the same number
//! `pdfcer object-list` prints and `object-delete` takes, so "object 412"
//! means one thing across every surface. That is what makes it usable as an
//! identity here.
//!
//! It is an identity **within one revision of one page**, and no further.
//! Deleting an object renumbers every object painted after it. So an edit
//! moves the meaning of a retained index, and this module's honest position
//! is to say so rather than to pretend otherwise:
//!
//! - A rebuild with the **same** revision (the zoom / page-return / panel
//!   case, which is the invariant's whole subject) re-resolves exactly.
//! - A rebuild after an **edit** re-resolves against the new decomposition
//!   and drops what no longer exists; indices that *shifted* silently name
//!   their new neighbour. Closing that needs a stable per-object token from
//!   `pdfcer-core`, which does not exist — `decompose_page` mints indices, not
//!   identities. It is recorded here rather than in a comment nobody reads,
//!   and it is a boundary finding for the engine, not a shortcut taken here.
//!
//! ## What this file deliberately does NOT do
//!
//! It never draws, never touches egui, never reads a pointer, and never
//! reaches a document — and beyond that it does not *decide* anything. There is
//! no rule here about what a click means, when a rung is entered, or what
//! Escape leaves behind: four plain data types and one three-line
//! [`SelectionLevel::ascend`]. Every behavioural question lives one level up,
//! which is what lets that level be read as a state machine rather than as a
//! state machine tangled with its own data definitions.

use crate::canvas::target::TargetId;

/// One selected thing, addressed by **identity** and never by position.
///
/// Four integers, and the shape is `GUI_ROADMAP.md`'s — *"page, object index,
/// sub-path, node"*. Enough to re-resolve against a fresh decomposition, and —
/// the point — containing nothing a zoom, a pan or a fit mode could
/// invalidate.
///
/// `Ord` so a selection set has a stable, reviewable order and so a
/// [`BTreeSet`](std::collections::BTreeSet) can de-duplicate it. The ordering
/// is `(page, object, subpath, node)`, i.e. document order first, which is also
/// the order the outlines are painted in — a multi-select that painted in click
/// order would re-stack its outlines whenever the operator shift-clicked, which
/// reads as flicker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Selection {
    /// The 0-based page index the object lives on.
    ///
    /// Carried even though the canvas draws one page today, because it is
    /// what lets a selection survive navigating away and back — the case the
    /// acceptance criterion turns on — and because `GUI_ROADMAP.md` Phase 4
    /// puts several pages on screen at once.
    pub page: usize,
    /// The object, by paint-order index (see the module docs on identity).
    pub object: TargetId,
    /// The entered part — a path's subpath or a text object's show-operator
    /// run — if the operator has descended one rung.
    ///
    /// `None` means "the whole object", which is a different statement from
    /// "part 0".
    pub subpath: Option<usize>,
    /// The entered anchor, **object-scoped**, if the operator has descended
    /// two rungs.
    ///
    /// Object-scoped rather than part-scoped because that is the space
    /// `vector::anchor_count` reports and `pdfcer node-move --node N`
    /// addresses; a second numbering would make the number pdfcer shows
    /// disagree with the number the operator can act on.
    ///
    /// `Some` implies `subpath.is_some()`: there is no way to pick a point
    /// without being inside the part that holds it.
    /// [`SelectionState`](super::SelectionState) is the only thing that
    /// constructs these and it maintains that.
    pub node: Option<usize>,
}

impl Selection {
    /// A whole-object selection.
    #[must_use]
    pub fn object(page: usize, object: TargetId) -> Self {
        Self {
            page,
            object,
            subpath: None,
            node: None,
        }
    }
}

/// Which rung of the selection ladder the operator has entered.
///
/// Three rungs, and the ladder is the vector-editor convention the operator
/// asked for: double-click descends, Escape ascends **one rung per press**.
///
/// The cap is structural rather than checked. A text object decomposes into
/// runs, and a run has no anchors, so
/// [`CanvasTargetProvider::nearest_node`](crate::canvas::target::CanvasTargetProvider::nearest_node)
/// can never return a node for one — the ladder stops at two rungs for text
/// without a special case anywhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum SelectionLevel {
    /// Whole objects. The rung a click starts on and Escape returns to.
    #[default]
    Object,
    /// Inside one object, selecting its parts — a path's subpaths or a text
    /// object's runs.
    ///
    /// This rung exists because a PDF path object can hold an entire drawing:
    /// one measured CAD export has **1,194 subpaths in a single object**, so
    /// "the object under the pointer" is usually not the thing the operator
    /// means.
    Part,
    /// Inside one part, selecting its anchors.
    ///
    /// Scoped to the entered part deliberately: the same measured export has
    /// one object holding **6,681 anchors**, and offering all of them as a
    /// grab target is what made the old ungated gesture unpredictable — the
    /// nearest anchor to a press could easily belong to a subpath the
    /// operator was not pointing at, with nothing drawn to say which.
    Node,
}

impl SelectionLevel {
    /// The rung one step up, or `None` at the top.
    #[must_use]
    pub fn ascend(self) -> Option<Self> {
        match self {
            Self::Object => None,
            Self::Part => Some(Self::Object),
            Self::Node => Some(Self::Part),
        }
    }
}

/// What one press of Escape did — reported rather than silently absorbed.
///
/// The caller traces it and, in the `Nothing` case, is free to let Escape
/// fall through to whatever else owns the key. Returning a value rather than
/// a `bool` is what keeps *"Escape ascends exactly one rung"* assertable:
/// a test can press Escape three times and check the three outcomes in
/// order, which a `bool` could not distinguish from one press that collapsed
/// the whole ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeOutcome {
    /// Left the entered rung, returning to the one above. The selection is
    /// **not** cleared — that is the next press's job.
    LeftLevel(SelectionLevel),
    /// Was at the Object rung with something selected: cleared it.
    ClearedSelection,
    /// Nothing was selected and no rung was entered. The canvas did not
    /// consume the key.
    Nothing,
}

/// What the provider found under a completed click, at every rung at once.
///
/// Assembled by the canvas — which owns the provider and the coordinate
/// conversion — and handed here as plain integers, so
/// [`SelectionState::click`](super::SelectionState::click) is a pure function
/// of "what is there" and "where am I" with no geometry in it. Every branch of
/// the ladder is then testable without a document, a decomposition or an egui
/// frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClickHit {
    /// The front-most object under the pointer, if any.
    pub object: Option<TargetId>,
    /// The nearest part of the **entered** object, if the click was inside
    /// one and a part was within tolerance.
    pub part: Option<usize>,
    /// The nearest anchor of the **entered** part, object-scoped.
    pub node: Option<usize>,
}
