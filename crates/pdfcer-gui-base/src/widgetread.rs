//! **A widget's free turn**: the engine's `forms::Widget::free_angle` and
//! `free_size`, the size swapped into the `/Rect` frame under a quarter turn.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/widgetread.md`.

use pdfcer_core::forms::Widget;

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
/// The angle is the engine's `Widget::free_angle` and the size its
/// `free_size`, which is given before `/MK /R`'s swap and so is swapped here
/// under a quarter turn of 90 or 270.
#[must_use]
pub fn free_turn(widget: &Widget) -> Option<FreeTurn> {
    let degrees = widget.free_angle?;
    let (bw, bh) = widget.free_size?;
    let quarter = widget.rotation.unwrap_or(0).rem_euclid(360);
    let (w, h) = if quarter == 90 || quarter == 270 {
        (bh, bw)
    } else {
        (bw, bh)
    };
    Some(FreeTurn { degrees, w, h })
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
