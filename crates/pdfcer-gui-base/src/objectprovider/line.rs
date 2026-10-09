//! **The Part rung's unit for a text object: one visual LINE, not one show
//! operator.**
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/objectprovider/line.md`.

use std::ops::Range;

use egui::{Pos2, Rect};
use pdfcer_core::vector::{Bounds, RunPositioning, SplitGranularity, TextObject, VectorObject};

use super::{ObjectModelProvider, RunMoveBlock, TargetId};

/// The lines of `text`, as half-open ranges over its run indices.
#[must_use]
fn lines_of(text: &TextObject) -> Vec<Range<usize>> {
    let count = text.runs.len();
    if count == 0 {
        return Vec::new();
    }
    let cuts = pdfcer_core::vector::edit::text_object_split_points(text, SplitGranularity::Line);
    let mut out = Vec::with_capacity(cuts.len() + 1);
    let mut start = 0usize;
    for cut in cuts {
        out.push(start..cut);
        start = cut;
    }
    out.push(start..count);
    out
}

impl ObjectModelProvider {
    /// The text object `target` names, or `None` for anything else.
    fn text_of(&self, target: TargetId) -> Option<&TextObject> {
        match self.object_for(target)? {
            VectorObject::Text(t) => Some(t),
            _ => None,
        }
    }

    /// How many visual lines the text object `target` has — `0` for anything
    /// else, the same answer and for the same reason as
    /// [`ObjectModelProvider::text_run_count`]'s.
    ///
    /// This is the Part rung's denominator: the *of* in *1 line of 144*.
    #[must_use]
    pub fn text_line_count_of(&self, target: TargetId) -> usize {
        self.text_of(target).map_or(0, |t| lines_of(t).len())
    }

    /// [`Self::text_line_count_of`] for a page object index.
    #[must_use]
    pub fn text_line_count(&self, object: usize) -> usize {
        self.text_line_count_of(TargetId::Object(object as u64))
    }

    /// The show operators line `line` of `target` is written in.
    #[must_use]
    pub fn text_line_runs_of(&self, target: TargetId, line: usize) -> Option<Range<usize>> {
        lines_of(self.text_of(target)?).get(line).cloned()
    }

    /// [`Self::text_line_runs_of`] for a page object index.
    #[must_use]
    pub fn text_line_runs(&self, object: usize, line: usize) -> Option<Range<usize>> {
        self.text_line_runs_of(TargetId::Object(object as u64), line)
    }

    /// Which line `run` belongs to.
    ///
    /// The translation the hit test needs: `pdfcer-core` answers a point in run
    /// indices and the Part rung addresses lines.
    #[must_use]
    pub fn text_line_of_run_of(&self, target: TargetId, run: usize) -> Option<usize> {
        lines_of(self.text_of(target)?)
            .iter()
            .position(|r| r.contains(&run))
    }

    /// [`Self::text_line_of_run_of`] for a page object index.
    #[must_use]
    pub fn text_line_of_run(&self, object: usize, run: usize) -> Option<usize> {
        self.text_line_of_run_of(TargetId::Object(object as u64), run)
    }

    /// The lines under `point`, nearest first and each named once.
    #[must_use]
    pub fn text_line_hits(&self, object: usize, point: Pos2, tolerance: f64) -> Vec<usize> {
        let Some(text) = self.text_of(TargetId::Object(object as u64)) else {
            return Vec::new();
        };
        let lines = lines_of(text);
        let mut out: Vec<usize> = Vec::new();
        for run in self.text_run_hits(object, point, tolerance) {
            if let Some(line) = lines.iter().position(|r| r.contains(&run))
                && !out.contains(&line)
            {
                out.push(line);
            }
        }
        out
    }

    /// [`Self::text_line_hits`], for either index space.
    ///
    /// A page object asks the engine's `hit_test_text_runs`, which takes a
    /// page-list index; a form leaf has no index in that list, so it asks
    /// `hit_test_text_runs_of` with the leaf's text object in hand. Same rule
    /// and ordering either way.
    #[must_use]
    pub fn text_line_hits_of(&self, target: TargetId, point: Pos2, tolerance: f64) -> Vec<usize> {
        if let Some(object) = target.page_object_index() {
            return self.text_line_hits(object, point, tolerance);
        }
        let (Some(text), Some(pdf)) = (self.text_of(target), self.canvas_to_pdf(point)) else {
            return Vec::new();
        };
        let runs = pdfcer_core::vector::hit_test_text_runs_of(text, pdf, super::resolve(tolerance));
        let lines = lines_of(text);
        let mut out: Vec<usize> = Vec::new();
        for run in runs {
            if let Some(line) = lines.iter().position(|r| r.contains(&run))
                && !out.contains(&line)
            {
                out.push(line);
            }
        }
        out
    }

    /// A line's bounds in **canvas** space, for drawing its outline.
    #[must_use]
    pub fn text_line_bounds_canvas_of(&self, target: TargetId, line: usize) -> Option<Rect> {
        let text = self.text_of(target)?;
        let runs = lines_of(text).get(line).cloned()?;
        let box_ = runs
            .filter_map(|r| text.runs.get(r))
            .fold(Bounds::EMPTY, |acc, run| acc.union(run.bounds));
        if box_.is_empty() {
            return None;
        }
        self.pdf_bounds_to_canvas(box_)
    }

    /// [`Self::text_line_bounds_canvas_of`] for a page object index.
    #[must_use]
    pub fn text_line_bounds_canvas(&self, object: usize, line: usize) -> Option<Rect> {
        self.text_line_bounds_canvas_of(TargetId::Object(object as u64), line)
    }

    /// Why moving line `line` of `target` would be refused, or `None` when
    /// the line can be moved whole: the engine's set guard over the line's
    /// runs, [`Self::text_runs_move_refusal_of`]. A line with no runs answers
    /// [`RunMoveBlock::NotThere`].
    #[must_use]
    pub fn text_line_move_refusal_of(&self, target: TargetId, line: usize) -> Option<RunMoveBlock> {
        let runs: Vec<usize> = self.text_line_runs_of(target, line)?.collect();
        self.text_runs_move_refusal_of(target, &runs)
    }

    /// [`Self::text_line_move_refusal_of`] for a page object index.
    #[must_use]
    pub fn text_line_move_refusal(&self, object: usize, line: usize) -> Option<RunMoveBlock> {
        self.text_line_move_refusal_of(TargetId::Object(object as u64), line)
    }

    /// Whether deleting line `line` of `target` would drag the line after it —
    /// the delete twin of [`Self::text_line_move_refusal_of`]'s last clause.
    #[must_use]
    pub fn text_line_delete_would_move_next_of(&self, target: TargetId, line: usize) -> bool {
        let Some(text) = self.text_of(target) else {
            return false;
        };
        let Some(runs) = lines_of(text).get(line).cloned() else {
            return false;
        };
        if runs.start == 0 && runs.end == text.runs.len() {
            return false;
        }
        text.runs
            .get(runs.end)
            .is_some_and(|next| next.positioned_by == RunPositioning::Inherited)
    }

    /// [`Self::text_line_delete_would_move_next_of`] for a page object index.
    #[must_use]
    pub fn text_line_delete_would_move_next(&self, object: usize, line: usize) -> bool {
        self.text_line_delete_would_move_next_of(TargetId::Object(object as u64), line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objectprovider::PartKind;
    use pdfcer_core::span::ByteSpan;
    use pdfcer_core::vector::{
        DecomposeDiagnostics, Matrix, PageObjects, Point, TextBoundsBasis, TextPreview, TextRun,
        TokenRange,
    };
    use pdfcer_render::tiny_skia::Transform;

    /// The fixture is built by hand, because `decompose` over a content
    /// stream produces NO runs: [`TextObject::runs`] is empty when no run
    /// could be laid out, and the resolver-only entry points have no `/Font`
    /// resources to lay one out with. That is why `pdfcer_gui::panels::objects::provider_tests`'s two
    /// text fixtures assert `part_count(0) == text_run_count(0)` without ever
    /// naming a number.
    fn text_object(runs: Vec<TextRun>) -> ObjectModelProvider {
        let page_bbox = runs
            .iter()
            .fold(Bounds::EMPTY, |acc, r| acc.union(r.bounds));
        let objects = PageObjects {
            objects: vec![VectorObject::Text(TextObject {
                page_bbox,
                runs,
                approximate: true,
                bounds_basis: TextBoundsBasis::FontMetrics,
                preview: TextPreview::Unavailable,
                font: None,
                tokens: TokenRange { start: 0, end: 1 },
                bytes: ByteSpan { start: 0, len: 1 },
                ctm: Matrix::IDENTITY,
                oc: None,
            })],
            initial: Matrix::IDENTITY,
            diagnostics: DecomposeDiagnostics::default(),
            leaves: Vec::new(),
        };
        ObjectModelProvider::from_parts(0, objects, Transform::identity())
    }

    /// One show operator on baseline `y`, spanning `x0..x1`.
    fn run(x0: f64, x1: f64, y: f64, positioned_by: RunPositioning) -> TextRun {
        TextRun {
            bounds: Bounds::EMPTY
                .union_point(Point::new(x0, y - 2.0))
                .union_point(Point::new(x1, y + 8.0)),
            tokens: TokenRange { start: 0, end: 1 },
            bytes: ByteSpan { start: 0, len: 1 },
            positioned_by,
            text_matrix: Matrix {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 1.0,
                e: x0,
                f: y,
            },
            text_start: 0,
            text_end: 0,
        }
    }

    /// Three show operators forming **two** lines: runs 0 and 1 share
    /// baseline 700 with run 1 riding on run 0's advance, and run 2 starts a
    /// new line at 660 with a `Tm` of its own.
    fn two_lines() -> ObjectModelProvider {
        text_object(vec![
            run(72.0, 100.0, 700.0, RunPositioning::Explicit),
            run(100.0, 180.0, 700.0, RunPositioning::Inherited),
            run(72.0, 190.0, 660.0, RunPositioning::Explicit),
        ])
    }

    /// Three self-positioned show operators on one baseline — his case cut
    /// down to the smallest size that shows the defect.
    fn one_line_three_fragments() -> ObjectModelProvider {
        text_object(vec![
            run(72.0, 100.0, 700.0, RunPositioning::Explicit),
            run(104.0, 140.0, 700.0, RunPositioning::Explicit),
            run(144.0, 190.0, 700.0, RunPositioning::Explicit),
        ])
    }

    /// A leaf's line hit test answers what the page object's does for the same
    /// text: the two arms of [`ObjectModelProvider::text_line_hits_of`] agree.
    #[test]
    fn a_leaf_text_object_hits_its_lines_as_a_page_object_does() {
        use pdfcer_core::vector::FormLeaf;
        let page = two_lines();
        let mut leaf = two_lines();
        let object = leaf.objects.objects.remove(0);
        leaf.objects.leaves.push(FormLeaf {
            object,
            containment: vec![pdfcer_core::object::ObjId::new(5, 0)],
            paint_order: 0,
            placement: Matrix::IDENTITY,
            form_object_index: 0,
        });
        let mut hit_any = false;
        for (x, y) in [
            (150.0, 703.0),
            (100.0, 663.0),
            (150.0, 663.0),
            (400.0, 400.0),
        ] {
            let point = Pos2::new(x, y);
            let want = page.text_line_hits(0, point, 1.0);
            hit_any |= !want.is_empty();
            assert_eq!(
                leaf.text_line_hits_of(TargetId::Leaf(0), point, 1.0),
                want,
                "at {point:?}"
            );
        }
        assert!(hit_any, "the probe points must reach a line");
    }

    #[test]
    fn runs_sharing_a_baseline_are_one_line() {
        let p = two_lines();
        assert_eq!(
            p.part_kind(0),
            Some(PartKind::TextLine),
            "it is a text object"
        );
        assert_eq!(p.text_run_count(0), 3, "three show operators");
        assert_eq!(p.text_line_count(0), 2, "two visual lines");
        assert_eq!(p.text_line_runs(0, 0), Some(0..2));
        assert_eq!(p.text_line_runs(0, 1), Some(2..3));
        assert_eq!(p.text_line_runs(0, 2), None, "there is no third line");
    }

    #[test]
    fn his_line_is_one_line_however_many_fragments_it_took() {
        // The defect at the smallest size that shows it: three show
        // operators, one line. A Part rung indexed in show operators offers
        // three targets here and moves a third of the words.
        let p = one_line_three_fragments();
        assert_eq!(p.text_run_count(0), 3);
        assert_eq!(p.text_line_count(0), 1);
        assert_eq!(p.text_line_runs(0, 0), Some(0..3));
        assert_eq!(
            p.text_line_move_refusal(0, 0),
            None,
            "every fragment carries its own Tm, so all three moves are planned"
        );
    }

    #[test]
    fn a_run_maps_to_the_line_that_holds_it() {
        let p = two_lines();
        assert_eq!(p.text_line_of_run(0, 0), Some(0));
        assert_eq!(p.text_line_of_run(0, 1), Some(0), "the inherited fragment");
        assert_eq!(p.text_line_of_run(0, 2), Some(1));
        assert_eq!(p.text_line_of_run(0, 9), None);
    }

    /// A line whose second fragment inherits its position moves whole: the
    /// set guard lets the follower through because its predecessor moves too.
    #[test]
    fn a_line_whose_second_fragment_inherits_moves_whole() {
        let p = two_lines();
        // Run 1 has no positioning operator of its own and rides on run 0.
        assert_eq!(p.text_line_move_refusal(0, 0), None);
        // The second line is one self-positioned fragment with nothing after
        // it, so nothing stands in the way.
        assert_eq!(p.text_line_move_refusal(0, 1), None);
    }

    /// The twin of the test above, and the only shape that earns
    /// [`RunMoveBlock::NoPositionOfItsOwn`] at line granularity: the line's
    /// **first** fragment is the one with no position, which for horizontal
    /// text can only be run 0 of the whole object.
    #[test]
    fn a_line_whose_first_fragment_inherits_names_the_line() {
        let mut p = one_line_three_fragments();
        let VectorObject::Text(text) = &mut p.objects.objects[0] else {
            unreachable!("fixture is a text object")
        };
        text.runs[0].positioned_by = RunPositioning::Inherited;
        assert_eq!(
            p.text_line_move_refusal(0, 0),
            Some(RunMoveBlock::NoPositionOfItsOwn)
        );
    }

    #[test]
    fn a_line_whose_successor_inherits_would_drag_it() {
        // Run 1 is on a different baseline and still inherits — which is what
        // rotated text does, since its advance changes `f` as well as `e`.
        // Moving line 0 would drag it.
        let p = text_object(vec![
            run(72.0, 100.0, 700.0, RunPositioning::Explicit),
            run(72.0, 190.0, 660.0, RunPositioning::Inherited),
        ]);
        assert_eq!(p.text_line_count(0), 2);
        assert_eq!(
            p.text_line_move_refusal(0, 0),
            Some(RunMoveBlock::WouldMoveNextRun)
        );
    }

    #[test]
    fn the_line_box_spans_every_fragment_of_it() {
        let p = one_line_three_fragments();
        let line = p.text_line_bounds_canvas(0, 0).expect("the line has a box");
        let fragment = p
            .text_run_bounds_canvas(0, 1)
            .expect("its middle fragment has one too");
        assert!(
            line.contains_rect(fragment) && line.width() > fragment.width() + 1.0,
            "the line box encloses the clicked fragment and is wider: {line:?} vs {fragment:?}"
        );
    }

    #[test]
    fn a_click_on_any_fragment_names_the_same_line() {
        let p = one_line_three_fragments();
        // The canvas transform is the identity, so these are PDF coordinates:
        // a press in the middle of each of the three fragments.
        for x in [86.0_f32, 122.0, 167.0] {
            assert_eq!(
                p.text_line_hits(0, Pos2::new(x, 702.0), 3.0),
                vec![0],
                "a press at x={x} is on line 0, and named once"
            );
        }
    }

    #[test]
    fn a_non_text_object_has_no_lines() {
        let cs = pdfcer_core::content::ContentStream::parse(b"10 10 m 90 90 l S".to_vec())
            .expect("parse");
        let p = ObjectModelProvider::from_parts(
            0,
            pdfcer_core::vector::decompose(&cs, Matrix::IDENTITY, &pdfcer_core::vector::NoXObjects),
            Transform::identity(),
        );
        assert_eq!(p.text_line_count(0), 0);
        assert_eq!(p.text_line_runs(0, 0), None);
        assert_eq!(p.text_line_bounds_canvas(0, 0), None);
        assert_eq!(p.text_line_move_refusal(0, 0), None);
        assert!(p.text_line_hits(0, Pos2::new(50.0, 50.0), 3.0).is_empty());
        assert!(!p.text_line_delete_would_move_next(0, 0));
    }

    #[test]
    fn deleting_a_line_asks_about_the_run_after_the_line() {
        let p = two_lines();
        // Line 0 is runs 0..2, so the run after it is run 2, which has a `Tm`
        // of its own and is not orphaned. Asking about the run after the
        // clicked FRAGMENT instead answers `true` here, because run 1
        // inherits — and would refuse a legal delete.
        assert!(!p.text_line_delete_would_move_next(0, 0));
        assert!(
            p.text_run_delete_would_move_next(0, 0),
            "which is exactly what the run-unit question answers"
        );
        // Line 1 is the last line, so there is nothing after it at all.
        assert!(!p.text_line_delete_would_move_next(0, 1));
    }

    /// Decompose page 0 of a fixture from this repository's `fixtures/`.
    fn local_fixture(rel: &str) -> ObjectModelProvider {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures")
            .join(rel);
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
        ObjectModelProvider::build(&doc.view(), &pages[0], 0).expect("the fixture page decomposes")
    }

    /// **The four answers, on a real document, in one page.**
    #[test]
    fn the_local_fixture_gives_all_four_line_move_answers() {
        let p = local_fixture("inherited-runs.pdf");
        let object = (0..p.page_objects().objects.len())
            .find(|&i| p.text_line_count(i) > 0)
            .expect("the fixture holds a text object");

        assert_eq!(p.text_run_count(object), 5, "five show operators");
        assert_eq!(p.text_line_count(object), 4, "four visual lines");

        assert_eq!(p.text_line_runs(object, 0), Some(0..2), "Alpha + Beta");
        assert_eq!(p.text_line_runs(object, 1), Some(2..3), "Gamma");
        assert_eq!(p.text_line_runs(object, 2), Some(3..4), "Delta");
        assert_eq!(p.text_line_runs(object, 3), Some(4..5), "Epsilon");

        assert_eq!(
            p.text_line_move_refusal(object, 0),
            None,
            "the join inside the line moves with the piece before it"
        );
        assert_eq!(
            p.text_line_move_refusal(object, 1),
            None,
            "the control — one explicitly placed run, nothing inherits from it"
        );
        assert_eq!(
            p.text_line_move_refusal(object, 2),
            Some(RunMoveBlock::WouldMoveNextRun),
            "the next LINE rides on this one's advance"
        );
        assert_eq!(
            p.text_line_move_refusal(object, 3),
            Some(RunMoveBlock::NoPositionOfItsOwn),
            "this line's first and only piece is inherited"
        );
    }

    /// The four points `move_line_of_text::AIMS` presses at land on the four
    /// lines it says they do — one each, and no point inside two boxes.
    #[test]
    fn the_aims_driven_at_this_fixture_land_one_per_line() {
        const AIMS: [(f64, f64); 4] = [
            (87.0, 704.0),
            (128.0, 664.0),
            (297.0, 414.0),
            (297.0, 448.0),
        ];

        let p = local_fixture("inherited-runs.pdf");
        let object = (0..p.page_objects().objects.len())
            .find(|&i| p.text_line_count(i) > 0)
            .expect("the fixture holds a text object");
        let text = p
            .text_of(TargetId::Object(object as u64))
            .expect("that object is text");

        let boxes: Vec<Bounds> = lines_of(text)
            .into_iter()
            .map(|runs| {
                runs.filter_map(|r| text.runs.get(r))
                    .fold(Bounds::EMPTY, |acc, run| acc.union(run.bounds))
            })
            .collect();
        assert_eq!(boxes.len(), AIMS.len(), "one aim per line");

        for (aim, (x, y)) in AIMS.iter().copied().enumerate() {
            let hit: Vec<usize> = boxes
                .iter()
                .enumerate()
                .filter(|(_, b)| b.min.x <= x && x <= b.max.x && b.min.y <= y && y <= b.max.y)
                .map(|(i, _)| i)
                .collect();
            assert_eq!(
                hit,
                vec![aim],
                "aim {aim} at ({x}, {y}) must be inside line {aim} and no other; boxes are {boxes:?}"
            );
        }
    }
}
