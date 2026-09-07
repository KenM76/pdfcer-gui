//! # `annotquad` — where an annotation's artwork **actually** sits, as four
//! corners rather than as an upright rectangle
//!
//! ## The operator's sentence this module exists to answer
//!
//! > *"the box outlined when an object is selected should be in the same angled
//! > orientation as the object."* — 2026-09-07, `OPERATOR_REQUESTS.md` **O147**
//!
//! The selection outline used to be drawn from `/Rect`, and §12.5.2 requires
//! `/Rect` **upright**: an annotation turned 30° was therefore bounded by an
//! axis-aligned rectangle visibly larger than the mark inside it, with the mark
//! floating in the middle at an angle the box did not share.
//!
//! ## ★★★ THIS MODULE WAS A WORKAROUND FOR ONE DAY, AND IS NOW A THIN ADAPTER
//!
//! It shipped on the morning of 2026-09-07 carrying **its own reader of the
//! appearance `/Matrix`** and **its own implementation of ISO 32000-1
//! §12.5.5's placement algorithm**, because `pdfcer_core::annot::Annotation`
//! modelled no rotation and `pdfcer-render`'s placement was `pub(crate)`. That
//! was filed the same morning
//! (`request_an_annotations_rotation_angle_cannot_be_read.md`), including an
//! addendum arguing that a public `appearance_placement` would be a better
//! shape than the field we had asked for.
//!
//! **`Pass 155.2` shipped that afternoon and gave us both**, and the engine's
//! reply answered the addendum in as many words: *"you were right that it is
//! the better shape … delete your matrix reader, your angle decomposition,
//! your `/AS` handling and your `MIN_BOX_EXTENT` copy."* All four are deleted.
//! What is left is the projection into canvas space, which is this shell's
//! business and nobody else's.
//!
//! ⇒ **The tripwire worked.** `the_engine_still_has_no_rotation_field` read the
//! pinned engine's own source and went red the moment `Annotation` grew
//! `appearance_matrix` — hours after it was written. It is kept, inverted, as
//! `tests::the_engine_owns_the_placement_and_this_module_only_projects`.
//!
//! ## What the engine now answers, and what is still ours
//!
//! | question | who answers |
//! |---|---|
//! | where do the artwork's four corners land, in page space? | `pdfcer_render::annot::appearance_placement` — §12.5.5's full algorithm, **the one the paint path runs** |
//! | what angle is that, in degrees? | `Annotation::appearance_rotation_degrees` — `None` for a shear, a mirror or a non-uniform scale |
//! | what are the raw six numbers? | `Annotation::appearance_matrix` |
//! | where is that **on this canvas**, at this zoom, on a `/Rotate 90` sheet? | **here**, via `crate::canvas::mapping::oriented_canvas_quad` |
//!
//! ★ The engine also publishes the free function
//! `pdfcer_core::annot::rotation_degrees([f64; 6])`, which decomposes a matrix
//! this shell does not have in its hand. **It is deliberately not called
//! here**: the method reads `appearance_matrix` *and* applies Table 95's
//! default for an appearance with no `/Matrix` key, so it answers `Some(0.0)`
//! where the free function would need this module to decide what an absent
//! matrix means — and deciding that here is precisely the private opinion about
//! somebody else's format that this module was rewritten to stop having.
//!
//! ★ **The corner order is the engine's and it is deliberate**: `[LL, LR, UR,
//! UL] of the /BBox` — *appearance* space, before the transform. Past 90° the
//! first element is no longer the leftmost point on the page. That is what lets
//! a caller draw an outline that follows the object and read a bearing off one
//! edge, and it is why `handles::GripFrame::Turned` documents its corners as
//! the **artwork's** frame rather than the page's.
//!
//! ★★ The engine pins, with a test of its own, that **the bearing of the first
//! placed edge equals `appearance_rotation_degrees()`** on the same annotation.
//! This shell reads the angle from one and draws the outline from the other, so
//! a divergence would put a grip where the artwork is not; that agreement is
//! held by an assertion on their side rather than by intent on ours.
//!
//! ## What is deliberately NOT done here
//!
//! **Nothing is drawn.** This module answers a geometric question and returns
//! numbers; the painter is `crate::canvas::overlay`. That split is what lets
//! one answer feed the outline, the grips, the properties panel's angle
//! read-out and a driven check that wants to assert on an orientation without a
//! screenshot.
//!
//! **No fallback quad is invented.** An annotation with no usable appearance
//! returns `None` and the caller keeps drawing `/Rect`, which for such an
//! annotation *is* where the mark is.

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
    /// ⚠⚠ **THE ENGINE DOES NOT NORMALISE, AND THIS COMMENT SAID IT DID FOR
    /// TWENTY MINUTES.** `Annotation::appearance_rotation_degrees` returns the
    /// `atan2` of the matrix directly, so a mark turned a quarter turn
    /// clockwise reads **`-89.15`**, not `270.85`. The first version of this
    /// adapter copied its old doc comment across unchanged — the local
    /// implementation it replaced *did* apply `rem_euclid` — and shipped a
    /// [`Self::is_upright`] whose range test `(0.1..=359.9)` therefore answered
    /// **`true` for every clockwise rotation**, so the selection outline went
    /// back to being axis-aligned on exactly the turn an operator makes most
    /// often.
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
/// A thin adapter over `pdfcer_render::annot::appearance_placement` and
/// `Annotation::appearance_rotation_degrees` — see the module header for what
/// this used to be and why it is not that any more.
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
