//! **A widget's free turn and opacity**, read from the file — the two values
//! `turn_widget` and `WidgetEdit::with_opacity` write and `forms::Widget`
//! does not carry (G177).
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/widgetread.md`.

use pdfcer_core::edit::EditSession;
use pdfcer_core::forms::Widget;
use pdfcer_core::graph::ObjectGraph;
use pdfcer_core::object::{Dict, Object};

/// Below this, two angles in degrees are the same angle.
const ANGLE_EPS: f64 = 1e-6;

/// `degrees` reduced into (-180, 180].
#[must_use]
pub fn normalise(degrees: f64) -> f64 {
    let r = degrees.rem_euclid(360.0);
    if r > 180.0 { r - 360.0 } else { r }
}

/// Whether `degrees` is a multiple of 90, zero included.
#[must_use]
pub fn is_quarter(degrees: f64) -> bool {
    let q = degrees / 90.0;
    (q - q.round()).abs() * 90.0 < ANGLE_EPS
}

/// A widget's free turn: the angle and the unturned box it turns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FreeTurn {
    /// Counterclockwise degrees in (-180, 180] on top of `/MK /R`; never a
    /// multiple of 90.
    pub degrees: f64,
    /// The unturned box's width in points, in the `/Rect` frame.
    pub w: f64,
    /// Its height.
    pub h: f64,
}

/// **The widget's free turn**, or `None` for an upright widget.
///
/// Read from the first normal appearance stream's `/Matrix` by
/// `EditSession::free_turn`'s rules: a pure rotation (no skew, no mirror),
/// less the `/MK /R` quarter turn, and a remainder that is itself a multiple
/// of 90 is no free turn. The size is the stream's `/BBox`, swapped under a
/// quarter turn of 90 or 270.
#[must_use]
pub fn free_turn(session: &EditSession, widget: &Widget) -> Option<FreeTurn> {
    let graph = session.graph();
    let Some(Object::Dict(dict)) = session.value(widget.id) else {
        return None;
    };
    let (m, bbox) = normal_appearance(&graph, dict)?;
    let quarter = widget.rotation.unwrap_or(0).rem_euclid(360);
    #[allow(clippy::cast_precision_loss)]
    let degrees = free_of(m, quarter as f64)?;
    let (bw, bh) = ((bbox[2] - bbox[0]).abs(), (bbox[3] - bbox[1]).abs());
    let (w, h) = if quarter == 90 || quarter == 270 {
        (bh, bw)
    } else {
        (bw, bh)
    };
    Some(FreeTurn { degrees, w, h })
}

/// The free angle a `/Matrix` states over `quarter`, or `None`.
fn free_of(m: [f64; 6], quarter: f64) -> Option<f64> {
    let [a, b, c, d, ..] = m;
    let rotation = (a - d).abs() < ANGLE_EPS
        && (b + c).abs() < ANGLE_EPS
        && (a.hypot(b) - 1.0).abs() < ANGLE_EPS;
    if !rotation {
        return None;
    }
    let free = (normalise(b.atan2(a).to_degrees() - quarter) * 1e9).round() / 1e9;
    (!is_quarter(free)).then_some(free)
}

/// The `/Matrix` (identity when absent) and `/BBox` of the widget's first
/// normal appearance stream.
fn normal_appearance(graph: &impl ObjectGraph, dict: &Dict) -> Option<([f64; 6], [f64; 4])> {
    let Object::Dict(ap) = graph.resolve(dict.get(b"AP")?) else {
        return None;
    };
    let stream = match graph.resolve(ap.get(b"N")?) {
        Object::Stream(s) => s,
        Object::Dict(states) => match graph.resolve(states.iter().next()?.1) {
            Object::Stream(s) => s,
            _ => return None,
        },
        _ => return None,
    };
    let bbox: [f64; 4] = numbers(graph, stream.dict.get(b"BBox")?)?.try_into().ok()?;
    let m = stream
        .dict
        .get(b"Matrix")
        .and_then(|o| numbers(graph, o))
        .and_then(|n| <[f64; 6]>::try_from(n).ok())
        .unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    Some((m, bbox))
}

/// An array of numbers, or `None`.
fn numbers(graph: &impl ObjectGraph, o: &Object) -> Option<Vec<f64>> {
    let Object::Array(items) = graph.resolve(o) else {
        return None;
    };
    items.iter().map(|o| graph.resolve(o).as_number()).collect()
}

/// **The widget's `/CA`**, or `None` when the key is absent (drawn opaque).
#[must_use]
pub fn opacity(session: &EditSession, widget: &Widget) -> Option<f64> {
    let graph = session.graph();
    let Some(Object::Dict(dict)) = session.value(widget.id) else {
        return None;
    };
    graph.resolve(dict.get(b"CA")?).as_number()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rot(deg: f64) -> [f64; 6] {
        let (s, c) = deg.to_radians().sin_cos();
        [c, s, -s, c, 0.0, 0.0]
    }

    #[test]
    fn a_free_angle_is_read_over_the_quarter_turn() {
        assert!((free_of(rot(30.0), 0.0).unwrap() - 30.0).abs() < 1e-6);
        assert!((free_of(rot(120.0), 90.0).unwrap() - 30.0).abs() < 1e-6);
        assert!((free_of(rot(-45.0), 0.0).unwrap() + 45.0).abs() < 1e-6);
    }

    #[test]
    fn a_quarter_turn_skew_or_mirror_is_no_free_angle() {
        assert_eq!(free_of(rot(90.0), 0.0), None);
        assert_eq!(free_of(rot(0.0), 0.0), None);
        assert_eq!(free_of([1.0, 0.0, 0.5, 1.0, 0.0, 0.0], 0.0), None);
        assert_eq!(free_of([-1.0, 0.0, 0.0, 1.0, 0.0, 0.0], 0.0), None);
    }

    #[test]
    fn angles_reduce_into_the_half_open_turn() {
        assert_eq!(normalise(190.0), -170.0);
        assert_eq!(normalise(-180.0), 180.0);
        assert!(is_quarter(270.0) && is_quarter(0.0) && !is_quarter(30.0));
    }
}
