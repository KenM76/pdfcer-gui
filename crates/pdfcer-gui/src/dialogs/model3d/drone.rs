//! The walking sticks, laid out and drawn as a drone's remote: a stick in
//! each bottom corner in DJI's Mode 2, a camera stick at the bottom centre,
//! and under it the buttons that centre the camera and switch stair
//! climbing.
//!
//! Mode 2: the left stick's up and down rises and sinks and its left and
//! right turns the body; the right stick's up and down flies forward and
//! back and its left and right strafes. `navwalk` reads them.

use egui::{Painter, Pos2, Rect, Ui, Vec2};
use pdfcer_gui_base::icons::{self, Icon};

/// The left stick's index in [`Drone::centres`].
pub(super) const LEFT: usize = 0;
/// The right stick's.
pub(super) const RIGHT: usize = 1;
/// The camera stick's.
pub(super) const CAMERA: usize = 2;

/// The buttons, in [`Drone::buttons`] order.
pub(super) const BUTTONS: [Icon; 3] = [Icon::CentreYaw, Icon::CentrePitch, Icon::Stairs];
/// [`BUTTONS`]' indices.
pub(super) const CENTRE_YAW: usize = 0;
pub(super) const CENTRE_PITCH: usize = 1;
pub(super) const STAIRS: usize = 2;

/// The gap kept from the picture's edges and between parts, points.
const MARGIN: f32 = 16.0;
/// A button's side, points: a thumb's.
const BUTTON: f32 = 44.0;
/// The gap between buttons, points.
const BUTTON_GAP: f32 = 6.0;
/// A stick's radius bounds, points.
const RADIUS: (f32, f32) = (30.0, 64.0);
/// The knob's radius, as a share of the stick's.
const KNOB: f32 = 0.36;

/// Where the sticks and buttons sit over one picture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Drone {
    /// The sticks' centres: [`LEFT`], [`RIGHT`], [`CAMERA`].
    pub centres: [Pos2; 3],
    pub radius: f32,
    /// The buttons, in [`BUTTONS`] order, in a row under the camera stick.
    pub buttons: [Rect; 3],
}

impl Drone {
    /// The sticks over the picture `rect`: the corners' low, the camera's
    /// raised by the button row under it. Three sticks fit across a phone
    /// held upright.
    pub fn layout(rect: Rect) -> Self {
        let radius = ((rect.width() - 4.0 * MARGIN) / 6.0)
            .min(rect.height() / 5.0)
            .clamp(RADIUS.0, RADIUS.1);
        let low = rect.bottom() - MARGIN - radius;
        let row = BUTTON * 3.0 + BUTTON_GAP * 2.0;
        let first = Pos2::new(rect.center().x - row / 2.0, rect.bottom() - MARGIN - BUTTON);
        let buttons = std::array::from_fn(|i| {
            #[allow(clippy::cast_precision_loss)]
            let x = (BUTTON + BUTTON_GAP) * i as f32;
            Rect::from_min_size(first + Vec2::new(x, 0.0), Vec2::splat(BUTTON))
        });
        Self {
            centres: [
                Pos2::new(rect.left() + MARGIN + radius, low),
                Pos2::new(rect.right() - MARGIN - radius, low),
                Pos2::new(rect.center().x, first.y - MARGIN / 2.0 - radius),
            ],
            radius,
            buttons,
        }
    }

    /// Draw the sticks at `tilts` (x right, y up) and the buttons, Stairs
    /// lit when `stairs` is on.
    pub fn paint(&self, ui: &Ui, painter: &Painter, tilts: [Vec2; 3], stairs: bool) {
        let visuals = ui.visuals();
        let (accent, _) = egui_shell::theme::Theme::accent_pair(ui.ctx());
        let base = visuals.window_fill.gamma_multiply(0.45);
        let ring = visuals.widgets.inactive.fg_stroke;
        for (centre, tilt) in self.centres.into_iter().zip(tilts) {
            painter.circle_filled(centre, self.radius, base);
            painter.circle_stroke(centre, self.radius, ring);
            let knob = centre + Vec2::new(tilt.x, -tilt.y) * self.radius;
            painter.circle_filled(knob, self.radius * KNOB, accent.gamma_multiply(0.8));
        }
        for (i, (rect, icon)) in self.buttons.into_iter().zip(BUTTONS).enumerate() {
            let on = i == STAIRS && stairs;
            let fill = if on {
                accent.gamma_multiply(0.85)
            } else {
                visuals.window_fill.gamma_multiply(0.6)
            };
            painter.rect_filled(rect, 8.0, fill);
            let image = if on {
                icons::selected_image(ui, icon)
            } else {
                icons::image(ui, icon)
            };
            image.paint_at(ui, rect.shrink(8.0));
        }
    }
}

/// A stick's tilt shaped for fine control: a small dead zone, then the
/// square of the rest, so half a tilt is a quarter speed.
pub(super) fn shaped(tilt: f32) -> f64 {
    const DEAD: f32 = 0.08;
    let rest = ((tilt.abs() - DEAD) / (1.0 - DEAD)).max(0.0);
    f64::from(tilt.signum() * rest * rest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_sticks_and_the_buttons_fit_a_phone_without_overlapping() {
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(411.0, 800.0));
        let d = Drone::layout(rect);
        let reach = |c: Pos2| Rect::from_center_size(c, Vec2::splat(d.radius * 2.0));
        let sticks = d.centres.map(reach);
        for (i, a) in sticks.iter().enumerate() {
            assert!(rect.contains_rect(*a), "stick {i} {a:?}");
            for b in &sticks[i + 1..] {
                assert!(!a.intersects(*b), "{a:?} {b:?}");
            }
            for button in d.buttons {
                assert!(!a.intersects(button), "{a:?} {button:?}");
            }
        }
        assert!(d.radius >= 50.0, "{}", d.radius);
    }

    #[test]
    fn a_stick_ignores_a_resting_thumb_and_eases_in() {
        assert!(shaped(0.05).abs() < f64::EPSILON);
        assert!((shaped(1.0) - 1.0).abs() < 1e-6);
        assert!((shaped(-1.0) + 1.0).abs() < 1e-6);
        assert!(shaped(0.5) < 0.3);
    }
}
