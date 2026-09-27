//! # `annotselection` — An annotation the operator has selected on the canvas: which one, and where its outline sits.

use egui::{Pos2, Rect};
use pdfcer_core::object::ObjId;

use crate::annotkind::AnnotKind;

/// One annotation, addressed the way the engine addresses it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotTarget {
    /// The page it lives on.
    ///
    /// Carried for the same reason [`crate::selectionidentity::Selection::page`] is: it lets a
    /// selection survive navigating away and back, and Phase 4 puts several
    /// pages on screen at once.
    pub page: usize,
    /// The annotation's object id — **stable**, unlike a content object's
    /// paint-order index.
    ///
    /// This is what every `EditSession` annotation verb takes, so a selection
    /// made here can be acted on without a second lookup that could resolve
    /// differently.
    pub id: ObjId,
    /// Which verb may restyle it. See [`AnnotKind`].
    pub kind: AnnotKind,
    /// `/Subtype`, as the file spells it — `Stamp`, `Square`, `Line`, `Text`.
    ///
    /// Operator-facing, through [`crate::text`]: the status line and the
    /// Format tab both say *what* is selected, and "Stamp" is the word the
    /// operator used when they placed it.
    pub subtype: String,
    /// §12.5.3 Table 165 bit 8 — the file says the user interface may not
    /// change this annotation's properties.
    ///
    /// Carried on the target rather than checked at each verb, so a surface
    /// can **omit** the controls it governs rather than offer them and let the
    /// engine refuse. That is R83: an affordance that cannot be honoured is
    /// not drawn.
    pub locked: bool,
}

/// A selected annotation, with the outline to draw for it.
#[derive(Debug, Clone, PartialEq)]
pub struct AnnotSelection {
    /// What is selected.
    pub target: AnnotTarget,
    /// Its `/Rect`, in **canvas space** — the zoom-independent space the
    /// content selection's outlines are also cached in, so a zoom or a pan
    /// moves where this is drawn without changing what it is.
    pub outline: Rect,
    /// **Where the artwork ACTUALLY sits**, when that differs from
    /// [`Self::outline`] — the four corners of the appearance's placed
    /// `/BBox`, canvas space, in the artwork's own frame (see
    /// `pdfcer_gui::canvas::annotquad`).
    ///
    /// `None` for the overwhelmingly common unturned case **and** for any
    /// annotation with no usable appearance stream, and both mean the same
    /// thing to a caller: *use the rectangle, it is the truth here.*
    ///
    /// # Why this rides on the selection rather than being re-read at paint
    ///
    /// Because the outline, the grips and the hit test must agree about where
    /// the object is, and this project has been bitten three times by two
    /// surfaces deriving the same geometry independently — most recently when a
    /// dimension's vertex handles were painted from the selection and
    /// hit-tested from a capability check, so they were visible and untouchable
    /// in the very mode that authors dimensions. One value, one decision, every
    /// consumer.
    pub oriented: Option<[Pos2; 4]>,
}
