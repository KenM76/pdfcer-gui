//! # `selectionidentity` — a selection is four integers, never a position
//!
//! ## Which half of `canvas::selection` this file is
//!
//! [The parent module](super) holds the mutable state that *accumulates*
//! selections — [`SelectionState`](super::SelectionState), the ladder it walks,
//! the `(page, epoch)`-keyed re-resolve, and every rule about what a click
//! means. This file holds the **vocabulary that state is made of**:
//! [`Selection`], [`SelectionLevel`], [`ClickHit`] and [`EscapeOutcome`].
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/selectionidentity.md`.

use crate::canvastarget::TargetId;

/// One selected thing, addressed by **identity** and never by position.
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
    /// The rung's name on the `PDFCER_DIAG` channel.
    #[must_use]
    pub fn traced(self) -> &'static str {
        match self {
            // ui-text-exempt: diagnostic trace tokens, never displayed
            Self::Object => "Object",
            Self::Part => "Part",
            Self::Node => "Node",
        }
    }

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClickHit {
    /// The front-most object under the pointer, if any.
    pub object: Option<TargetId>,
    /// The nearest part of the **entered** object, if the click was inside
    /// one and a part was within tolerance.
    pub part: Option<usize>,
    /// The nearest anchor of the **entered** part, object-scoped.
    pub node: Option<usize>,
    /// Whether [`Self::part`] names a **text chunk the operator can see** — a
    /// line of a multi-line text object, with `View ▸ Text chunks` switched on
    /// and a box drawn around it.
    ///
    /// Not filled by `pdfcer_gui::canvas::input::probe`, and that is the one
    /// field of this struct that is not. `probe` holds a
    /// [`CanvasTargetProvider`](crate::canvastarget::CanvasTargetProvider),
    /// which can say what class an object is but cannot count a text object's
    /// lines, and it holds no [`egui::Context`], so it can reach neither half
    /// of the question. `pdfcer_gui::canvas::chunks::boxed` answers both, and
    /// `canvas::clicking` — the one caller holding the document, the preference
    /// and the point at once — sets this immediately after probing. Everywhere
    /// else it is `false`, which is the ladder's behaviour with the boxes off.
    ///
    /// It gates one rule and no other: a plain click at the Object rung, on the
    /// object that is **already** the whole selection, narrows to this chunk
    /// instead of re-selecting the block. A path's subpath never sets it, so
    /// clicking a line of vector work twice still leaves the object selected.
    pub chunk: bool,
}
