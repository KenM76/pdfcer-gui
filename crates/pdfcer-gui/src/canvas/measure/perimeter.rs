//! # `canvas::measure::perimeter` — **click around a shape; one number for the
//! whole way round**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/measure/perimeter.md`.

use egui::Pos2;

use pdfcer_core::dimension::DimensionKind;
use pdfcer_core::vector::Point;

use crate::canvas::mapping::PageMapping;
use pdfcer_gui_base::measure::place::Placing;

use super::state::MeasureState;

/// The picks made so far, and whether the operator has closed the ring.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PerimeterPick {
    /// The picked vertices, in pick order, page space, points.
    points: Vec<Point>,
    /// Whether the last vertex joins the first.
    ///
    /// Set only by [`Self::close`], i.e. only by the operator clicking the
    /// first vertex. It is never inferred from the geometry: two vertices that
    /// happen to coincide are a shape the operator drew, not a ring they meant.
    closed: bool,
}

/// The fewest vertices an **open** path can have and still be a length.
pub const MIN_OPEN: usize = 2;

/// The fewest vertices a **closed** perimeter can have.
pub const MIN_CLOSED: usize = 3;

impl PerimeterPick {
    /// The vertices so far, in pick order.
    #[must_use]
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// Whether the ring has been closed.
    #[must_use]
    pub const fn is_closed(&self) -> bool {
        self.closed
    }

    /// Whether a gesture is under way — one pick is enough, because the
    /// preview should follow the pointer from the second click onward.
    #[must_use]
    pub const fn in_progress(&self) -> bool {
        !self.points.is_empty()
    }

    /// Add a vertex.
    pub fn push(&mut self, p: Point) {
        self.points.push(p);
    }

    /// Close the ring, reporting whether it could be closed.
    pub fn close(&mut self) -> bool {
        if self.closed || self.points.len() < MIN_CLOSED {
            return false;
        }
        self.closed = true;
        true
    }

    /// Forget everything. Called after a commit and by Escape.
    pub fn clear(&mut self) {
        self.points.clear();
        self.closed = false;
    }

    /// **The dimension this pick would author**, or `None` when there is not
    /// enough of a shape to be one.
    #[must_use]
    pub fn author(&self) -> Option<DimensionKind> {
        let minimum = if self.closed { MIN_CLOSED } else { MIN_OPEN };
        if self.points.len() < minimum {
            return None;
        }
        Some(DimensionKind::Perimeter {
            points: self.points.clone(),
            closed: self.closed,
            offset: 0.0,
            text_along: 0.0,
        })
    }

    /// **The shape as it would be drawn if the operator released now**, with
    /// `pointer` as a provisional last vertex.
    #[must_use]
    pub fn preview(&self, pointer: Point) -> Option<DimensionKind> {
        if self.points.is_empty() {
            return None;
        }
        let mut points = self.points.clone();
        points.push(pointer);
        Some(DimensionKind::Perimeter {
            points,
            // Never previewed as closed. The operator has not closed it, and
            // drawing the closing segment before they do would show a shape
            // one segment longer than the one this click will commit.
            closed: false,
            offset: 0.0,
            text_along: 0.0,
        })
    }

    /// The total length of the picked segments in **page points**, including
    /// the closing one when the ring is closed.
    #[must_use]
    pub fn length_points(&self) -> f64 {
        let mut total = self
            .points
            .windows(2)
            .map(|w| (w[1].x - w[0].x).hypot(w[1].y - w[0].y))
            .sum::<f64>();
        // The closing segment is added HERE, and forgetting it is the hazard
        // the PDF spec corpus names by name for `/Polygon`: `/Vertices` does
        // not repeat the first point, so a perimeter routine that does not
        // close the ring reports a total one segment short of the shape on
        // screen. A number that disagrees with its own picture.
        if self.closed
            && self.points.len() >= MIN_CLOSED
            && let (Some(first), Some(last)) = (self.points.first(), self.points.last())
        {
            total += (first.x - last.x).hypot(first.y - last.y);
        }
        total
    }
}

/// **End the gesture: hand the traced shape to the placing click and empty
/// the pick.** Nothing to disclose: every vertex is one the operator clicked.
pub(super) fn complete(st: &mut MeasureState) -> bool {
    let Some(kind) = st.perimeter.author() else {
        return false;
    };
    st.placing = Some(Placing {
        kind,
        disclosures: Vec::new(),
    });
    st.perimeter.clear();
    true
}

/// **Take one resolved point for the perimeter tool**, and answer the three
/// endings.
pub(super) struct Click<'a> {
    /// The page the picks are on.
    pub page_index: usize,
    /// The point, already through the snap query and the derived-candidate
    /// confirm - see the function's own docs for why this tool takes a
    /// RESOLVED point where the circular tool takes a raw click.
    pub picked: Point,
    /// The same click in canvas space, for the close-the-ring hit test, which
    /// has to happen at a fixed physical size rather than a fixed page size.
    pub canvas_point: Pos2,
    /// Whether this was the second click of a double-click.
    pub double: bool,
    /// The page, for the page -> canvas bridge the ring test needs.
    pub page: &'a pdfcer_core::page_tree::Page,
    /// The frame's mapping, for the click tolerance.
    pub map: &'a PageMapping,
}

pub(super) fn click(st: &mut MeasureState, c: Click<'_>) {
    let Click {
        page_index,
        picked,
        canvas_point,
        double,
        page,
        map,
    } = c;
    if double {
        if !complete(st) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "measure-finish via=double-click outcome=declined reason=too-few-vertices n={}",
                    st.perimeter.points().len()
                )
            });
            return;
        }
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("measure-finish via=double-click kind=perimeter page={page_index}")
        });
        return;
    }

    // **The Length tool never closes**, and the guard is here rather than
    // inside `closes_the_ring` on purpose: that function answers a geometric
    // question - *did this click land on the first vertex?* - and the answer is
    // the same for both tools. What differs is what the click MEANS, which is a
    // property of the armed tool and belongs at the decision, not inside the
    // measurement.
    //
    // For the Length tool a click on the first vertex is an ordinary vertex. A
    // path that returns to where it started is a perfectly ordinary path - a
    // loop of cable is still cable - and swallowing that click would be the
    // tool refusing a shape the operator drew.
    if st.kind == super::MeasureKind::Perimeter
        && closes_the_ring(st, canvas_point, picked, page, map)
    {
        if !st.perimeter.close() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "measure-perimeter-close outcome=declined reason=too-few-vertices n={}",
                    st.perimeter.points().len()
                )
            });
            return;
        }
        if complete(st) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("measure-finish via=close-ring kind=perimeter page={page_index}")
            });
        }
        return;
    }

    st.perimeter.push(picked);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        //
        // One line per vertex, carrying the running total in PAGE POINTS. An
        // armed tool part-way through a shape and an armed tool that has picked
        // nothing are the same screenshot at a glance, which is defect 8's
        // lesson; this is how a driven check proves a click became a vertex.
        format!(
            "measure-perimeter-vertex n={} length_pt={:.2}",
            st.perimeter.points().len(),
            st.perimeter.length_points()
        )
    });
}

/// **Did this click land on the first vertex?**
fn closes_the_ring(
    st: &MeasureState,
    canvas_point: Pos2,
    picked: Point,
    page: &pdfcer_core::page_tree::Page,
    map: &PageMapping,
) -> bool {
    if st.perimeter.points().len() < MIN_CLOSED {
        return false;
    }
    let Some(first) = st.perimeter.points().first() else {
        return false;
    };
    #[allow(clippy::cast_possible_truncation)]
    let as_pos = Pos2::new(first.x as f32, first.y as f32);
    let Some(first_canvas) = crate::viewer::pdf_space_to_canvas(as_pos, page) else {
        return false;
    };
    // **The SNAP tolerance, not the click tolerance** - and this was wrong
    // on the first driven run.
    //
    // The check reported `distance=23.1 tolerance=15.3` on the benchmark sheet:
    // the ring refused to close by eight canvas units, and the operator would
    // have clicked the corner they started at and got a fifth vertex on top of
    // it.
    //
    // The cause is not a sloppy hand, it is snapping. **The first vertex is
    // stored where the SNAP put it**, which on a dense drawing can be twenty
    // units from where the operator clicked - that is what snapping is for. So
    // the closing click is measured against a target that has already moved,
    // and the distance it may have moved by IS the snap tolerance. Using the
    // click tolerance asks the operator to hit a target more precisely than the
    // tool's own snapping placed it.
    //
    // ...and the RESOLVED point is compared as well, in page space. When the
    // closing click snaps to the same feature the first vertex snapped to, the
    // two are identical and the distance is exactly zero whatever the raw
    // pointer did. On real geometry that is the common case, and it is the one
    // that should feel effortless.
    #[allow(clippy::cast_possible_truncation)]
    let tolerance = map.snap_tolerance() as f32;
    #[allow(clippy::cast_possible_truncation)]
    let resolved = Pos2::new(picked.x as f32, picked.y as f32);
    #[allow(clippy::cast_possible_truncation)]
    let first_page = Pos2::new(first.x as f32, first.y as f32);
    let distance = first_canvas
        .distance(canvas_point)
        .min(first_page.distance(resolved));
    // Traced on every click, because "the ring did not close" has two
    // completely different causes and no screenshot can tell them apart: the
    // click was too far away (the operator missed), or the conversion is wrong
    // (the first vertex is not where it is drawn). The distance and the
    // tolerance side by side answer that in one line.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "measure-perimeter-ring-test distance={distance:.1} tolerance={tolerance:.1} \
             first_canvas={:.1},{:.1} click={:.1},{:.1}",
            first_canvas.x, first_canvas.y, canvas_point.x, canvas_point.y
        )
    });
    distance <= tolerance
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> PerimeterPick {
        let mut p = PerimeterPick::default();
        for (x, y) in [(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)] {
            p.push(Point::new(x, y));
        }
        p
    }

    /// **The closing segment is counted.** An open trace of the four
    /// corners of a square is three sides; closing it is four. Forgetting the
    /// closing segment is the exact hazard the PDF spec corpus names for
    /// `/Polygon`, and it would print a number one side short of the shape on
    /// screen.
    #[test]
    fn closing_the_ring_adds_the_closing_segment_to_the_total() {
        let mut p = square();
        assert!((p.length_points() - 300.0).abs() < 1e-9, "three sides open");
        assert!(p.close());
        assert!(
            (p.length_points() - 400.0).abs() < 1e-9,
            "four sides closed — the closing segment is not free"
        );
    }

    /// The two floors, and they are different numbers for a stated reason: a
    /// closed shape with two vertices traces a line there and back and would
    /// print twice the distance between two points.
    #[test]
    fn a_shape_too_small_to_be_one_authors_nothing() {
        let mut p = PerimeterPick::default();
        assert!(p.author().is_none(), "no picks is no shape");
        p.push(Point::new(0.0, 0.0));
        assert!(p.author().is_none(), "one pick is no shape");
        p.push(Point::new(10.0, 0.0));
        assert!(p.author().is_some(), "two picks is an open path");
        assert!(!p.close(), "…and two picks is NOT a ring");
        assert!(!p.is_closed());
    }

    /// Closing is the operator's act, never an inference. Two coincident
    /// vertices are a shape they drew, not a ring they meant.
    #[test]
    fn a_shape_is_never_closed_by_its_geometry() {
        let mut p = PerimeterPick::default();
        for (x, y) in [(0.0, 0.0), (50.0, 0.0), (50.0, 50.0), (0.0, 0.0)] {
            p.push(Point::new(x, y));
        }
        assert!(!p.is_closed(), "coincident first and last is not closed");
        let Some(DimensionKind::Perimeter { closed, .. }) = p.author() else {
            panic!("authors a perimeter");
        };
        assert!(!closed);
    }

    /// The preview is never drawn closed, because the click it is previewing
    /// does not close it. Showing the closing segment early would promise a
    /// shape one segment longer than the one about to be committed.
    #[test]
    fn the_preview_is_open_even_when_the_pick_is_about_to_be_closed() {
        let p = square();
        let Some(DimensionKind::Perimeter { points, closed, .. }) =
            p.preview(Point::new(-10.0, 50.0))
        else {
            panic!("previews a perimeter");
        };
        assert_eq!(points.len(), 5, "the pointer is a provisional vertex");
        assert!(!closed);
    }

    /// A committed pick is emptied, so a second Finish cannot author the same
    /// shape twice from a set the operator believes they have spent.
    #[test]
    fn clearing_forgets_the_ring_as_well_as_the_points() {
        let mut p = square();
        assert!(p.close());
        p.clear();
        assert!(p.points().is_empty());
        assert!(!p.is_closed(), "the flag is cleared with the points");
        assert!(p.author().is_none());
    }
}
