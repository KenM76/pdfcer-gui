//! # `annotquad` — where an annotation's artwork **actually** sits, as four
//! corners rather than as an upright rectangle
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/annotquad.md`.

use pdfcer_core::annot::Annotation;
use pdfcer_core::object::ObjId;
use pdfcer_core::page_tree::Page;
use pdfcer_core::view::DocumentView;

/// An annotation's artwork as it is actually placed on the page.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrientedBox {
    /// The four corners, in the engine's order: `[LL, LR, UR, UL]` **of the
    /// appearance `/BBox`**, each pushed through §12.5.5's placement.
    ///
    /// Those names describe the **artwork's** frame, not the page's — after a
    /// 100° turn corner 0 is at the top of the screen. That is the engine's
    /// documented choice and it is the useful one: it lets a caller draw a
    /// closed outline that follows the object and read a bearing off one edge.
    pub corners: [(f64, f64); 4],
    /// The rotation the appearance `/Matrix` expresses, in **degrees
    /// anticlockwise**, normalised by **this module** into `[0, 360)`.
    ///
    ///
    /// It was caught by `tools/ui-verify`'s `rotating_a_markup_turns_it` on the
    /// first driven run after the pin moved, with `turned=0` beside a trace
    /// line saying the engine had turned the mark. **3,860 in-process tests
    /// were green**, including four in this module — because every one of them
    /// used a *positive* angle.
    ///
    /// ⇒ **A contract you write for somebody else's function is a claim to
    /// measure, not to carry over.** The normalisation is done here, once, and
    /// tested with a negative angle beside a positive one.
    ///
    /// `None` when the matrix is not a rotation with an optional uniform
    /// positive scale — a skew, a mirror, or an anisotropic scale is **not an
    /// angle** and the engine does not report one. The corners are still
    /// correct in that case, which is why this is a separate field: an outline
    /// can be drawn round a sheared stamp; a number cannot be put in a
    /// properties field for it.
    ///
    /// An annotation with an appearance but **no `/Matrix` key** answers
    /// `Some(0.0)` — Table 95's default, and what the renderer paints with —
    /// while `Annotation::appearance_matrix` answers `None`, because the file
    /// really did say nothing. The engine draws that distinction deliberately,
    /// so an ordinary unrotated mark shows `0°` in a properties field rather
    /// than a blank.
    pub degrees: Option<f64>,
}

impl OrientedBox {
    /// Is this box upright — i.e. would drawing `/Rect` give the same picture?
    #[must_use]
    pub fn is_upright(&self) -> bool {
        self.degrees.is_none_or(|d| !(0.1..=359.9).contains(&d))
    }
}

/// **Where `annot`'s artwork actually sits**, or `None` when the question has
/// no honest answer for it.
#[must_use]
pub fn oriented(view: &DocumentView<'_>, annot: &Annotation) -> Option<OrientedBox> {
    Some(OrientedBox {
        corners: pdfcer_render::annot::appearance_placement(view, annot)?,
        // `rem_euclid`, and read [`OrientedBox::degrees`] before removing
        // it: the engine returns a signed `atan2` and this shell needs
        // `[0, 360)`. Omitting it made every clockwise rotation report itself
        // upright and cost a driven run to find.
        degrees: annot
            .appearance_rotation_degrees()
            .map(|d| d.rem_euclid(360.0)),
    })
}

/// **[`oriented`] for one annotation named by id**, walking the page to find it.
#[must_use]
pub fn oriented_by_id(view: &DocumentView<'_>, page: &Page, id: ObjId) -> Option<OrientedBox> {
    let annot = pdfcer_core::annot::page_annotations(view, page.id)
        .into_iter()
        .find(|a| a.id == Some(id))?;
    oriented(view, &annot)
}
