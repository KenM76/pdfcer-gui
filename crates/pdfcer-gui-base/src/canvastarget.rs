//! # `canvastarget` — the seam a hit-testable content model plugs into
//!
//! The canvas selects *things*. It does not know what a thing is, how it was
//! decomposed, or what coordinate frame its geometry was authored in. All it
//! needs is: **what is under this point, what is inside this rect, and where
//! is the thing I already have?** That question set is
//! [`CanvasTargetProvider`], and everything in `canvas/` is written against
//! it rather than against `pdfcer-core`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/canvastarget.md`.

use egui::{Pos2, Rect};
use pdfcer_core::vector::{FormMarquee, MarqueeMode};

use crate::objectprovider::ObjectModelProvider;
pub use crate::objectprovider::TargetId;
use crate::pick::PickClass;

/// The seam a hit-testable content model plugs into.
pub trait CanvasTargetProvider {
    /// **Every** target at a canvas-space `point` within `tolerance`,
    /// **front-most first**.
    fn hit_test_all(&self, page_index: usize, point: Pos2, tolerance: f64) -> Vec<TargetId>;

    /// The **front-most** target at a canvas-space `point`, or `None`.
    fn hit_test(&self, page_index: usize, point: Pos2, tolerance: f64) -> Option<TargetId> {
        self.hit_test_all(page_index, point, tolerance)
            .into_iter()
            .next()
    }

    /// Which **class** a target belongs to, for the operator's selection
    /// filter — or `None` for a target this provider no longer knows.
    fn object_class(&self, page_index: usize, target: TargetId) -> Option<PickClass> {
        let _ = (page_index, target);
        None
    }

    /// **The outermost form XObject a target is painted inside**, as a page
    /// object, or `None` for a target that is not inside one.
    fn containing_form(&self, page_index: usize, target: TargetId) -> Option<TargetId> {
        let _ = (page_index, target);
        None
    }

    /// **Is this container worth selecting instead of what is inside it?**
    fn container_is_worth_selecting(&self, page_index: usize, container: TargetId) -> bool {
        let _ = (page_index, container);
        true
    }

    /// Every target a marquee rect takes, under `mode` and `forms`.
    fn hit_test_rect(
        &self,
        page_index: usize,
        rect: Rect,
        mode: MarqueeMode,
        forms: FormMarquee,
    ) -> Vec<TargetId>;

    /// One target's canvas-space bounding rect, or `None` for a target this
    /// provider no longer knows.
    fn bounds(&self, page_index: usize, target: TargetId) -> Option<Rect>;

    /// Which **part** of `object` a canvas-space click lands on — subpaths
    /// for a path object, show-operator runs for a text object — nearest
    /// first.
    fn page_objects_model(&self, page_index: usize) -> Option<&pdfcer_core::vector::PageObjects> {
        let _ = page_index;
        None
    }

    /// One object's page-space anchor samples — **the circular measure tool's
    /// fit input**, and the only query on this trait whose result is not
    /// canvas space.
    ///
    /// **PDF user space, deliberately**, where every other geometric value
    /// crossing this trait is canvas space. The reason is the same one that
    /// keeps the two-line pick going through
    /// [`Self::page_objects_model`]: these points are handed straight to
    /// [`fit_circle_taubin`](pdfcer_core::dimension::fit_circle_taubin), which
    /// is the **engine's** fit, and the engine works in PDF user space. A
    /// canvas-space sample set would have to be converted back before the fit,
    /// which is a second Y-flip in a shell whose header says there is exactly
    /// one. The provider already stores its `PageObjects` in that frame, so
    /// this is a read rather than a conversion.
    ///
    /// Empty is a real answer and the common one: a text, image or form object
    /// carries no anchors — the same exclusion the snap engine applies — and so
    /// does a query about a page this provider does not serve or an index it
    /// does not have. A caller that must distinguish those is asking the wrong
    /// object; the *canvas* knows which page it drew.
    ///
    /// **A provided method returning nothing**, so a future provider over
    /// annotations or placed dimensions does not have to invent anchors for
    /// objects that have none in order to compile.
    fn object_sample_points(
        &self,
        page_index: usize,
        index: usize,
    ) -> Vec<pdfcer_core::vector::Point> {
        let _ = (page_index, index);
        Vec::new()
    }

    // ===================================================================
    // THE SAME THREE QUESTIONS, FOR EITHER INDEX SPACE
    // ===================================================================
    //
    // `OPERATOR_REQUESTS.md` O70, 2026-09-01. The three above take a page
    // paint-order index, which is the only address the Part and Node rungs
    // have ever had — so those rungs were structurally unavailable for
    // anything painted inside a form XObject. `canvas::input::probe` said so
    // in a comment: *"the ladder stopping at the Object rung for a leaf,
    // expressed where the address space runs out."*
    //
    // These take a `TargetId` and are the ones `probe` now asks. The
    // page-index forms stay for the callers that legitimately hold one, and
    // the default implementations here delegate to them so a test double that
    // has no leaves is unaffected.

    /// [`Self::part_hits`], for either index space.
    fn part_hits_of(
        &self,
        page_index: usize,
        target: TargetId,
        point: Pos2,
        tolerance: f64,
    ) -> Vec<usize> {
        match target.page_object_index() {
            Some(object) => self.part_hits(page_index, object, point, tolerance),
            // A double that does not model leaves answers "no parts" rather
            // than pretending — the same shape `object_class` uses for "I
            // cannot say", and the honest answer for a provider with one list.
            None => Vec::new(),
        }
    }

    /// [`Self::part_bounds`], for either index space.
    fn part_bounds_of(&self, page_index: usize, target: TargetId, part: usize) -> Option<Rect> {
        self.part_bounds(page_index, target.page_object_index()?, part)
    }

    /// [`Self::nearest_node`], for either index space.
    fn nearest_node_of(
        &self,
        page_index: usize,
        target: TargetId,
        part: usize,
        point: Pos2,
        tolerance: f64,
    ) -> Option<usize> {
        self.nearest_node(
            page_index,
            target.page_object_index()?,
            part,
            point,
            tolerance,
        )
    }

    fn part_hits(
        &self,
        page_index: usize,
        object: usize,
        point: Pos2,
        tolerance: f64,
    ) -> Vec<usize>;

    /// A part's own canvas-space bounds, for its outline.
    fn part_bounds(&self, page_index: usize, object: usize, part: usize) -> Option<Rect>;

    /// The **object-scoped** index of the anchor of `part` nearest a
    /// canvas-space `point` within `tolerance` — the Node rung's pick.
    fn nearest_node(
        &self,
        page_index: usize,
        object: usize,
        part: usize,
        point: Pos2,
        tolerance: f64,
    ) -> Option<usize>;
}

/// **How much of a page's content a container may cover and still be worth
/// selecting**, as a fraction of the union of every page object's bounds.
const COVERS_EVERYTHING: f32 = 0.9;

/// **The re-attachment.**
impl CanvasTargetProvider for ObjectModelProvider {
    /// Measured, not assumed. See the trait for why the question exists.
    fn container_is_worth_selecting(&self, page_index: usize, container: TargetId) -> bool {
        let Some(bounds) = self.bounds(page_index, container) else {
            return true;
        };
        // AGAINST THE PAGE — and the first version of this compared against
        // the union of every page object's bounds instead.
        //
        // That was wrong in a way only a SECOND fixture could show: when the
        // form is the only page object, the union IS the form, the ratio is
        // 1.0, and every lone container is judged "holds everything" — a stamp
        // on an otherwise empty sheet included.
        //
        // Caught by two of this project's own driven checks contradicting
        // each other within the hour, which is the most useful thing a suite
        // can do. One demanded the leaf on a page-sized wrapper; the other
        // demanded the container on a 320×220 form on a 400×300 page. Both are
        // right, and only a page-relative measure satisfies both.
        let Some(page) = self.page_extent() else {
            return true;
        };
        if page.x <= f32::EPSILON || page.y <= f32::EPSILON {
            return true;
        }
        let covers = (bounds.width() / page.x).min(1.0) * (bounds.height() / page.y).min(1.0);
        let worth = covers < COVERS_EVERYTHING;
        // Traced, because the alternative is inferring this from a selection
        // two layers away. When those two checks disagreed, neither could say
        // what the predicate had actually answered — the numbers had to be
        // reconstructed by hand from a fixture generator. One line ends that.
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "container-worth id={} covers={covers:.3} page={:.0}x{:.0} box={:.0}x{:.0} worth={worth}",
                container.raw(),
                page.x,
                page.y,
                bounds.width(),
                bounds.height()
            )
        });
        worth
    }

    /// The real model. Guarded on the page, because this provider decomposes
    /// exactly one page and answering for another would hand the measure tool
    /// geometry from a different sheet.
    fn page_objects_model(&self, page_index: usize) -> Option<&pdfcer_core::vector::PageObjects> {
        (page_index == self.page_index()).then(|| self.page_objects())
    }

    fn hit_test_all(&self, page_index: usize, point: Pos2, tolerance: f64) -> Vec<TargetId> {
        Self::hit_test_all(self, page_index, point, tolerance)
    }

    /// The real samples, guarded on the page for the reason the block's own
    /// docs give: the provider decomposes exactly one page, and answering for
    /// another would fit a circle to a different sheet's geometry.
    fn object_sample_points(
        &self,
        page_index: usize,
        index: usize,
    ) -> Vec<pdfcer_core::vector::Point> {
        if page_index != self.page_index() {
            return Vec::new();
        }
        Self::object_sample_points(self, index)
    }

    fn hit_test_rect(
        &self,
        page_index: usize,
        rect: Rect,
        mode: MarqueeMode,
        forms: FormMarquee,
    ) -> Vec<TargetId> {
        Self::hit_test_rect(self, page_index, rect, mode, forms)
    }

    /// `Self::containing_form`, spelled as the inherent call rather than as
    /// `self.containing_form(..)`, which would be ambiguous to a reader for
    /// the same reason the four lines above are spelled this way: the
    /// provider has an inherent method of that name and this is the trait's.
    /// Rust resolves the bare form to the inherent one, so it happens to be
    /// correct and reads as a recursion.
    fn containing_form(&self, page_index: usize, target: TargetId) -> Option<TargetId> {
        Self::containing_form(self, page_index, target)
    }

    // The three `_of` overrides, which is where a leaf's Part and Node rungs
    // actually come from — the trait's defaults answer only for a page object.
    // `provider::geometry` holds the implementations and its header carries why
    // none of it needed an engine request.
    fn part_hits_of(
        &self,
        page_index: usize,
        target: TargetId,
        point: Pos2,
        tolerance: f64,
    ) -> Vec<usize> {
        if page_index != self.page_index() {
            return Vec::new();
        }
        use crate::objectprovider::PartKind;
        // Each part kind has its own hit test, for a page object and a leaf alike.
        match self.part_kind_of(target) {
            Some(PartKind::Subpath) => self.subpath_hits_of(target, point, tolerance),
            Some(PartKind::TextLine) => self.text_line_hits_of(target, point, tolerance),
            None => Vec::new(),
        }
    }

    // Dispatches on the part kind, as `part_hits_of` does.
    fn part_bounds_of(&self, page_index: usize, target: TargetId, part: usize) -> Option<Rect> {
        if page_index != self.page_index() {
            return None;
        }
        use crate::objectprovider::PartKind;
        match self.part_kind_of(target)? {
            PartKind::Subpath => self.subpath_bounds_canvas_of(target, part),
            PartKind::TextLine => self.text_line_bounds_canvas_of(target, part),
        }
    }

    fn nearest_node_of(
        &self,
        page_index: usize,
        target: TargetId,
        part: usize,
        point: Pos2,
        tolerance: f64,
    ) -> Option<usize> {
        (page_index == self.page_index())
            .then(|| Self::nearest_node_of(self, target, part, point, tolerance))
            .flatten()
    }

    /// The real classifier, guarded on the page for the same reason every
    /// other query here is: this provider decomposes exactly one page.
    fn object_class(&self, page_index: usize, target: TargetId) -> Option<PickClass> {
        if page_index != self.page_index() {
            return None;
        }
        // Both lists. The classifier is `object_kind`, unchanged and
        // still the only one in this crate — a leaf holds a `VectorObject`
        // exactly like a page object does, so the same hop answers for it and
        // the operator's pick filter works identically inside a form.
        let model = self.page_objects();
        let object = match target {
            TargetId::Object(i) => model.objects.get(usize::try_from(i).ok()?)?,
            TargetId::Leaf(i) => &model.leaves.get(usize::try_from(i).ok()?)?.object,
        };
        Some(PickClass::of_object(crate::objectsummary::object_kind(
            object,
        )))
    }

    fn bounds(&self, page_index: usize, target: TargetId) -> Option<Rect> {
        Self::bounds(self, page_index, target)
    }

    fn part_hits(
        &self,
        page_index: usize,
        object: usize,
        point: Pos2,
        tolerance: f64,
    ) -> Vec<usize> {
        if page_index != self.page_index() {
            return Vec::new();
        }
        Self::part_hits(self, object, point, tolerance)
    }

    fn part_bounds(&self, page_index: usize, object: usize, part: usize) -> Option<Rect> {
        if page_index != self.page_index() {
            return None;
        }
        Self::part_bounds_canvas(self, object, part)
    }

    fn nearest_node(
        &self,
        page_index: usize,
        object: usize,
        part: usize,
        point: Pos2,
        tolerance: f64,
    ) -> Option<usize> {
        if page_index != self.page_index() {
            return None;
        }
        Self::nearest_node(self, object, part, point, tolerance)
    }
}
