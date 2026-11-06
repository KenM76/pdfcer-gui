//! The walking sticks, laid out and drawn as a drone's remote: a stick in
//! each bottom corner in DJI's Mode 2, and at the bottom centre a camera
//! stick two thirds their size between the buttons that centre the camera
//! left and right and up and down, with stair climbing's button over it.
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
/// A corner stick's radius bounds, points.
const RADIUS: (f32, f32) = (30.0, 64.0);
/// The camera stick's radius, as a share of a corner stick's.
const CAMERA_SHARE: f32 = 2.0 / 3.0;
/// The knob's radius, as a share of the stick's.
const KNOB: f32 = 0.36;

/// Where the sticks and buttons sit over one picture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Drone {
    /// The sticks' centres: [`LEFT`], [`RIGHT`], [`CAMERA`].
    pub centres: [Pos2; 3],
    /// The sticks' radii, in [`Drone::centres`] order.
    pub radii: [f32; 3],
    /// The buttons, in [`BUTTONS`] order: the centring pair either side of
    /// the camera stick, stair climbing over it.
    pub buttons: [Rect; 3],
}

impl Drone {
    /// The sticks over the picture `rect`, all on its bottom edge. The
    /// corner sticks are as large as leaves room between them for the
    /// camera stick and its two buttons, so the row fits across a phone
    /// held upright.
    pub fn layout(rect: Rect) -> Self {
        let middle = 2.0 * BUTTON + 4.0 * BUTTON_GAP;
        let radius = ((rect.width() - 2.0 * MARGIN - middle) / (4.0 + 2.0 * CAMERA_SHARE))
            .min(rect.height() / 5.0)
            .clamp(RADIUS.0, RADIUS.1);
        let camera = radius * CAMERA_SHARE;
        let low = rect.bottom() - MARGIN;
        let centre = Pos2::new(rect.center().x, low - camera);
        let beside = camera + BUTTON_GAP + BUTTON / 2.0;
        let button = |at: Pos2| Rect::from_center_size(at, Vec2::splat(BUTTON));
        Self {
            centres: [
                Pos2::new(rect.left() + MARGIN + radius, low - radius),
                Pos2::new(rect.right() - MARGIN - radius, low - radius),
                centre,
            ],
            radii: [radius, radius, camera],
            buttons: [
                button(centre - Vec2::new(beside, 0.0)),
                button(centre + Vec2::new(beside, 0.0)),
                button(centre - Vec2::new(0.0, beside)),
            ],
        }
    }

    /// Draw the sticks at `tilts` (x right, y up) and the buttons, Stairs
    /// lit when `stairs` is on. `level`, the left stick has no up and down
    /// (walking solid), and its base carries a sideways track saying so.
    pub fn paint(&self, ui: &Ui, painter: &Painter, tilts: [Vec2; 3], stairs: bool, level: bool) {
        let visuals = ui.visuals();
        let (accent, _) = egui_shell::theme::Theme::accent_pair(ui.ctx());
        let base = visuals.window_fill.gamma_multiply(0.45);
        let ring = visuals.widgets.inactive.fg_stroke;
        let sticks = self.centres.into_iter().zip(self.radii).zip(tilts);
        for (i, ((centre, radius), tilt)) in sticks.enumerate() {
            painter.circle_filled(centre, radius, base);
            painter.circle_stroke(centre, radius, ring);
            if level && i == LEFT {
                let track =
                    Rect::from_center_size(centre, Vec2::new(radius * 2.0, radius * KNOB * 2.0));
                painter.rect_filled(track, radius * KNOB, accent.gamma_multiply(0.25));
            }
            let knob = centre + Vec2::new(tilt.x, -tilt.y) * radius;
            painter.circle_filled(knob, radius * KNOB, accent.gamma_multiply(0.8));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_sticks_and_the_buttons_fit_a_phone_without_overlapping() {
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(411.0, 800.0));
        let d = Drone::layout(rect);
        let sticks: Vec<Rect> = d
            .centres
            .iter()
            .zip(d.radii)
            .map(|(c, r)| Rect::from_center_size(*c, Vec2::splat(r * 2.0)))
            .collect();
        for (i, a) in sticks.iter().enumerate() {
            assert!(rect.contains_rect(*a), "stick {i} {a:?}");
            for b in &sticks[i + 1..] {
                assert!(!a.intersects(*b), "{a:?} {b:?}");
            }
            for button in d.buttons {
                assert!(!a.intersects(button), "{a:?} {button:?}");
            }
        }
        assert!(d.radii[LEFT] >= 50.0, "{:?}", d.radii);
    }

    #[test]
    fn the_camera_stick_is_two_thirds_size_between_the_centring_buttons() {
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(1200.0, 800.0));
        let d = Drone::layout(rect);
        assert!((d.radii[CAMERA] - d.radii[LEFT] * 2.0 / 3.0).abs() < 1e-3);
        let camera = d.centres[CAMERA];
        let [yaw, pitch, stairs] = d.buttons;
        assert!(yaw.right() < camera.x - d.radii[CAMERA], "{yaw:?}");
        assert!(pitch.left() > camera.x + d.radii[CAMERA], "{pitch:?}");
        assert!((yaw.center().y - camera.y).abs() < 1e-3);
        assert!((pitch.center().y - camera.y).abs() < 1e-3);
        assert!(stairs.bottom() < camera.y - d.radii[CAMERA], "{stairs:?}");
        assert!((camera.y + d.radii[CAMERA] - (rect.bottom() - MARGIN)).abs() < 1e-3);
    }
}
