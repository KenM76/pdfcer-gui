//! # `provider::geometry` — **the same questions, asked of either index space**
//!
//! `OPERATOR_REQUESTS.md` **O70**, 2026-09-01. Everything here answers *"what
//! is inside this thing?"* for a [`TargetId`] rather than for a page
//! paint-order index, which is what lets the Part and Node rungs be offered for
//! something painted inside a form XObject.
//!
//! ## ★★ Why a module rather than more methods in [`super`]
//!
//! R2's gate, and it points at a real seam. `super` answers *"what is on this
//! page and where is it?"* — the decomposition, the hit tests, the canvas
//! projection, the panel's tree. This answers the narrower question of what one
//! already-identified object is made of, and it does so for **both** address
//! spaces, which is a distinction none of the rest of that file has to make.
//!
//! ## Why these are additions rather than changed signatures
//!
//! The index-based methods in `super` have thirty-odd call sites and every one
//! is correct: `canvas::input`, `canvas::painting`, `canvas::shapes` and the
//! Objects panel all hold page indices for good reasons. Changing them in one
//! edit is a diff nobody can review against a hot path this project has already
//! broken twice by second-guessing.
//!
//! ⇒ So this is the general form, the index form delegates to it, and call
//! sites move over one at a time behind their own tests. The duplication is a
//! `TargetId::Object(i)` wrapper, not a second implementation — the distinction
//! `canvas::mapping`'s header draws between a shared helper and a second
//! opinion.
//!
//! ## ★★ The engine already published the leaf-friendly forms
//!
//! `hit_test_subpaths_of(&PathObject, …)` takes the object rather than a model
//! and an index, and `PathObject::page_subpaths` is geometry in page space
//! whoever holds it. So none of this needed a request — the shell had been
//! asking the narrower question because the narrower question was all it had
//! ever needed.
//!
//! Proven against a real file rather than a stub, in
//! `crates/pdfcer-gui/tests/leaf_geometry.rs`: a stub carries rectangles and
//! cannot model where a path's anchors are, so a unit test against one would
//! assert that the plumbing returns what the stub was given.

use egui::{Pos2, Rect};
use pdfcer_core::vector::{Point, VectorObject};

use super::{ObjectModelProvider, PartKind, TargetId, resolve};

impl ObjectModelProvider {
    // ===================================================================
    // ★★★ THE SAME FOUR QUESTIONS, ASKED OF EITHER INDEX SPACE
    // ===================================================================
    //
    // `OPERATOR_REQUESTS.md` O70, 2026-09-01. Everything below this line
    // answers *"what is inside this thing?"* for a `TargetId` rather than for
    // a page paint-order index, which is what lets the Part and Node rungs be
    // offered for something painted inside a form XObject.
    //
    // ## Why they are additions rather than changed signatures
    //
    // The index-based methods have thirty-odd call sites and every one of them
    // is correct: `canvas::input`, `canvas::painting`, `canvas::shapes` and the
    // Objects panel all hold page indices for good reasons. Changing them all
    // in one edit is a diff nobody can review against a hot path this project
    // has already broken twice by second-guessing it.
    //
    // ⇒ So the `_of` family is the general form, the index form delegates to
    // it, and call sites move over one at a time behind their own tests. The
    // duplication is a `TargetId::Object(i)` wrapper, not a second
    // implementation — which is the distinction `mapping`'s header draws
    // between a shared helper and a second opinion.
    //
    // ## ★★ The engine already published the leaf-friendly forms
    //
    // `hit_test_subpaths_of(&PathObject, …)` takes the object rather than a
    // model and an index, and `PathObject::page_subpaths` is geometry in page
    // space whoever holds it. So none of this needed a request — the shell was
    // asking the narrower question because the narrower question was all it
    // had ever needed.

    /// **The decomposed object a target names**, from either list.
    ///
    /// ★ `None` for an index the page does not have, which is the contract
    /// every accessor here inherits: a selection can outlive an edit that
    /// removed what it named, and the honest answer is to drop the entry
    /// rather than to panic on the frame that is trying to draw it.
    #[must_use]
    pub fn object_for(&self, target: TargetId) -> Option<&VectorObject> {
        match target {
            TargetId::Object(i) => self.objects.objects.get(usize::try_from(i).ok()?),
            TargetId::Leaf(i) => self
                .objects
                .leaves
                .get(usize::try_from(i).ok()?)
                .map(|leaf| &leaf.object),
        }
    }

    /// [`Self::part_kind`], for either index space.
    #[must_use]
    pub fn part_kind_of(&self, target: TargetId) -> Option<PartKind> {
        match self.object_for(target) {
            Some(VectorObject::Path(_)) => Some(PartKind::Subpath),
            Some(VectorObject::Text(_)) => Some(PartKind::Run),
            _ => None,
        }
    }

    /// [`Self::subpath_hits`], for either index space.
    ///
    /// ★ Through `hit_test_subpaths_of` rather than `hit_test_subpaths`: the
    /// first takes the path, the second takes the model and an index into its
    /// page list. The engine published both, and the object-taking form is the
    /// one a leaf can use — the geometry is identical, only the addressing
    /// differs.
    #[must_use]
    pub fn subpath_hits_of(&self, target: TargetId, point: Pos2, tolerance: f64) -> Vec<usize> {
        let Some(VectorObject::Path(path)) = self.object_for(target) else {
            return Vec::new();
        };
        let Some(pdf) = self.canvas_to_pdf(point) else {
            return Vec::new();
        };
        // ★ `vector::hit::` rather than `vector::`: the object-taking form is
        // `pub` in its own module and is NOT in `vector`'s re-export list,
        // where its index-taking sibling is. The module itself is `pub mod`,
        // so the path is public and this is a spelling rather than a
        // limitation — worth naming so the next reader does not conclude the
        // function is private and write a second one here.
        pdfcer_core::vector::hit::hit_test_subpaths_of(path, pdf, resolve(tolerance))
    }

    /// [`Self::subpath_node_points`], for either index space.
    ///
    /// The indices are **object-scoped** — the space `vector::anchor_count`
    /// reports and the node verbs take — and they are computed by the same
    /// running-offset walk the page-object form uses, because the two must
    /// agree about what anchor 7 is or a drag would move a different point
    /// from the one drawn.
    #[must_use]
    pub fn subpath_node_points_of(&self, target: TargetId, subpath: usize) -> Vec<(usize, Point)> {
        let Some(VectorObject::Path(path)) = self.object_for(target) else {
            return Vec::new();
        };
        let subpaths = path.page_subpaths();
        let mut offset = 0usize;
        for (i, sp) in subpaths.iter().enumerate() {
            let anchors: Vec<Point> = sp.anchors().collect();
            if i == subpath {
                return anchors
                    .into_iter()
                    .enumerate()
                    .map(|(n, p)| (offset + n, p))
                    .collect();
            }
            offset += anchors.len();
        }
        Vec::new()
    }

    /// [`Self::object_node_points`], for either index space.
    #[must_use]
    pub fn object_node_points_of(&self, target: TargetId) -> Vec<(usize, Point)> {
        let Some(VectorObject::Path(path)) = self.object_for(target) else {
            return Vec::new();
        };
        path.page_subpaths()
            .iter()
            .flat_map(|sp| sp.anchors())
            .enumerate()
            .collect()
    }

    /// [`Self::nearest_node`], for either index space.
    ///
    /// Ties resolve to the lower index, exactly as the page-object form does —
    /// the rule is restated in one place by delegating the point list, so the
    /// two cannot answer differently for the same geometry.
    #[must_use]
    pub fn nearest_node_of(
        &self,
        target: TargetId,
        subpath: usize,
        point: Pos2,
        tolerance: f64,
    ) -> Option<usize> {
        let pdf = self.canvas_to_pdf(point)?;
        let mut best: Option<(usize, f64)> = None;
        for (index, p) in self.subpath_node_points_of(target, subpath) {
            if !p.is_finite() {
                continue;
            }
            let d = p.distance(pdf);
            if d <= tolerance && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((index, d));
            }
        }
        best.map(|(index, _)| index)
    }

    /// [`Self::node_handles`], for either index space.
    ///
    /// The Bézier control points of one anchor: the incoming one is the second
    /// control of the segment **before** it, the outgoing one the first control
    /// of the segment **after**. Only cubics have them, so a polyline answers
    /// empty and no handle is drawn — which is the honest answer rather than a
    /// grab target for a gesture with nothing to move.
    ///
    /// ★ The object-scoped anchor index is brought back into the subpath's own
    /// space with the SAME running offset [`Self::subpath_node_points_of`]
    /// computes. Two walks that disagreed about which subpath anchor 7 falls in
    /// would draw a handle on one curve and move another.
    #[must_use]
    pub fn node_handles_of(
        &self,
        target: TargetId,
        subpath: usize,
        node: usize,
    ) -> Vec<(pdfcer_core::vector::Handle, Point)> {
        use pdfcer_core::vector::{Handle, Segment};

        let Some(VectorObject::Path(path)) = self.object_for(target) else {
            return Vec::new();
        };
        let subpaths = path.page_subpaths();
        let mut offset = 0usize;
        for (i, sp) in subpaths.iter().enumerate() {
            let count = sp.anchors().count();
            if i == subpath {
                // The anchor is not in this subpath: a selection that out-ran a
                // decomposition, refused rather than guessed at — the posture
                // `canvas::moving`'s `NodeNotFound` takes.
                let Some(local) = node.checked_sub(offset).filter(|k| *k < count) else {
                    return Vec::new();
                };
                let mut out = Vec::with_capacity(2);
                if let Some(Segment::Cubic { c2, .. }) =
                    local.checked_sub(1).and_then(|j| sp.segments.get(j))
                {
                    out.push((Handle::Incoming, *c2));
                }
                if let Some(Segment::Cubic { c1, .. }) = sp.segments.get(local) {
                    out.push((Handle::Outgoing, *c1));
                }
                return out;
            }
            offset += count;
        }
        Vec::new()
    }

    /// [`Self::subpath_bounds_canvas`], for either index space.
    ///
    /// ★ Computed from the subpath's own anchors rather than through
    /// `vector::subpath_bounds`, which takes a model and a page index. Anchors
    /// alone under-report a curve whose control points bow outside them — the
    /// same approximation the page-object form inherits from the engine on a
    /// Bézier, and the box is a selection outline rather than a measurement.
    /// `None` for an empty subpath, which has no box to draw.
    #[must_use]
    pub fn subpath_bounds_canvas_of(&self, target: TargetId, subpath: usize) -> Option<Rect> {
        let points = self.subpath_node_points_of(target, subpath);
        let mut bounds: Option<pdfcer_core::vector::Bounds> = None;
        for (_, p) in points {
            if !p.is_finite() {
                continue;
            }
            bounds = Some(match bounds {
                None => pdfcer_core::vector::Bounds { min: p, max: p },
                Some(b) => pdfcer_core::vector::Bounds {
                    min: Point {
                        x: b.min.x.min(p.x),
                        y: b.min.y.min(p.y),
                    },
                    max: Point {
                        x: b.max.x.max(p.x),
                        y: b.max.y.max(p.y),
                    },
                },
            });
        }
        self.pdf_bounds_to_canvas(bounds?)
    }
}
