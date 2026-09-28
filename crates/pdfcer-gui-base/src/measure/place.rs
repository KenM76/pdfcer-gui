//! Where a ce dimension lands when the operator clicks to place it.
//!
//! Every measure gesture ends in a placing step: once the picks say *what* is
//! measured, the dimension follows the pointer and the next click says
//! *where*. The click point is also where the value text goes, so one
//! function resolves a pointer into each kind's own placement fields, and the
//! live preview and the commit both call it.

use pdfcer_core::dimension::DimensionKind;
use pdfcer_core::vector::Point;

/// A measured ce dimension waiting for its placing click.
#[derive(Debug, Clone, PartialEq)]
pub struct Placing {
    /// The dimension as the gesture authored it; only its placement fields
    /// change when it is placed.
    pub kind: DimensionKind,
    /// What the gesture inferred, carried to the commit unchanged.
    pub disclosures: Vec<String>,
}

impl Placing {
    /// The dimension a placing click at `p` commits.
    #[must_use]
    pub fn at(&self, p: Point) -> DimensionKind {
        placed_at(&self.kind, p)
    }
}

/// `kind` placed so its value text sits at page-space point `p`.
///
/// The measured geometry is never touched — only where the dimension is
/// drawn — so the value cannot change with where it is dropped.
///
/// - **Linear**: perpendicular standoff and the text's slide along the line,
///   from `placement_from_point`.
/// - **Perimeter**: the text's displacement from the vertex centroid.
/// - **Circular**: the leader turns to face `p` and the text sits at `p`'s
///   distance from the centre, clamped as the engine clamps it.
/// - **Angular**: the arc's radius is `p`'s distance from the apex. The
///   engine draws the text at the arc's chord midpoint whatever `text_along`
///   holds, so only the radius follows the click (request G064).
#[must_use]
pub fn placed_at(kind: &DimensionKind, p: Point) -> DimensionKind {
    let mut out = kind.clone();
    match &mut out {
        DimensionKind::Linear {
            offset, text_along, ..
        }
        | DimensionKind::Perimeter {
            offset, text_along, ..
        } => {
            if let Some((o, t)) = kind.placement_from_point(p) {
                *offset = o;
                *text_along = t;
            }
        }
        DimensionKind::Circular {
            leader_angle,
            text_distance,
            ..
        } => {
            if let Some((distance, angle)) = kind.placement_from_point(p) {
                *leader_angle = angle;
                *text_distance = Some(distance);
            }
            let clamped = out.circular_text_distance();
            if let DimensionKind::Circular { text_distance, .. } = &mut out {
                *text_distance = clamped.or(*text_distance);
            }
        }
        DimensionKind::Angular { apex, radius, .. } => {
            let r = (p.x - apex.x).hypot(p.y - apex.y);
            if r.is_finite() && r > MIN_ARC_RADIUS_PT {
                *radius = r;
            }
        }
    }
    out
}

/// The smallest arc radius a placing click sets, in points. A click on the
/// apex itself has no radius to give; the arc keeps the one it had.
const MIN_ARC_RADIUS_PT: f64 = 1.0;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::float_cmp)]
mod tests {
    use super::*;
    use pdfcer_core::dimension::FitCircle;
    use pdfcer_core::vector::AxisConstraint;

    fn p(x: f64, y: f64) -> Point {
        Point::new(x, y)
    }

    fn close(a: Point, b: Point) -> bool {
        (a.x - b.x).abs() < 1e-6 && (a.y - b.y).abs() < 1e-6
    }

    #[test]
    fn a_linear_dimension_puts_its_text_where_the_click_was() {
        let kind = DimensionKind::Linear {
            a: p(0.0, 0.0),
            b: p(100.0, 0.0),
            constraint: AxisConstraint::Aligned,
            offset: 0.0,
            text_along: 0.0,
            extension_gap: [None; 2],
        };
        let placed = placed_at(&kind, p(70.0, 25.0));
        let anchor = placed.label_anchor().unwrap();
        assert!(close(anchor, p(70.0, 25.0)), "text at {anchor:?}");
        assert_eq!(placed.measured_points(), kind.measured_points());
    }

    #[test]
    fn a_perimeter_puts_its_text_where_the_click_was() {
        let kind = DimensionKind::Perimeter {
            points: vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)],
            closed: true,
            offset: 0.0,
            text_along: 0.0,
        };
        let placed = placed_at(&kind, p(40.0, -8.0));
        assert!(close(placed.label_anchor().unwrap(), p(40.0, -8.0)));
    }

    #[test]
    fn a_circular_dimension_puts_its_text_where_the_click_was() {
        let kind = DimensionKind::Circular {
            fit: FitCircle {
                center: p(50.0, 50.0),
                radius: 10.0,
                residual: 0.0,
            },
            show_diameter: false,
            leader_angle: 0.0,
            text_distance: None,
        };
        let placed = placed_at(&kind, p(50.0, 80.0));
        assert!(close(placed.label_anchor().unwrap(), p(50.0, 80.0)));
    }

    #[test]
    fn a_click_inside_a_radius_stops_the_text_at_the_centre() {
        let kind = DimensionKind::Circular {
            fit: FitCircle {
                center: p(0.0, 0.0),
                radius: 10.0,
                residual: 0.0,
            },
            show_diameter: false,
            leader_angle: 0.0,
            text_distance: None,
        };
        let DimensionKind::Circular { text_distance, .. } = placed_at(&kind, p(0.0, 0.0)) else {
            panic!("still circular");
        };
        assert_eq!(text_distance, Some(-10.0));
    }

    #[test]
    fn an_angular_arc_passes_through_the_click() {
        let kind = DimensionKind::Angular {
            apex: p(0.0, 0.0),
            dir_a: p(1.0, 0.0),
            dir_b: p(0.0, 1.0),
            radius: 5.0,
            text_along: 0.0,
        };
        let DimensionKind::Angular { radius, .. } = placed_at(&kind, p(30.0, 40.0)) else {
            panic!("still angular");
        };
        assert_eq!(radius, 50.0);
        let DimensionKind::Angular { radius, .. } = placed_at(&kind, p(0.0, 0.0)) else {
            panic!("still angular");
        };
        assert_eq!(radius, 5.0, "a click on the apex keeps the arc");
    }
}
