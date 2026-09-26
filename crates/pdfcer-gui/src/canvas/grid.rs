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
const MIN_GRID_PITCH_PTS: f32 = 8.0;

/// The alpha, out of 255, of a **minor** grid line.
const GRID_MINOR_ALPHA: u8 = 26;

/// The alpha, out of 255, of a **major** grid line — the ones that coincide
/// with a numbered ruler tick.
const GRID_MAJOR_ALPHA: u8 = 56;

/// Draw the grid on every page the frame is showing.
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
