//! **A widget's free turn**: the engine's `forms::Widget::free_angle` with
//! the unturned box's size, which `forms::Widget` does not carry (G316) and
//! is read here from the appearance's `/BBox`.
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

/// **The widget's free turn**, or `None` for an upright widget or one whose
/// normal appearance has no readable `/BBox`.
///
/// The angle is the engine's `Widget::free_angle`. The size is the first
/// normal appearance stream's `/BBox`, swapped under a quarter turn of 90 or
/// 270.
#[must_use]
pub fn free_turn(session: &EditSession, widget: &Widget) -> Option<FreeTurn> {
    let degrees = widget.free_angle?;
    let graph = session.graph();
    let Some(Object::Dict(dict)) = session.value(widget.id) else {
        return None;
    };
    let bbox = normal_bbox(&graph, dict)?;
    let (bw, bh) = ((bbox[2] - bbox[0]).abs(), (bbox[3] - bbox[1]).abs());
    let quarter = widget.rotation.unwrap_or(0).rem_euclid(360);
    let (w, h) = if quarter == 90 || quarter == 270 {
        (bh, bw)
    } else {
        (bw, bh)
    };
    Some(FreeTurn { degrees, w, h })
}

/// The `/BBox` of the widget's first normal appearance stream.
fn normal_bbox(graph: &impl ObjectGraph, dict: &Dict) -> Option<[f64; 4]> {
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
    let Object::Array(items) = graph.resolve(stream.dict.get(b"BBox")?) else {
        return None;
    };
    let n: Vec<f64> = items
        .iter()
        .map(|o| graph.resolve(o).as_number())
        .collect::<Option<_>>()?;
    n.try_into().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angles_reduce_into_the_half_open_turn() {
        assert_eq!(normalise(190.0), -170.0);
        assert_eq!(normalise(-180.0), 180.0);
        assert!(is_quarter(270.0) && is_quarter(0.0) && !is_quarter(30.0));
    }
}
