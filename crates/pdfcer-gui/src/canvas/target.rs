//! # `canvas::target` — the seam a hit-testable content model plugs into
//!
//! The canvas selects *things*. It does not know what a thing is, how it was
//! decomposed, or what coordinate frame its geometry was authored in. All it
//! needs is: **what is under this point, what is inside this rect, and where
//! is the thing I already have?** That question set is
//! [`CanvasTargetProvider`], and everything in `canvas/` is written against
//! it rather than against `pdfcer-core`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/target.md`.

use egui::{Pos2, Rect};
use pdfcer_core::vector::{FormMarquee, MarqueeMode};

use crate::canvas::pick::PickClass;
use crate::panels::objects::provider::ObjectModelProvider;
pub use crate::panels::objects::provider::TargetId;

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
        use crate::panels::objects::provider::PartKind;
        match (self.part_kind_of(target), target.page_object_index()) {
            (Some(PartKind::Subpath), _) => self.subpath_hits_of(target, point, tolerance),
            //
            // The first version of this override handled `Subpath` and answered
            // empty for everything else, which silently dropped the `Run` arm
            // the index-based `part_hits` had always dispatched. The symptom
            // was `canvas-anchors-declined reason=not-entered`: a Points-tool
            // click on text found no part, so the rung was never entered and no
            // points drew. Every unit test passed; the driven suite caught it.
            //
            // ⇒ The lesson is the one this project keeps relearning about
            // generalising a function: the new axis (which index space) is easy
            // to see, and the axis that was ALREADY there (which kind of part)
            // is the one that gets dropped.
            (Some(PartKind::TextLine), Some(object)) => {
                self.text_line_hits(object, point, tolerance)
            }
            // A text line INSIDE a form has no leaf-indexed hit test yet —
            // `text_line_hits` indexes the page's own list, and answering from it
            // would return another object's lines entirely. Empty is the honest
            // answer, and it is the next thing to build rather than an
            // oversight.
            (Some(PartKind::TextLine), None) | (None, _) => Vec::new(),
        }
    }

    // Dispatches on the KIND, for the reason written out at length in
    // `part_hits_of` above: the first version of that function generalised over
    // index space and dropped the part-kind axis that was already there, and
    // this one had the same defect — subpath-only, so a text line's outline
    // came back `None` the moment the Part rung was reached through a
    // `TargetId` rather than an object index.
    fn part_bounds_of(&self, page_index: usize, target: TargetId, part: usize) -> Option<Rect> {
        if page_index != self.page_index() {
            return None;
        }
        use crate::panels::objects::provider::PartKind;
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
        Some(PickClass::of_object(
            crate::panels::objects::summary::object_kind(object),
        ))
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

/// A provider assembled from plain rectangles — the seam every selection
/// test in `canvas/` uses.
#[cfg(test)]
#[derive(Debug, Default, Clone)]
pub struct StubTargets {
    /// Which page this stub answers for.
    pub page: usize,
    /// One rect per object, in paint order.
    pub objects: Vec<Rect>,
    /// One rect per **form-interior leaf**, in the order the real provider
    /// would list them.
    ///
    /// A second list rather than a flag on the first, because that is the
    /// shape the engine ships and the shape the two index spaces have. A stub
    /// that modelled a leaf as "an object with a marker" could not reproduce
    /// the one property every test here turns on: that `objects[1]` and
    /// `leaves[1]` are different things, and that only the first is an edit
    /// operand.
    ///
    ///
    /// It read *"deliberately not hit by `hit_test_rect`, matching the live
    /// provider"*. The live provider has returned leaves from a marquee since
    /// the day that method was written — its own comment says so at length —
    /// so the stub and the shell disagreed about the one thing this list
    /// exists to model, and every test that used the stub to reason about a
    /// band near a form was measuring a shell that does not exist.
    ///
    /// The general rule, which this is the second instance of in this file:
    /// **a double's doc comment claiming to match the real thing is a claim to
    /// measure, not a note to write.** Nothing fails when it stops being true.
    pub leaves: Vec<Rect>,
    /// Optional per-object part rects, in part order. An object with no
    /// entry has no parts — the image case.
    pub parts: std::collections::BTreeMap<usize, Vec<Rect>>,
    /// Optional per-object anchor samples, in **PDF user space** — the
    /// circular measure tool's fit input.
    ///
    /// Absent for an object means *"carries no fit geometry"*, which is the
    /// text/image case the real provider answers the same way. Stated
    /// explicitly rather than derived from [`Self::objects`] because that
    /// distinction is precisely what `measure::circular::click` refuses on, and
    /// a stub that manufactured four corners for every object could not
    /// express the refusal at all.
    pub samples: std::collections::BTreeMap<usize, Vec<pdfcer_core::vector::Point>>,
    /// Which page object each leaf is painted inside, as `(leaf, object)`.
    ///
    /// Empty means *"these leaves have no container this stub knows about"*,
    /// which is what the live provider answers for a target that is not in a
    /// form — so a test that does not set it gets the no-substitution path.
    pub containers: std::collections::BTreeMap<usize, usize>,
}

#[cfg(test)]
impl StubTargets {
    /// A stub for `page` holding `objects` in paint order.
    pub fn new(page: usize, objects: impl IntoIterator<Item = Rect>) -> Self {
        Self {
            page,
            objects: objects.into_iter().collect(),
            leaves: Vec::new(),
            parts: std::collections::BTreeMap::new(),
            samples: std::collections::BTreeMap::new(),
            containers: std::collections::BTreeMap::new(),
        }
    }

    /// Say which page object each leaf lives inside, as `(leaf, object)`.
    #[must_use]
    pub fn with_containers(mut self, pairs: impl IntoIterator<Item = (usize, usize)>) -> Self {
        self.containers = pairs.into_iter().collect();
        self
    }

    /// Give the page some form-interior leaves, front-most last.
    #[must_use]
    pub fn with_leaves(mut self, leaves: impl IntoIterator<Item = Rect>) -> Self {
        self.leaves = leaves.into_iter().collect();
        self
    }

    /// Give `object` some parts.
    #[must_use]
    pub fn with_parts(mut self, object: usize, parts: impl IntoIterator<Item = Rect>) -> Self {
        self.parts.insert(object, parts.into_iter().collect());
        self
    }

    /// Give `object` some page-space anchor samples — what makes it pickable
    /// by the circular measure tool.
    #[must_use]
    pub fn with_samples(
        mut self,
        object: usize,
        samples: impl IntoIterator<Item = pdfcer_core::vector::Point>,
    ) -> Self {
        self.samples.insert(object, samples.into_iter().collect());
        self
    }

    /// The rect grown by `tolerance` on every side — the stub's model of "a
    /// click may miss an edge by the catch radius". Crude next to the real
    /// per-segment distance test, and deliberately so: what the selection
    /// layer must get right is *that it passes a page-space tolerance at
    /// all*, and a stub that ignored the argument could not fail that way.
    fn caught(rect: Rect, tolerance: f64) -> Rect {
        #[allow(
            clippy::cast_possible_truncation,
            reason = "a catch radius is a handful of points; f32 is exact well past that" // ui-text-exempt: clippy lint justification, never displayed
        )]
        let pad = tolerance.max(0.0) as f32;
        rect.expand(pad)
    }
}

#[cfg(test)]
impl CanvasTargetProvider for StubTargets {
    fn hit_test_all(&self, page_index: usize, point: Pos2, tolerance: f64) -> Vec<TargetId> {
        if page_index != self.page {
            return Vec::new();
        }
        let leaves = self
            .leaves
            .iter()
            .enumerate()
            .filter(|(_, r)| Self::caught(**r, tolerance).contains(point))
            .map(|(i, _)| TargetId::Leaf(i as u64))
            .rev();
        let objects = self
            .objects
            .iter()
            .enumerate()
            .filter(|(_, r)| Self::caught(**r, tolerance).contains(point))
            .map(|(i, _)| TargetId::Object(i as u64))
            // Paint order is back to front; the contract is front-most first.
            .rev();
        leaves.chain(objects).collect()
    }

    fn object_sample_points(
        &self,
        page_index: usize,
        index: usize,
    ) -> Vec<pdfcer_core::vector::Point> {
        if page_index != self.page {
            return Vec::new();
        }
        self.samples.get(&index).cloned().unwrap_or_default()
    }

    fn containing_form(&self, page_index: usize, target: TargetId) -> Option<TargetId> {
        if page_index != self.page {
            return None;
        }
        let leaf = target.leaf_index()?;
        self.containers
            .get(&leaf)
            .map(|object| TargetId::Object(*object as u64))
    }

    fn hit_test_rect(
        &self,
        page_index: usize,
        rect: Rect,
        mode: MarqueeMode,
        forms: FormMarquee,
    ) -> Vec<TargetId> {
        if page_index != self.page {
            return Vec::new();
        }
        let selects = |r: &Rect| match mode {
            MarqueeMode::Enclosed => rect.contains_rect(*r),
            MarqueeMode::Touched => rect.intersects(*r),
        };
        // The stub has no `ImageSource`, so it cannot tell a form from any
        // other page object the way the engine does. What it CAN model is the
        // consequence, which is the part a caller depends on: under `Exclude`
        // an object that some leaf names as its container is skipped. That is
        // the same derivation `without_page_wrappers` uses — a container is
        // whatever `containers` says is one — so the stub's notion of a form
        // and the shell's notion of a form come from one place.
        let is_form = |i: usize| self.containers.values().any(|c| *c == i);
        let mut out: Vec<TargetId> = self
            .objects
            .iter()
            .enumerate()
            .filter(|(i, r)| selects(r) && !(forms == FormMarquee::Exclude && is_form(*i)))
            .map(|(i, _)| TargetId::Object(i as u64))
            .collect();
        // Leaves, appended rather than interleaved. The live provider gets
        // paint order from the engine; this stub has no paint order to get,
        // because a `Rect` carries none. Appending is therefore an honest
        // simplification rather than a divergence to hide: the SET is the
        // contract these tests assert on, and the one test that cared about
        // ordering asserts it against the real provider, where the order is
        // real.
        out.extend(
            self.leaves
                .iter()
                .enumerate()
                .filter(|(_, r)| selects(r))
                .map(|(i, _)| TargetId::Leaf(i as u64)),
        );
        out
    }

    fn bounds(&self, page_index: usize, target: TargetId) -> Option<Rect> {
        if page_index != self.page {
            return None;
        }
        match target {
            TargetId::Object(i) => self.objects.get(usize::try_from(i).ok()?).copied(),
            TargetId::Leaf(i) => self.leaves.get(usize::try_from(i).ok()?).copied(),
        }
    }

    fn part_hits(
        &self,
        page_index: usize,
        object: usize,
        point: Pos2,
        tolerance: f64,
    ) -> Vec<usize> {
        if page_index != self.page {
            return Vec::new();
        }
        self.parts
            .get(&object)
            .map(|parts| {
                parts
                    .iter()
                    .enumerate()
                    .filter(|(_, r)| Self::caught(**r, tolerance).contains(point))
                    .map(|(i, _)| i)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn part_bounds(&self, page_index: usize, object: usize, part: usize) -> Option<Rect> {
        if page_index != self.page {
            return None;
        }
        self.parts.get(&object)?.get(part).copied()
    }

    fn nearest_node(
        &self,
        page_index: usize,
        object: usize,
        part: usize,
        point: Pos2,
        tolerance: f64,
    ) -> Option<usize> {
        // The stub's nodes are the part rect's four corners, numbered from
        // the object's first part — object-scoped, as the contract requires.
        if page_index != self.page {
            return None;
        }
        let parts = self.parts.get(&object)?;
        let offset = parts.iter().take(part).count() * 4;
        let rect = parts.get(part)?;
        let corners = [
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
        ];
        corners
            .iter()
            .enumerate()
            .map(|(i, c)| (i, f64::from(c.distance(point))))
            .filter(|(_, d)| *d <= tolerance)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| offset + i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(Pos2::new(x, y), egui::vec2(w, h))
    }

    /// The stub reports front-most first, so a test that depends on stacking
    /// order is exercising the same contract the live provider honours.
    #[test]
    fn the_stub_reports_the_front_most_object_first() {
        let p = StubTargets::new(
            0,
            [rect(0.0, 0.0, 100.0, 100.0), rect(40.0, 40.0, 20.0, 20.0)],
        );
        assert_eq!(
            p.hit_test_all(0, Pos2::new(50.0, 50.0), 0.0),
            vec![TargetId::Object(1), TargetId::Object(0)]
        );
        assert_eq!(
            p.hit_test(0, Pos2::new(50.0, 50.0), 0.0),
            Some(TargetId::Object(1))
        );
        // A query about another page is a miss, not a panic.
        assert!(p.hit_test_all(1, Pos2::new(50.0, 50.0), 0.0).is_empty());
    }

    /// The stub actually consults the tolerance it is handed. A stub that
    /// ignored it could not fail the way a caller passing raw screen pixels
    /// fails, which would make every selection test blind to the defect this
    /// stage is most at risk of.
    #[test]
    fn the_stub_honours_the_tolerance_it_is_given() {
        let p = StubTargets::new(0, [rect(0.0, 0.0, 10.0, 10.0)]);
        let just_outside = Pos2::new(14.0, 5.0);
        assert!(p.hit_test_all(0, just_outside, 1.0).is_empty());
        assert_eq!(
            p.hit_test_all(0, just_outside, 6.0),
            vec![TargetId::Object(0)]
        );
    }

    /// The marquee encloses rather than touches, on both sides of the seam.
    #[test]
    fn the_stub_marquee_requires_full_enclosure() {
        let p = StubTargets::new(
            0,
            [rect(0.0, 0.0, 10.0, 10.0), rect(100.0, 100.0, 10.0, 10.0)],
        );
        let grazing = Rect::from_min_size(Pos2::new(5.0, 5.0), egui::vec2(200.0, 200.0));
        assert_eq!(
            p.hit_test_rect(0, grazing, MarqueeMode::Enclosed, FormMarquee::Include),
            vec![TargetId::Object(1)],
            "an object the marquee only grazes must not be selected"
        );
        // …and the SAME band as a crossing window takes BOTH — O88.
        //
        // The pair is the point. An `Enclosed`-only assertion passes against a
        // stub that ignores its mode argument entirely, which is exactly the
        // shape a hurried implementation of this change would have: the
        // parameter added, threaded, and never read. Asserting both modes over
        // one rect is the only way this test can tell them apart.
        assert_eq!(
            p.hit_test_rect(0, grazing, MarqueeMode::Touched, FormMarquee::Include),
            vec![TargetId::Object(0), TargetId::Object(1)],
            "a crossing window must take the object it only grazes as well"
        );
    }

    /// The stub's marquee reaches INSIDE a form, and honours `FormMarquee`.
    #[test]
    fn the_stub_marquee_reaches_inside_a_form_and_honours_the_policy() {
        // Object 0 is the form: object 1's leaf names it as its container.
        // Object 1 sits outside the band entirely, so it cannot be confused
        // with the leaf in either direction.
        let mut p = StubTargets::new(
            0,
            [rect(0.0, 0.0, 100.0, 100.0), rect(500.0, 500.0, 10.0, 10.0)],
        );
        p.leaves = vec![rect(10.0, 10.0, 20.0, 20.0)];
        p.containers.insert(0, 0);
        let band = Rect::from_min_size(Pos2::new(-5.0, -5.0), egui::vec2(120.0, 120.0));

        assert_eq!(
            p.hit_test_rect(0, band, MarqueeMode::Enclosed, FormMarquee::Include),
            vec![TargetId::Object(0), TargetId::Leaf(0)],
            "Include must return the container AND what is drawn inside it — \
             which is what the shipped rubber band does"
        );
        assert_eq!(
            p.hit_test_rect(0, band, MarqueeMode::Enclosed, FormMarquee::Exclude),
            vec![TargetId::Leaf(0)],
            "Exclude must drop the container and keep the leaf; a stub that \
             accepted `forms` and ignored it would return the container here"
        );
    }

    /// Node indices stay object-scoped across a part boundary — the same law
    /// the real provider's `node_rung_tests` pins, restated on the stub so a
    /// selection test reading a node index is reading the same numbering the
    /// live provider would have produced.
    #[test]
    fn stub_node_indices_keep_counting_across_parts() {
        let p = StubTargets::new(0, [rect(0.0, 0.0, 100.0, 100.0)]).with_parts(
            0,
            [rect(0.0, 0.0, 10.0, 10.0), rect(50.0, 50.0, 10.0, 10.0)],
        );
        assert_eq!(p.nearest_node(0, 0, 0, Pos2::new(0.0, 0.0), 2.0), Some(0));
        assert_eq!(
            p.nearest_node(0, 0, 1, Pos2::new(50.0, 50.0), 2.0),
            Some(4),
            "the second part's points must continue the object's numbering"
        );
        // Out of tolerance is nothing, rather than the nearest regardless.
        assert_eq!(p.nearest_node(0, 0, 0, Pos2::new(30.0, 30.0), 2.0), None);
    }
}
