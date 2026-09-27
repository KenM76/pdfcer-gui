//! # `canvas::targetstub` — a target provider assembled from plain rectangles
//!
//! The seam every selection test in `canvas/` uses, kept in this crate
//! because only this crate's tests construct it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/targetstub.md`.

use egui::{Pos2, Rect};
use pdfcer_core::vector::{FormMarquee, MarqueeMode};

use crate::canvas::target::{CanvasTargetProvider, TargetId};

/// A provider assembled from plain rectangles — the seam every selection
/// test in `canvas/` uses.
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
