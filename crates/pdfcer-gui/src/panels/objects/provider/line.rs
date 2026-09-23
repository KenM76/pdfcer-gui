//! **The Part rung's unit for a text object: one visual LINE, not one show
//! operator.**
//!
//! # Why the two are not the same thing
//!
//! A PDF text object is a list of show operators (`Tj`/`TJ`/`'`/`"`), each with
//! its own origin. What the operator sees as one line of a title block is
//! however many of those its producer chose to write. Measured on
//! `SW41177.pdf` page 0, object 5871 — a SolidWorks notes column — that is
//! **237 show operators forming 144 lines**, and the single line
//! `#2 USE SPACERS 8 9 10 11 IF REQUIRED.` is **nine** of them.
//!
//! So a Part rung indexed in show operators tells the operator he is on *line
//! 54 of 237* when the block has 144 lines, draws a selection box round a 19 pt
//! fragment in the middle of the words he clicked, and moves that fragment
//! alone when he drags it. `OPERATOR_REQUESTS.md` O214, in his words: *"take a
//! line like this that is part of a larger block and … relocate it by dragging
//! and moving as if it wasn't part of a larger block."*
//!
//! # The grouping is the engine's, not a second opinion
//!
//! [`pdfcer_core::vector::edit::text_object_split_points`] at
//! [`SplitGranularity::Line`] decides where the lines are, and this module
//! calls it rather than comparing baselines itself. `pdfcer-core` already has
//! to answer this question to plan a split; a shell that answered it a second
//! way would one day draw a box round one grouping and move another.
//!
//! ⚠ **It is an inference and the engine says so** — §14.8 does not record
//! where a text object's lines are. The grouping is *consecutive runs sharing a
//! baseline, in stream order*, which is what makes it usable on CAD output: a
//! sheet has dozens of unrelated labels sharing a y-coordinate across its
//! width, and grouping by baseline alone would weld them into one line.
//!
//! # What follows from the grouping, and it is the whole of the move design
//!
//! A run whose `positioned_by` is
//! [`Inherited`](pdfcer_core::vector::RunPositioning::Inherited) starts wherever
//! the previous show operator's pen stopped — so for horizontal text its
//! baseline is its predecessor's, and it is therefore **always in its
//! predecessor's line group**. Two consequences:
//!
//! 1. **A line's first fragment is the only one that can lack a position of its
//!    own**, and only when it is run 0 of the whole object.
//! 2. **An inherited fragment inside a line needs no move of its own** — it
//!    follows the fragment before it. A line is therefore moved as one SET
//!    through `EditSession::move_text_runs`, and
//!    [`ObjectModelProvider::text_line_move_refusal_of`] asks the engine's set
//!    guard, which lets an inherited run through when its predecessor moves
//!    too.

use std::ops::Range;

use egui::{Pos2, Rect};
use pdfcer_core::vector::{Bounds, RunPositioning, SplitGranularity, TextObject, VectorObject};

use super::{ObjectModelProvider, RunMoveBlock, TargetId};

/// The lines of `text`, as half-open ranges over its run indices.
///
/// Always covers every run exactly once and is never empty for a non-empty
/// object, because [`text_object_split_points`](pdfcer_core::vector::edit::text_object_split_points)
/// never returns 0 and never returns an index past the last run.
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
    ///
    /// `None` for a non-text object or a line index the object does not have —
    /// which is a stale selection, and the caller's cue to say nothing rather
    /// than to name line 0.
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
    ///
    /// Order comes from [`pdfcer_core::vector::hit_test_text_runs`], which
    /// answers nearest-first in run indices; several runs of one line under the
    /// pointer collapse to that line's first sighting, so the caller's
    /// `first()` is still the nearest thing to the pointer.
    ///
    /// Page objects only. A text object painted from inside a form XObject has
    /// no hit test at any granularity — [`Self::text_run_hits`] indexes the
    /// page's own list, so answering a leaf from it would return another
    /// object's runs entirely — and this inherits that hole rather than
    /// papering over it. `canvas::target`'s `part_hits_of` records it in the
    /// same terms.
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

    /// A line's bounds in **canvas** space, for drawing its outline.
    ///
    /// The union of its fragments' boxes. Drawing one fragment's box instead is
    /// the visible half of O214: the operator clicks the middle of a phrase,
    /// gets a rectangle round nineteen points of it, and concludes the
    /// selection is broken.
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
    ///
    /// Asks about the run *after the line*, not about the run after the one the
    /// operator clicked. Deleting a line removes every fragment of it, so the
    /// only run whose origin can be orphaned is the first one left standing.
    ///
    /// `false` when the line is the whole object: deleting every run deletes
    /// the text object, which the engine allows unconditionally.
    ///
    /// This one reads `positioned_by` directly, because the delete-side guard
    /// has no exported twin to call — the standing hazard
    /// [`ObjectModelProvider::text_run_delete_would_move_next`] records.
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
    use crate::panels::objects::provider::PartKind;
    use pdfcer_core::span::ByteSpan;
    use pdfcer_core::vector::{
        DecomposeDiagnostics, Matrix, PageObjects, Point, TextBoundsBasis, TextPreview, TextRun,
        TokenRange,
    };
    use pdfcer_render::tiny_skia::Transform;

    /// The fixture is built by hand, because `decompose` over a content
    /// stream produces NO runs: [`TextObject::runs`] is empty when no run
    /// could be laid out, and the resolver-only entry points have no `/Font`
    /// resources to lay one out with. That is why `provider::tests`'s two
    /// text fixtures assert `part_count(0) == text_run_count(0)` without ever
    /// naming a number.
    ///
    /// **What keeps it from being an oracle written by the thing it
    /// measures:** every field below is fixed by sub-clause 9.4.2 rather than
    /// chosen. `text_matrix` is `Tm` as it stood at the show operator, so two
    /// `Tj`s with no positioning operator between them share one, and a `Tm`
    /// in between gives the second a new `f`. The same shape was measured on
    /// a real decomposition in `tests/ken_sw41177_line_move_probe.rs`, where
    /// nine consecutive runs of one visual line came back sharing baseline
    /// 927.23. The driven proof over a real file is `ui-verify`'s.
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
    ///
    /// ★ Built by declaring run 0 `Inherited`. That is what a text object
    /// whose first show operator relies on the text-state carried in from
    /// before `BT` looks like, and the engine refuses to move it for the same
    /// reason it refuses any other: there is no operand to rewrite.
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

    /// **The four answers, on a real document, in one page.**
    ///
    /// Every other test in this module builds its runs by hand, which makes
    /// them a calibration of the grouping rule and not a measurement of it:
    /// the fixture and the code under test were written from the same reading
    /// of 9.4.2, so both can be wrong together. This one decomposes
    /// `fixtures/inherited-runs.pdf` — a file on disk, written by a generator
    /// that knows nothing about `runs_share_a_line` — and asserts the table in
    /// that generator's header.
    ///
    /// ★★ **The rotated pair is the load-bearing half.** An inherited run
    /// advances along the text direction, so a HORIZONTAL one always lands on
    /// its predecessor's baseline and is always inside its predecessor's line
    /// group. Rotation is the only way a line can BEGIN with an inherited run,
    /// and without it `NoPositionOfItsOwn` and `WouldMoveNextRun` are sentences
    /// no document could produce at line granularity — which would leave a
    /// build that had deleted them passing every check.
    ///
    /// ★ Line 1 is the CONTROL. Without an answer of `None` somewhere on the
    /// page, a build that refused every line move would satisfy the other
    /// three assertions.
    #[test]
    fn the_local_fixture_gives_all_four_line_move_answers() {
        let doc = crate::app::state::open_local_fixture("inherited-runs.pdf");
        let p = doc.page_objects().expect("the fixture page decomposes");
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
    ///
    /// # ★★★ Why a unit test owns the harness's coordinates
    ///
    /// `AIMS` asserts an ANSWER per aim, never a line index, because
    /// `canvas-selection` carries no part index for it to read back. That
    /// makes the aims self-checking only while the four lines give four
    /// different answers — and silently wrong the moment two of them agree.
    /// Here the index IS visible, so the mapping from point to line can be
    /// stated outright.
    ///
    /// The exclusivity half is the load-bearing one. The rotated pair is
    /// stacked along one narrow column and meets at a single y, so a point
    /// that fell in both boxes would still satisfy a containment-only
    /// assertion while aiming at whichever of the two the hit test happened
    /// to return first.
    ///
    /// ★ PDF user space, y-up, straight off the engine's decomposition. No
    /// canvas transform is involved: `AIMS` is in page coordinates and the
    /// harness maps it at drive time, so converting here would introduce the
    /// one step this is meant to hold still.
    #[test]
    fn the_aims_driven_at_this_fixture_land_one_per_line() {
        const AIMS: [(f64, f64); 4] = [
            (87.0, 704.0),
            (128.0, 664.0),
            (297.0, 414.0),
            (297.0, 448.0),
        ];

        let doc = crate::app::state::open_local_fixture("inherited-runs.pdf");
        let p = doc.page_objects().expect("the fixture page decomposes");
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
