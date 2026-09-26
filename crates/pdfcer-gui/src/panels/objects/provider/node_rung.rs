//! # `provider::node_rung` — the Point rung's pick sets: anchors and handles
//!
//! Everything this shell knows about **the points a path is made of**: which
//! anchors belong to which subpath, what number each one answers to, which
//! Bézier control points shape the curve on either side of it, and which of
//! those a press lands on. It is the third rung of the selection ladder —
//! *object → part → point* — answered for the page's own paint order.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/objects/provider/node_rung.md`.

use egui::Pos2;
use pdfcer_core::vector::{Handle, Point, Segment, VectorObject};

use super::ObjectModelProvider;

impl ObjectModelProvider {
    /// The anchors of ONE subpath, each paired with its **object-scoped**
    /// index — the Point rung's pick set (decision 028 §Q1).
    ///
    /// # Why not [`Self::object_sample_points`], which already returns anchors
    ///
    /// That one returns the whole object's flat list, and using it as a node
    /// pick set is a hazard decision 028 found already shipped: on a measured
    /// CAD export one path object holds **6,681 anchors**, so "the nearest
    /// anchor to the press" can easily belong to a subpath the operator is
    /// not pointing at, and nothing is drawn beforehand to say which. Scoping
    /// the pick set to the ENTERED subpath is what makes the grab predictable
    /// — the operator can only hit points they descended into and can see.
    ///
    /// The same number is why the Objects panel nests points under a *part*
    /// rather than listing an object's anchors directly: 6,681 sibling rows
    /// under one object is not a tree, it is a wall.
    ///
    /// # Why the index is object-scoped even though the set is subpath-scoped
    ///
    /// Decision 025 §1.3(b): the number pdfcer shows and the number
    /// `pdfcer node-move --node N` addresses must be the same number.
    /// `vector::anchor_count` counts across the whole object, so the running
    /// offset is added here rather than letting the GUI invent a second
    /// numbering that would disagree with every other consumer.
    ///
    /// Returns empty for a non-path object or an out-of-range index — the
    /// same exclusion [`Self::object_sample_points`] applies, for the same
    /// reason (text and image objects are not node-editable, decision 011
    /// §2.1).
    #[must_use]
    /// The **Bézier handles** of one anchor of one subpath, in PDF user space.
    pub fn node_handles(
        &self,
        index: usize,
        subpath: usize,
        node: usize,
    ) -> Vec<(pdfcer_core::vector::Handle, Point)> {
        use pdfcer_core::vector::{Handle, Segment};

        let Some(VectorObject::Path(path)) = self.objects.objects.get(index) else {
            return Vec::new();
        };
        let subpaths = path.page_subpaths();
        // The object-scoped anchor index has to be brought back into the
        // subpath's own space, using the SAME running offset
        // `subpath_node_points` computes — see its comment for why the offset
        // is the object-scoped index of the subpath's first anchor.
        let mut offset = 0usize;
        for (i, sp) in subpaths.iter().enumerate() {
            let count = sp.anchors().count();
            if i == subpath {
                let Some(local) = node.checked_sub(offset).filter(|k| *k < count) else {
                    // The anchor is not in this subpath. A selection that
                    // out-ran a decomposition, refused rather than guessed at —
                    // the same posture `canvas::moving`'s `NodeNotFound` takes.
                    return Vec::new();
                };
                let mut out = Vec::with_capacity(2);
                // Incoming: the second control point of the segment BEFORE it.
                if let Some(Segment::Cubic { c2, .. }) =
                    local.checked_sub(1).and_then(|j| sp.segments.get(j))
                {
                    out.push((Handle::Incoming, *c2));
                }
                // Outgoing: the first control point of the segment AFTER it.
                if let Some(Segment::Cubic { c1, .. }) = sp.segments.get(local) {
                    out.push((Handle::Outgoing, *c1));
                }
                return out;
            }
            offset += count;
        }
        Vec::new()
    }

    pub fn subpath_node_points(&self, index: usize, subpath: usize) -> Vec<(usize, Point)> {
        let Some(VectorObject::Path(path)) = self.objects.objects.get(index) else {
            return Vec::new();
        };
        let subpaths = path.page_subpaths();
        // The running offset IS the object-scoped index of the target
        // subpath's first anchor, because `anchor_count` flattens the same
        // walk in the same order.
        let mut offset = 0usize;
        for (i, sp) in subpaths.iter().enumerate() {
            let anchors: Vec<Point> = sp.anchors().collect();
            if i == subpath {
                return anchors
                    .into_iter()
                    .enumerate()
                    .map(|(k, p)| (offset + k, p))
                    .collect();
            }
            offset += anchors.len();
        }
        Vec::new()
    }

    /// **Every** anchor of the path object at paint-order `index`, each with
    /// its object-scoped index — [`Self::subpath_node_points`] flattened
    /// across all subpaths.
    #[must_use]
    pub fn object_node_points(&self, index: usize) -> Vec<(usize, Point)> {
        let Some(VectorObject::Path(path)) = self.objects.objects.get(index) else {
            return Vec::new();
        };
        path.page_subpaths()
            .iter()
            .flat_map(|sp| sp.anchors())
            .enumerate()
            .collect()
    }

    /// The Bézier control points ("handles") of one subpath, each tagged with
    /// the **object-scoped index of the node it belongs to** and which side
    /// of that node it shapes.
    #[must_use]
    pub fn subpath_handle_points(
        &self,
        object: usize,
        subpath: usize,
    ) -> Vec<(usize, Handle, Point)> {
        let Some(VectorObject::Path(path)) = self.objects.objects.get(object) else {
            return Vec::new();
        };
        let subpaths = path.page_subpaths();
        let mut offset = 0usize;
        for (i, sp) in subpaths.iter().enumerate() {
            let anchors = sp.anchors().count();
            if i != subpath {
                offset += anchors;
                continue;
            }
            let mut out = Vec::new();
            for (k, seg) in sp.segments.iter().enumerate() {
                if let Segment::Cubic { c1, c2, .. } = *seg {
                    // `c1` shapes the curve leaving anchor k …
                    out.push((offset + k, Handle::Outgoing, c1));
                    // … and `c2` shapes the curve arriving at anchor k+1.
                    out.push((offset + k + 1, Handle::Incoming, c2));
                }
            }
            return out;
        }
        Vec::new()
    }

    /// The handle of `subpath` nearest `point` within `tolerance`, as
    /// `(node index, side)` — the Point rung's handle pick.
    #[must_use]
    pub fn nearest_handle(
        &self,
        object: usize,
        subpath: usize,
        pdf: Point,
        tolerance: f64,
    ) -> Option<(usize, Handle)> {
        let mut best: Option<((usize, Handle), f64)> = None;
        for (index, side, p) in self.subpath_handle_points(object, subpath) {
            if !p.is_finite() {
                continue;
            }
            let d = p.distance(pdf);
            if d <= tolerance && best.is_none_or(|(_, bd)| d < bd) {
                best = Some(((index, side), d));
            }
        }
        best.map(|(hit, _)| hit)
    }

    /// The object-scoped index of the anchor of `subpath` nearest `point`
    /// within `tolerance`, or `None` — the Point rung's pick.
    #[must_use]
    pub fn nearest_node(
        &self,
        object: usize,
        subpath: usize,
        point: Pos2,
        tolerance: f64,
    ) -> Option<usize> {
        let pdf = self.canvas_to_pdf(point)?;
        let mut best: Option<(usize, f64)> = None;
        for (index, p) in self.subpath_node_points(object, subpath) {
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
}
