//! # `canvas::grid` — the drawing grid, in the page's own space
//!
//! The middle of `RIBBON_IA.md` §5.2's *"Rulers · Grid · Guides"*, split out of
//! [`super::rulers`] when that file reached R2's 1,500-line ceiling. The seam
//! is the one that module's header already implied: a ruler is chrome **beside**
//! the canvas that reserves layout space and answers to R128, while a grid is
//! chrome **over the page** that reserves nothing and answers to a different
//! question entirely.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/grid.md`.

use egui::{Rect, Stroke, Ui};

use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::rulers::{Axis, Ladder, Scale};
use crate::canvas::strip::PageView;

/// The shortest on-screen distance, in logical points, between two **grid**
/// lines.
///
/// The grid uses the same ladder as the ruler but is allowed to be far denser,
/// because a grid line carries no text and its whole purpose is to be fine
/// enough to judge alignment against. 8 points is about where a hairline grid
/// stops reading as a grid and starts reading as a tint.
///
/// This is also the **line-count bound**, and the bound is what makes a
/// page-space grid affordable on a 129,758-object A3 sheet: at 8 points'
/// minimum pitch a 2,000-point-wide viewport holds at most 250 vertical lines,
/// whatever the page's size and whatever the zoom.
const MIN_GRID_PITCH_PTS: f32 = 8.0;

/// The alpha, out of 255, of a **minor** grid line.
///
/// Low, because a grid is a reference the operator looks *through*. The
/// standard is the one `overlay`'s find highlight records after a screenshot
/// corrected it: the operator's next act is to read the drawing, and chrome
/// that competes with the drawing has defeated its own purpose. On a dense CAD
/// sheet a grid at even a quarter opacity turns black linework grey.
const GRID_MINOR_ALPHA: u8 = 26;

/// The alpha, out of 255, of a **major** grid line — the ones that coincide
/// with a numbered ruler tick.
///
/// Two weights rather than one, and the ratio matters more than either number:
/// a uniform grid gives the eye nothing to count by, so judging "how far is
/// that" means counting hairlines one at a time. Every drafting grid in every
/// CAD package does this, and the heavy lines are the ones that line up with
/// the ruler's numbers.
const GRID_MAJOR_ALPHA: u8 = 56;

/// Draw the grid on every page the frame is showing.
///
/// One call from `super::interact`'s draw step, so the "which space"
/// decision this module exists to enact is made in exactly one place rather
/// than being spread across the canvas's page loop.
///
/// `clip` is the scroll viewport: the grid is confined to the intersection of
/// it with each page, which is both correct (there is no paper outside a page)
/// and what bounds the cost — see [`MIN_GRID_PITCH_PTS`].
pub(super) fn draw(ui: &Ui, doc: &OpenDoc, pages: &[PageView], clip: Rect) {
    let scale = Scale::of(doc);
    let ladder = Ladder::for_lines(scale, doc.view.zoom, MIN_GRID_PITCH_PTS);
    let base = ui.visuals().widgets.noninteractive.bg_stroke.color;
    // The grid's colours are the theme's ordinary hairline at two alphas.
    // Deliberately NOT the selection hue: a grid is not an affordance and
    // describes nothing the operator is about to act on, so borrowing the
    // colour that means "this is what a verb would touch" would say something
    // false — the same argument `overlay::draw_find_hits` makes for not
    // borrowing `warn_fg_color`.
    let minor = Stroke::new(1.0, super::overlay::at_alpha(base, GRID_MINOR_ALPHA));
    let major = Stroke::new(1.0, super::overlay::at_alpha(base, GRID_MAJOR_ALPHA));

    for view in pages {
        let visible = view.map.image_rect().intersect(clip);
        if visible.width() <= 0.0 || visible.height() <= 0.0 {
            continue;
        }
        let painter = ui.painter().with_clip_rect(visible);
        lines(&painter, Axis::X, view.map, visible, ladder, minor, major);
        lines(&painter, Axis::Y, view.map, visible, ladder, minor, major);
    }
}

/// Draw the grid lines spaced along one axis of one page.
///
/// Named for what it draws — a run of lines — rather than for the axis it is
/// given, so the parameter and the function do not share a name.
///
/// `Axis::X` produces **vertical** lines, spaced along the page's x. See
/// [`Axis`] on why one enum carries both readings.
fn lines(
    painter: &egui::Painter,
    axis: Axis,
    map: PageMapping,
    visible: Rect,
    ladder: Ladder,
    minor: Stroke,
    major: Stroke,
) {
    let (span, cross) = match axis {
        Axis::X => (visible.x_range(), visible.y_range()),
        Axis::Y => (visible.y_range(), visible.x_range()),
    };
    let from = f64::from(axis.of(map.to_page(axis.point(span.min))));
    let to = f64::from(axis.of(map.to_page(axis.point(span.max))));
    if !from.is_finite() || !to.is_finite() || to <= from || ladder.minor <= 0.0 {
        return;
    }

    for value in ladder.steps(from, to) {
        let at = axis.of(map.to_screen(axis.point(value as f32)));
        let stroke = if ladder.is_major(value) { major } else { minor };
        match axis {
            Axis::X => painter.vline(at, cross, stroke),
            Axis::Y => painter.hline(cross, at, stroke),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::rulers::MIN_MAJOR_PITCH_PTS;

    /// **Every grid line is at least [`MIN_GRID_PITCH_PTS`] apart on
    /// screen**, at every zoom on the ladder — the defect a measurement found
    /// after a screenshot did not.
    ///
    /// The bound applies to the **minor** step, because a minor line is a line
    /// that gets drawn. The version that bounded the *major* chose a 1-point
    /// grid on the benchmark A3 sheet — a line every 1.4 screen pixels, which
    /// is a tint rather than a grid, and ten times the shape count. It passed
    /// the old form of this test, which asserted only that the grid was finer
    /// than the ruler.
    ///
    /// The upper bound is asserted too. A grid whose lines drift to 80 points
    /// apart has stopped being something to judge alignment against, and the
    /// climb in [`Ladder::for_lines`] is exactly the code that could overshoot.
    #[test]
    fn every_grid_line_keeps_a_usable_pitch_at_every_zoom() {
        let scale = Scale::default();
        for &zoom in crate::viewer::ZOOM_LADDER {
            let grid = Ladder::for_lines(scale, zoom, MIN_GRID_PITCH_PTS);
            let pitch = grid.minor * f64::from(zoom);
            assert!(
                pitch >= f64::from(MIN_GRID_PITCH_PTS) - 1e-6,
                "at {zoom}× the grid lines are {pitch} pt apart — a tint, not a grid"
            );
            assert!(
                pitch <= f64::from(MIN_GRID_PITCH_PTS) * 5.0 + 1e-6,
                "at {zoom}× the grid lines are {pitch} pt apart, too far to judge against"
            );
        }
    }

    /// **Every numbered ruler tick has a grid line under it**, at every zoom
    /// on the ladder.
    ///
    /// The coincidence is the whole reason to ship a ruler and a grid rather
    /// than two independent ornaments: a feature sitting on a grid line can be
    /// read off the ruler without counting.
    ///
    /// The claim is deliberately *a grid line*, not *a heavy grid line*. Both
    /// ladders are 1-2-5 numbers and a 1-2-5 number is not always divisible by
    /// a smaller one — 500 over 200 is 2.5 — so the stronger statement is not
    /// true in general, and it was written as though it were. What is true, and
    /// what the operator actually needs, is that the ruler's labelled step is a
    /// whole number of grid *minors*; this asserts that, and that the grid is
    /// the finer of the two.
    #[test]
    fn every_ruler_label_has_a_grid_line_under_it() {
        let scale = Scale::default();
        for &zoom in crate::viewer::ZOOM_LADDER {
            let ruler = Ladder::for_labels(scale, zoom, MIN_MAJOR_PITCH_PTS);
            let grid = Ladder::for_lines(scale, zoom, MIN_GRID_PITCH_PTS);
            assert!(
                grid.minor <= ruler.major + 1e-9,
                "at {zoom}× the grid is coarser than the ruler's labelled step"
            );
            let ratio = ruler.major / grid.minor;
            assert!(
                (ratio - ratio.round()).abs() < 1e-6,
                "at {zoom}× a ruler label sits {ratio} grid lines along"
            );
        }
    }
}
