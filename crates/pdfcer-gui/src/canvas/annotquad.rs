//! # `annotquad` — where an annotation's artwork **actually** sits, as four
//! corners rather than as an upright rectangle
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/annotquad.md`.

use pdfcer_core::annot::Annotation;
use pdfcer_core::object::ObjId;
use pdfcer_core::page_tree::Page;
use pdfcer_core::view::DocumentView;

/// An annotation's artwork as it is actually placed on the page.
///
/// Page space, PDF convention (y up), the same space `/Rect` is in — mapping to
/// canvas or screen space is the caller's job, through
/// [`crate::canvas::mapping::oriented_canvas_quad`], so this shares the
/// projection every other overlay uses.
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
    /// ★ An annotation with an appearance but **no `/Matrix` key** answers
    /// `Some(0.0)` — Table 95's default, and what the renderer paints with —
    /// while `Annotation::appearance_matrix` answers `None`, because the file
    /// really did say nothing. The engine draws that distinction deliberately,
    /// so an ordinary unrotated mark shows `0°` in a properties field rather
    /// than a blank.
    pub degrees: Option<f64>,
}

impl OrientedBox {
    /// Is this box upright — i.e. would drawing `/Rect` give the same picture?
    ///
    /// Used by callers that want to keep the cheap path (and the existing
    /// axis-aligned grip geometry) for the overwhelmingly common unturned case.
    /// The tolerance is a **tenth of a degree**, which at the width of a sheet
    /// is well under a pixel and is far larger than any float noise a
    /// `/Matrix` round-trip introduces.
    ///
    /// ★ **A quarter turn is NOT upright by this test, and that is deliberate.**
    /// A 90°-turned annotation's `/Rect` does bound it exactly, so an outline
    /// drawn from `/Rect` would look right — but its *corner order* has rotated,
    /// so a grip the operator grabs at the artwork's own top-left is at the
    /// page's bottom-left. Answering `true` here would put the grips back on the
    /// page's frame and quietly reintroduce that mismatch. Only a genuinely
    /// unturned appearance, and a matrix that is not an angle at all (where
    /// there is no orientation to honour), take the cheap path.
    #[must_use]
    pub fn is_upright(&self) -> bool {
        self.degrees.is_none_or(|d| !(0.1..=359.9).contains(&d))
    }
}

/// **Where `annot`'s artwork actually sits**, or `None` when the question has
/// no honest answer for it.
///
///
/// `None` is a **correct** answer, not a failure, and the engine lists its
/// causes: no `/Rect`, no reachable appearance stream, no readable `/BBox`, or
/// a transformed box so thin that §12.5.5 step (b)'s fit matrix is singular. In
/// every one of those the caller should keep using `/Rect`, which for an
/// annotation with no appearance **is** where the mark is.
#[must_use]
pub fn oriented(view: &DocumentView<'_>, annot: &Annotation) -> Option<OrientedBox> {
    Some(OrientedBox {
        corners: pdfcer_render::annot::appearance_placement(view, annot)?,
        // ★★★ `rem_euclid`, and read [`OrientedBox::degrees`] before removing
        // it: the engine returns a signed `atan2` and this shell needs
        // `[0, 360)`. Omitting it made every clockwise rotation report itself
        // upright and cost a driven run to find.
        degrees: annot
            .appearance_rotation_degrees()
            .map(|d| d.rem_euclid(360.0)),
    })
}

/// **[`oriented`] for one annotation named by id**, walking the page to find it.
///
/// The shape most callers here want: the selection and the properties panel
/// both hold an `ObjId` and a page, not an `Annotation`.
///
/// ★ Through `annot::page_annotations` rather than a hand-rolled dictionary
/// read, because `appearance_placement` takes the engine's own modelled
/// `Annotation` — including its `Appearance` enum, which is where `/AS`
/// selection and the *"named but not painted"* distinction live. A shell that
/// built an `Annotation` by hand to pass in here would be re-implementing
/// exactly the part the engine was asked to take over.
///
/// # Cost
///
/// One `/Annots` walk, bounded by `pdfcer_core::annot::MAX_ANNOTS_PER_PAGE`,
/// plus one appearance-stream resolve. Called on selection and on the frame
/// after an edit — see [`crate::canvas::selection::SelectionState`]'s
/// `resolve_annot` — never per frame per annotation.
#[must_use]
pub fn oriented_by_id(view: &DocumentView<'_>, page: &Page, id: ObjId) -> Option<OrientedBox> {
    let annot = pdfcer_core::annot::page_annotations(view, page.id)
        .into_iter()
        .find(|a| a.id == Some(id))?;
    oriented(view, &annot)
}

#[cfg(test)]
mod tests;
