//! # `canvas::measure::perimeter` — **click around a shape; one number for the
//! whole way round**
//!
//! ## The operator's ask, verbatim
//!
//! > *"give me perimeter measuring tool as well where I click around to make a
//! > shape and it adds the distance of all the segments together for the
//! > dimension display. let me right click and add segments to the dimension.
//! > also I want to be able to edit the endpoints of the lines to adjust the
//! > shape. this should come with all the scaling options of the other
//! > dimensioning tools."*
//!
//! — Ken, 2026-08-20. He measures CAD site plans; a perimeter is a fence run, a
//! kerb line, a wall length. The last sentence is the one that decided the
//! design: *"all the scaling options"* means it must be a real
//! [`DimensionKind`] carried by a [`Group`], not a markup annotation with a
//! number typed into it — so that scale, unit, number format, drafting
//! standard, layer and the style cascade all come free rather than being
//! reimplemented badly.
//!
//! The engine agreed and shipped its whole half the same day it was filed.
//!
//! ## This tool is a HYBRID of the two that already exist, and that is why it
//! ## was cheap to write
//!
//! | | picks | how it ends |
//! |---|---|---|
//! | [`MeasureKind::Linear`] | **points**, with snapping | a fixed arity — three clicks and it is done |
//! | [`MeasureKind::Circular`] | **objects** | **open-ended** — double-click, or the `measure.finish` command |
//! | **this** | **points**, with snapping | **open-ended**, plus a third ending of its own |
//!
//! So the point resolution — the snap query, the derived-candidate two-click
//! confirm, the operator's snap master toggle — is [`super::click`]'s existing
//! machinery, untouched, and this module is only asked *"what does one resolved
//! point mean?"*. And the ending is [`super::circular`]'s answer, already
//! settled with the operator on 2026-08-14: a **double-click**, because that is
//! what every polyline tool in every drawing package uses, plus a ribbon
//! command for a pick that is awkward to double-click on.
//!
//! ## ★ The third ending: click the first vertex to CLOSE
//!
//! A perimeter has a shape the other two do not — it can be a *ring*. Ken's
//! words were *"click around to make a shape"*, which is a closed one; a path
//! length (a pipe run, a cable route) is the same gesture left open.
//!
//! Rather than a modifier or a toggle nobody would find, the convention every
//! drawing package uses: **clicking the first vertex closes the shape**. It is
//! discoverable by accident, it is what a hand does anyway when tracing a
//! footprint, and it makes the two shapes one tool instead of two.
//!
//! The hit test for "the first vertex" is in **canvas space** at the same
//! tolerance a selecting click uses, so it is the same physical target size at
//! every zoom. Doing it in page space would make the ring impossible to close
//! when zoomed out and trivially easy to close by accident when zoomed in.
//!
//! ## What is NOT here
//!
//! **Vertex editing** — dragging a corner, right-clicking a segment to add one,
//! right-clicking a vertex to remove one. That is editing a *committed*
//! dimension, so it belongs beside [`crate::canvas::dimdrag`] with the rest of
//! the after-the-fact editing, not in the tool that authors it. The engine's
//! verbs for it (`move_dimension_vertex`, `insert_dimension_vertex`,
//! `remove_dimension_vertex`, and `vertex_edit_preview` so a menu can be greyed
//! correctly) all exist. Recorded here so the gap is named rather than implied.

use egui::Pos2;

use pdfcer_core::dimension::DimensionKind;
use pdfcer_core::vector::Point;

use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;
use crate::canvas::mapping::PageMapping;

use super::state::MeasureState;

/// The picks made so far, and whether the operator has closed the ring.
///
/// # Why the vertices live here and not in [`crate::canvas::selection`]
///
/// The same rule [`super::pick::CircularPick`] states and for the same reason:
/// a half-traced outline is **not a selection**. No verb on the Format tab
/// means anything applied to it, Delete least of all, and borrowing the
/// selection to hold it would arm a destructive control over a set the operator
/// assembled for a completely different purpose.
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
///
/// pdfcer policy, and the engine labels it as such: ISO 32000-1 §12.5.6.9 states
/// no minimum, no maximum and no degenerate-case behaviour at all. Two points
/// is one segment, which is a length.
pub const MIN_OPEN: usize = 2;

/// The fewest vertices a **closed** perimeter can have.
///
/// Also policy. A closed shape with two vertices traces a line there and back:
/// one stroke on screen, printing twice the distance between two points — a
/// number that disagrees with the picture, which is the one thing this
/// subsystem exists to prevent.
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
    ///
    /// No de-duplication: a repeated point contributes a zero-length segment,
    /// which is invisible rather than wrong and which the operator removes by
    /// dragging the vertex after the fact. Refusing it would mean this tool
    /// silently discarding a click, which reads as the tool being broken.
    pub fn push(&mut self, p: Point) {
        self.points.push(p);
    }

    /// Close the ring, reporting whether it could be closed.
    ///
    /// Refuses below [`MIN_CLOSED`], and refuses a second close — an already
    /// closed pick has been committed and emptied, so reaching here twice would
    /// mean the state machine has slipped.
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
    ///
    /// The single place a `DimensionKind` is built for this tool, so the
    /// preview and the commit cannot describe different shapes — the standing
    /// rule in [`super`], and the reason [`super::circular::commit`] exists as
    /// one function reached by two endings.
    ///
    /// `offset` and `text_along` are zero: the label starts at the vertex
    /// centroid, and moving it from there is a [`crate::canvas::dimdrag`] drag
    /// afterwards rather than a fourth thing to get right during authoring.
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
    ///
    /// Used by the live preview and by nothing else. The provisional vertex is
    /// appended rather than replacing anything, so the rubber band runs from
    /// the last committed pick to the pointer — which is the picture every
    /// polyline tool draws and the one that says *"this click would add this
    /// segment"*.
    ///
    /// `None` before the first pick: there is no shape yet and drawing a
    /// zero-length segment at the pointer would be a mark that means nothing.
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
    ///
    /// Page points, deliberately — this is the raw measurement, and turning it
    /// into the operator's units is [`crate::text::measure`]'s job through the
    /// group's scale and number format. Two places that both applied the scale
    /// would double it, and one that applied it here would put a unit-aware
    /// number in a geometry function.
    #[must_use]
    pub fn length_points(&self) -> f64 {
        let mut total = self
            .points
            .windows(2)
            .map(|w| (w[1].x - w[0].x).hypot(w[1].y - w[0].y))
            .sum::<f64>();
        // ★ The closing segment is added HERE, and forgetting it is the hazard
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

/// **End the gesture: author the dimension and empty the pick.**
///
/// ★ The one commit path, reached by all three endings — closing the ring,
/// double-clicking, and the `measure.finish` command. That is the same argument
/// [`super::circular::commit`] makes and it matters more here, because there
/// are three doors rather than two: three places each building a
/// `DimensionKind` is three chances for one of them to forget the `closed`
/// flag.
///
/// Pure over the state and the action list — no `egui`, no context, no memory —
/// which is what makes every ending assertable without a window.
///
/// Returns `false` and raises nothing when there is not enough shape to author.
pub(super) fn commit(st: &mut MeasureState, page_index: usize, actions: &mut Vec<Action>) -> bool {
    let Some(kind) = st.perimeter.author() else {
        return false;
    };
    actions.push(Action::Dimension(DimensionAction::Commit {
        page: page_index,
        group: st.group,
        kind,
        // Nothing to disclose. Every vertex is one the operator clicked and the
        // shape on screen is the shape being authored — there is no inference
        // here to own up to, which is the difference from the circular tool's
        // best-fit residual.
        disclosures: Vec::new(),
    }));
    st.perimeter.clear();
    true
}

/// **Take one resolved point for the perimeter tool**, and answer the three
/// endings.
///
/// Called from [`super::click`]'s match, *after* the point has been through the
/// snap query and the derived-candidate confirm — unlike
/// [`super::circular::click`], which runs before all that because it picks
/// objects rather than points. A perimeter's vertices want snapping as much as
/// a linear dimension's do: an operator tracing a building footprint is aiming
/// at the corners of paths that are already on the page.
///
/// # The order of the three questions, and why it is this order
///
/// 1. **A double-click ends it open.** Asked first because the pair's *first*
///    click has already been through here as an ordinary pick and has already
///    added its vertex — see [`super::click`]'s note on why that is the right
///    reading of how `egui` reports a double-click rather than an accident of
///    it. So by the time this fires, the shape is complete and the second click
///    must not add a duplicate vertex on top of the last one.
/// 2. **A click on the first vertex closes the ring.** Asked before the
///    ordinary pick, because that point is also a perfectly good place to put a
///    vertex and the operator who clicks it means the ring — this is the whole
///    of the convention.
/// 3. **Otherwise it is a vertex.**
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

pub(super) fn click(st: &mut MeasureState, c: Click<'_>, actions: &mut Vec<Action>) {
    let Click {
        page_index,
        picked,
        canvas_point,
        double,
        page,
        map,
    } = c;
    if double {
        if !commit(st, page_index, actions) {
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

    // ★★ **The Length tool never closes**, and the guard is here rather than
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
        if commit(st, page_index, actions) {
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
///
/// Compared in **canvas space** at the same tolerance a selecting click uses,
/// so the target is the same physical size at every zoom. In page space the
/// ring would be impossible to close zoomed out — the first vertex would be a
/// sub-pixel target — and trivially easy to close by accident zoomed in.
///
/// Requires [`MIN_CLOSED`] vertices before it will answer `true`. Below that
/// there is no ring to close, and reading a click on the first of two vertices
/// as a close would consume a pick and then refuse, which from the operator's
/// chair is a click that did nothing.
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
    // ★★ **The SNAP tolerance, not the click tolerance** - and this was wrong
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
    // ★ Traced on every click, because "the ring did not close" has two
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

    /// ★★ **The closing segment is counted.** An open trace of the four
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

    /// ★ The preview is never drawn closed, because the click it is previewing
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
