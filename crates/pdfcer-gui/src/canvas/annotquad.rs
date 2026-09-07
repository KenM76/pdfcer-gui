//! # `annotquad` — where an annotation's artwork **actually** sits, as four
//! corners rather than as an upright rectangle
//!
//! ## The operator's sentence this module exists to answer
//!
//! > *"the box outlined when an object is selected should be in the same angled
//! > orientation as the object."* — 2026-09-07, `OPERATOR_REQUESTS.md` **O147**
//!
//! He is right, and until this module existed the shell could not do it. The
//! selection outline was drawn from `/Rect`, and §12.5.2 requires `/Rect`
//! **upright**: an annotation turned 30° is therefore bounded by an axis-aligned
//! rectangle visibly larger than the mark inside it, with the mark floating in
//! the middle at an angle the box does not share. `pdfcer-core`'s own
//! `rotate_annotation` doc comment predicted precisely this experience —
//! *"an operator turning a stamp 30° watches a dashed box swell around artwork
//! that did not change size"* — and this shell's answer had been an off-canvas
//! **sentence** explaining it. That is Rule 4's surviving half done correctly
//! and it was still the wrong answer, because the operator does not want the
//! swelling explained. He wants the outline to fit.
//!
//! ## What is computed here, in one line
//!
//! **ISO 32000-1 §12.5.5's appearance placement algorithm, run forwards on the
//! four corners of the appearance stream's `/BBox`** — producing the same
//! quadrilateral the *renderer* draws the artwork inside, in page space.
//!
//! ```text
//!   (a)  quad   = /Matrix  x  the four corners of /BBox      <- may be at any angle
//!   (b)  bounds = the upright box of `quad`
//!   (c)  A      = the scale+translate that maps `bounds` onto /Rect
//!        corners = A x quad                                  <- what is returned
//! ```
//!
//! Step (c) is the one nobody expects and it is normative: §12.5.5 requires the
//! transformed appearance box to **fit `/Rect` exactly**, so a `/Rect` that is
//! larger than the artwork needs *stretches the artwork to fill it*. Running the
//! algorithm rather than reasoning about it is what makes this outline correct
//! on a file pdfcer did not author, on an annotation whose producer wrote a
//! `/Matrix` for its own reasons, and — importantly today — on an annotation
//! that has been damaged by the growth defect in **O145**, where it hugs the
//! *grown* artwork instead of quietly disagreeing with the screen.
//!
//! ## ★★★ THIS MODULE IS A WORKAROUND AND IT IS FILED AS ONE
//!
//! `pdfcer_core::annot::Annotation` models `rect`, `vertices`, `line`,
//! `ink_list`, `color`, `icon`, `state`, `constant_alpha` and an `Appearance`
//! enum that carries a `stream_id` **and nothing else**. There is no rotation
//! and no matrix, so there is no supported route to the fact this module needs.
//!
//! `EditSession::value` and `EditSession::graph` are public, so the shell *can*
//! walk `/AP` → `/N` → (`/AS`) → `/Matrix` itself, and that is what happens
//! below. It is filed at the engine as
//! `request_an_annotations_rotation_angle_cannot_be_read.md`, per decision 058:
//! **a workaround the GUI did not report is a boundary defect that stays.**
//!
//! ⚠ **This is a second reader of a structure `pdfcer-core` owns**, and it is
//! the weaker one by construction — it does not know what `Appearance::
//! StateUnresolved` knows, it re-implements §12.5.5 rather than sharing the
//! renderer's implementation (`pdfcer-render`'s is `pub(crate)`), and every
//! future clause the engine learns about appearance selection is a clause this
//! file will get wrong. `AWAITED_ENGINE_FIELDS` and the test that reads it are
//! the tripwire that fires the day the engine makes this module unnecessary, so
//! it cannot outlive its cause in silence — the mechanism this project adopted
//! after a shim survived two hours past its own obsolescence with nothing to
//! notice.
//!
//! ## What is deliberately NOT done here
//!
//! **Nothing is drawn.** This module answers a geometric question and returns
//! numbers; the painter is `canvas::overlay`. That split is what lets the same
//! answer feed the outline, the properties panel's angle read-out, and any
//! driven check that wants to assert on an orientation without a screenshot.
//!
//! **No fallback quad is invented.** An annotation with no usable appearance
//! stream returns `None` and the caller keeps drawing `/Rect`, which for such an
//! annotation *is* the truth — a `/Square` with no `/AP` is a rectangle, and a
//! shell that guessed an angle for it would be inventing one.

use pdfcer_core::graph::ObjectGraph;
use pdfcer_core::object::{Dict, ObjId, Object};

/// **The names this module is waiting for the engine to publish.**
///
/// The day any of these appears as a field of `pdfcer_core::annot::Annotation`,
/// this module's whole reason to exist has been removed, and
/// `tests::the_engine_still_has_no_rotation_field` goes red naming it.
///
/// # ★★★ Why this is a grep of the ENGINE's source and not a `debug_assert`
///
/// The first draft of this tripwire was `const ENGINE_HAS_TAKEN_OVER: bool =
/// false;` with a `debug_assert!(!ENGINE_HAS_TAKEN_OVER)` beside it, and that
/// is **not a tripwire at all**: nothing can set it but a human who has already
/// noticed, which is the one case a tripwire is not needed for. (Clippy flagged
/// it as *"this assertion has a constant value"*, and was right for a better
/// reason than it knew.)
///
/// A tripwire has to be keyed on **the other side's API**, not on this side's
/// intention. The test reads the **pinned** engine checkout — the exact bytes
/// this build compiles against, located from `Cargo.lock` so it cannot drift
/// from the pin — and fails when one of these names appears in the read model.
/// It costs one file read per `cargo test` run and it fires without anybody
/// looking.
#[cfg(test)]
const AWAITED_ENGINE_FIELDS: [&str; 3] = ["rotation", "appearance_matrix", "matrix"];

/// An annotation's artwork as it is actually placed on the page.
///
/// Page space, PDF convention (y up), the same space `/Rect` is in — mapping to
/// canvas or screen space is the caller's job and is done through
/// `viewer::pdf_space_to_canvas` so this shares the projection every other
/// overlay uses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrientedBox {
    /// The four corners, in the order the `/BBox` corners were taken:
    /// lower-left, lower-right, upper-right, upper-left **of the `/BBox`**,
    /// each pushed through the placement. After a rotation those names describe
    /// the artwork's own frame, not the page's — corner 0 is the artwork's
    /// bottom-left wherever on screen that has ended up, which is exactly what
    /// a caller drawing a closed outline wants.
    pub corners: [(f64, f64); 4],
    /// The rotation the appearance `/Matrix` expresses, in **degrees
    /// anticlockwise**, normalised to `[0, 360)`.
    ///
    /// `None` when the matrix is not a rotation with an optional uniform
    /// positive scale — a skew, a mirror, or an anisotropic scale is **not an
    /// angle** and must not be reported as one. The corners are still correct in
    /// that case, which is why this is a separate field rather than the whole
    /// return value: an outline can be drawn round a sheared stamp; a number
    /// cannot be put in a properties field for it.
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

/// A 2-D affine matrix in PDF order, `[a b c d e f]` (§8.3.3).
///
/// Maps `(x, y)` to `(a·x + c·y + e, b·x + d·y + f)`. Kept local and tiny
/// rather than reaching for `pdfcer_core::vector::geometry::Matrix` because the
/// only operations needed here are *apply to a point* and *read `a` and `b`*,
/// and a local four-line type is one less public API this module depends on
/// while it is a workaround waiting to be deleted.
#[derive(Debug, Clone, Copy)]
struct Mat([f64; 6]);

impl Mat {
    const IDENTITY: Self = Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    fn apply(self, (x, y): (f64, f64)) -> (f64, f64) {
        let [a, b, c, d, e, f] = self.0;
        (a * x + c * y + e, b * x + d * y + f)
    }

    /// The rotation this matrix expresses, in degrees anticlockwise in
    /// `[0, 360)`, or `None` when it is not a rotation-with-uniform-scale.
    ///
    /// # The test, and why each half of it is there
    ///
    /// A rotation by θ with uniform positive scale s is
    /// `[s·cosθ, s·sinθ, −s·sinθ, s·cosθ, e, f]`. So:
    ///
    /// * `a ≈ d` **and** `b ≈ −c` — this is what excludes a shear, an
    ///   anisotropic scale, and a mirror. A mirror in x is
    ///   `[−s·cosθ, …, …, s·cosθ]`, which fails `a ≈ d`; a mirror in y fails it
    ///   too. Both are legal `/Matrix` values that no angle describes.
    /// * `s > 0` — a degenerate (collapsed) matrix has no orientation, and
    ///   `atan2(0, 0)` is `0.0`, which would be a confidently wrong answer.
    ///
    /// The comparison is **relative to the scale**, not absolute: a stamp whose
    /// `/Matrix` carries a scale of 300 would fail a fixed epsilon on rounding
    /// alone, and one with a scale of 0.001 would pass it while being visibly
    /// sheared.
    fn rotation_degrees(self) -> Option<f64> {
        let [a, b, c, d, _, _] = self.0;
        let scale = a.hypot(b);
        if !scale.is_finite() || scale <= 1e-9 {
            return None;
        }
        let tolerance = scale * 1e-4;
        if (a - d).abs() > tolerance || (b + c).abs() > tolerance {
            return None;
        }
        let degrees = b.atan2(a).to_degrees();
        Some(degrees.rem_euclid(360.0))
    }
}

/// The four numbers at `key`, resolved through the graph, or `None`.
///
/// Rejects a short array outright rather than padding: a `/BBox` with three
/// entries is malformed, and inventing a fourth would produce a plausible
/// outline around artwork placed somewhere else entirely.
fn numbers<G: ObjectGraph + ?Sized, const N: usize>(
    graph: &G,
    dict: &Dict,
    key: &[u8],
) -> Option<[f64; N]> {
    let array = graph.resolve(dict.get(key)?).as_array()?;
    if array.len() < N {
        return None;
    }
    let mut out = [0.0_f64; N];
    for (slot, item) in out.iter_mut().zip(array.iter()) {
        *slot = graph.resolve(item).as_number()?;
        if !slot.is_finite() {
            return None;
        }
    }
    Some(out)
}

/// The **normal** appearance stream's dictionary for this annotation, applying
/// §12.5.5's `/AS` selection rule.
///
/// # The three shapes `/AP` `/N` can take, and what each means
///
/// | `/N` resolves to | meaning | handled |
/// |---|---|---|
/// | a **stream** | the ordinary case: one appearance | taken |
/// | a **subdictionary** with an `/AS` naming one of its entries | an appearance *state* (a check box's `/Off` and `/Yes`) | the named entry is taken |
/// | a **subdictionary** with no usable `/AS` | §12.5.5 NOTE 3 — *"reasonable behaviour such as displaying nothing"* | **`None`** |
///
/// ★ The last row matters and is the reason this is a function rather than two
/// `?` operators. `pdfcer-core` models it as a distinct `Appearance::
/// StateUnresolved` and **refuses to guess** a first/`On`/`Off` key, because
/// real readers disagree about which to pick and guessing shows a state no
/// other viewer shows. This shell must not guess either: an outline drawn from
/// an appearance the renderer declined to paint would be a box around nothing.
///
/// **One exception, and it matches the engine's:** a subdictionary with exactly
/// one entry and no `/AS` is not ambiguous — there is nothing to choose between
/// — so it is taken. The engine's own wording scopes `StateUnresolved` to *"`/AS`
/// is missing against a **multi-entry** subdictionary"*.
fn normal_appearance<'a, G: ObjectGraph + ?Sized>(
    graph: &'a G,
    annot: &'a Dict,
) -> Option<&'a Dict> {
    let ap = graph.resolve(annot.get(b"AP")?).as_dict()?;
    let n = graph.resolve(ap.get(b"N")?);
    if matches!(n, Object::Stream(_)) {
        return n.as_dict();
    }
    let states = n.as_dict()?;
    let selected = match annot.get(b"AS").map(|o| graph.resolve(o)) {
        Some(Object::Name(name)) => states.get(name.as_bytes())?,
        // Exactly one entry and no /AS: nothing to choose between.
        _ if states.len() == 1 => states.iter().next()?.1,
        _ => return None,
    };
    let stream = graph.resolve(selected);
    matches!(stream, Object::Stream(_))
        .then(|| stream.as_dict())
        .flatten()
}

/// **Where `annot_id`'s artwork actually sits**, or `None` when the question has
/// no honest answer for this annotation.
///
/// `None` is returned — and it is a *correct* answer, not a failure — when:
///
/// * the object is not an annotation dictionary, or has no `/Rect`;
/// * there is no usable normal appearance stream (no `/AP`, a dangling `/N`, or
///   an unresolvable appearance state — see [`normal_appearance`]);
/// * the appearance has no `/BBox`, or a `/BBox` that its own `/Matrix`
///   collapses to a sliver, which makes §12.5.5's step-(c) fit matrix singular
///   and there is no honest placement. `pdfcer-render` refuses the same case by
///   the same test and counts it as `annotations_placement_degenerate`.
///
/// In every one of those cases the caller should keep using `/Rect`, which for
/// an annotation with no appearance stream **is** where the mark is.
///
/// # Cost
///
/// Two dictionary reads and sixteen multiplications. It is called on selection
/// and on paint, not per frame per annotation, and it allocates nothing.
#[must_use]
pub fn oriented<G: ObjectGraph + ?Sized>(graph: &G, annot_id: ObjId) -> Option<OrientedBox> {
    let annot = graph.resolved(annot_id).as_dict()?;
    let [rllx, rlly, rurx, rury] = numbers::<_, 4>(graph, annot, b"Rect")?;
    // §7.9.5: a rectangle's corners may be given in either order, and the
    // reader normalises. `page_tree::Rect::from_corners` does this for the
    // engine's own reads; doing it here too is what keeps an annotation written
    // upper-left-first from producing an inside-out outline.
    let (rect_x0, rect_x1) = (rllx.min(rurx), rllx.max(rurx));
    let (rect_y0, rect_y1) = (rlly.min(rury), rlly.max(rury));

    let appearance = normal_appearance(graph, annot)?;
    let [bllx, blly, burx, bury] = numbers::<_, 4>(graph, appearance, b"BBox")?;
    let matrix = numbers::<_, 6>(graph, appearance, b"Matrix").map_or(Mat::IDENTITY, Mat);

    // Step (a): the /BBox corners through the appearance's own /Matrix. The
    // result is *"a quadrilateral with arbitrary orientation"* — §12.5.5's own
    // words, and the whole reason this module can answer the operator's
    // question at all.
    let quad = [
        matrix.apply((bllx, blly)),
        matrix.apply((burx, blly)),
        matrix.apply((burx, bury)),
        matrix.apply((bllx, bury)),
    ];
    if quad.iter().any(|(x, y)| !x.is_finite() || !y.is_finite()) {
        return None;
    }

    // Step (b): its upright bounding box.
    let (mut lo_x, mut lo_y) = (f64::INFINITY, f64::INFINITY);
    let (mut hi_x, mut hi_y) = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    for &(x, y) in &quad {
        lo_x = lo_x.min(x);
        lo_y = lo_y.min(y);
        hi_x = hi_x.max(x);
        hi_y = hi_y.max(y);
    }
    let (span_x, span_y) = (hi_x - lo_x, hi_y - lo_y);
    // The same degeneracy floor `pdfcer-render::annot` applies, and for the
    // same reason: below it the step-(c) division is by (near) zero and `A` is
    // singular, so there is no placement to report — not a placement of zero.
    if span_x <= 1e-6 || span_y <= 1e-6 {
        return None;
    }

    // Step (c): the scale-and-translate that makes that box fit /Rect exactly.
    let sx = (rect_x1 - rect_x0) / span_x;
    let sy = (rect_y1 - rect_y0) / span_y;
    let mut corners = [(0.0, 0.0); 4];
    for (slot, &(x, y)) in corners.iter_mut().zip(quad.iter()) {
        *slot = (rect_x0 + (x - lo_x) * sx, rect_y0 + (y - lo_y) * sy);
    }

    let degrees = matrix.rotation_degrees();

    Some(OrientedBox { corners, degrees })
}

#[cfg(test)]
mod tests;
