//! The points a ce dimension's vertex drag moves: a perimeter's corners and a
//! centre line's two ends, in page space, points.
//!
//! A linear ce dimension's picked points are not offered: its extension-line
//! grips (`canvas::dimext`) sit at those ends, and a vertex grip there would
//! outrank them in `pressing` and make the gap unreachable.

use pdfcer_core::dimension::{DimensionId, DimensionKind};
use pdfcer_core::vector::Point;

use crate::app::state::OpenDoc;
use crate::canvas::selection::{AnnotKind, SelectionState};

/// The selected ce dimension's record, of any kind.
pub(super) fn record(
    doc: &OpenDoc,
    selection: &SelectionState,
) -> Option<(DimensionId, DimensionKind)> {
    let annot = selection.annot()?;
    if annot.target.kind != AnnotKind::CeDimension {
        return None;
    }
    let model = doc.session.dimension_model();
    let record = model
        .dimensions()
        .iter()
        .find(|r| r.annot == Some(annot.target.id))?;
    Some((record.id, record.kind.clone()))
}

/// The selected ce dimension and its vertex-draggable points — a perimeter's
/// corners or a centre line's ends. A centre line is reached here and not
/// through [`super::selected`], whose `Some` offers a whole-object move it cannot
/// commit.
pub(super) fn reshapeable(
    doc: &OpenDoc,
    selection: &SelectionState,
) -> Option<(DimensionId, DimensionKind, Vec<Point>, bool)> {
    let (id, kind) = record(doc, selection)?;
    let (points, closed) = points(&kind)?;
    Some((id, kind, points, closed))
}

/// **Every draggable point of the selected ce dimension, in CANVAS space** —
/// a perimeter's corners or a centre line's ends — in index order.
#[must_use]
pub fn vertices(doc: &OpenDoc, selection: &SelectionState) -> Vec<egui::Pos2> {
    let Some((_, _, points, _)) = reshapeable(doc, selection) else {
        return Vec::new();
    };
    let Some(page) = doc.pages.get(doc.view.page_index) else {
        return Vec::new();
    };
    points
        .iter()
        .filter_map(|p| {
            #[allow(clippy::cast_possible_truncation)]
            let as_pos = egui::Pos2::new(p.x as f32, p.y as f32);
            crate::viewer::pdf_space_to_canvas(as_pos, page)
        })
        .collect()
}

/// `kind`'s draggable points in index order and whether they close; `None`
/// for a kind with none. The index is the one `move_dimension_vertex` takes.
pub(super) fn points(kind: &DimensionKind) -> Option<(Vec<Point>, bool)> {
    if let Some((points, closed)) = kind.polyline() {
        return Some((points.to_vec(), closed));
    }
    match *kind {
        DimensionKind::CentreLine { a, b } => Some((vec![a, b], false)),
        _ => None,
    }
}

/// The page-space segments `kind` draws with its points replaced by `moved`,
/// through the same function a committed ce dimension's preview uses. Empty
/// when `moved` does not fit the kind (a centre line is exactly two points).
pub(super) fn preview(kind: &DimensionKind, moved: &[Point], closed: bool) -> Vec<(Point, Point)> {
    let shaped = match (kind, moved) {
        (DimensionKind::CentreLine { .. }, &[a, b]) => DimensionKind::CentreLine { a, b },
        (DimensionKind::Perimeter { .. }, _) => DimensionKind::Perimeter {
            points: moved.to_vec(),
            closed,
            area: false,
            offset: 0.0,
            text_along: 0.0,
        },
        _ => return Vec::new(),
    };
    super::super::measure::pick::dimension_preview_segments(&shaped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_centre_line_offers_its_two_ends_open() {
        let (a, b) = (Point::new(1.0, 2.0), Point::new(30.0, 2.0));
        let line = DimensionKind::CentreLine { a, b };
        assert_eq!(points(&line), Some((vec![a, b], false)));
        let moved = [a, Point::new(30.0, 40.0)];
        assert_eq!(preview(&line, &moved, false), vec![(moved[0], moved[1])]);
    }

    #[test]
    fn a_centre_line_given_three_points_previews_nothing() {
        let line = DimensionKind::CentreLine {
            a: Point::new(0.0, 0.0),
            b: Point::new(1.0, 0.0),
        };
        let three = [Point::new(0.0, 0.0); 3];
        assert!(preview(&line, &three, false).is_empty());
    }
}
