//! The navigation gimbal: the model's three axes drawn from the camera in
//! the picture's top-right corner, each end a target to tap. A tap on an
//! axis end views the model from that side (orbiting) or faces that way
//! (walking); the centre fits the model (orbiting) or levels the look
//! (walking). It is a control over the picture, not model content.

use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use pdfcer_3d::Camera;

use super::orbit::{cross, dot, unit};
use crate::text::panels::modelnav as t;

/// The gimbal's side, in points, on a picture wide enough for it.
const SIDE: f32 = 112.0;
/// The gap from the picture's corner, in points.
const INSET: f32 = 8.0;
/// A tip's radius, in points; the hit area is larger, for a finger.
const TIP: f32 = 10.0;
const HIT: f32 = 16.0;

/// X, Y, Z: the colours every CAD program gives the axes.
const COLOURS: [Color32; 3] = [
    Color32::from_rgb(0xE0, 0x45, 0x45), // NOT A THEME COLOUR: the X axis's conventional red
    Color32::from_rgb(0x45, 0xB0, 0x45), // NOT A THEME COLOUR: the Y axis's conventional green
    Color32::from_rgb(0x45, 0x75, 0xE5), // NOT A THEME COLOUR: the Z axis's conventional blue
];

/// One end of an axis, placed on the picture.
#[derive(Debug, Clone, Copy)]
pub(super) struct Tip {
    /// The unit model axis, signed.
    pub axis: [f64; 3],
    pub at: Pos2,
    /// How far it points away from the viewer, -1 to 1.
    depth: f64,
}

/// The gimbal laid out for `camera` over `picture`.
#[derive(Debug, Clone)]
pub(super) struct Gimbal {
    pub centre: Pos2,
    pub tips: Vec<Tip>,
    radius: f32,
}

impl Gimbal {
    /// `None` on a picture too small to spare the corner, or for a camera
    /// with no direction.
    pub fn layout(picture: Rect, camera: &Camera) -> Option<Self> {
        let side = SIDE.min(picture.width() * 0.3).min(picture.height() * 0.4);
        if side < 64.0 {
            return None;
        }
        let forward = unit(std::array::from_fn(|i| camera.target[i] - camera.eye[i]))?;
        let right = unit(cross(forward, camera.up))?;
        let image_up = cross(right, forward);
        let centre = Pos2::new(
            picture.right() - INSET - side / 2.0,
            picture.top() + INSET + side / 2.0,
        );
        let arm = side / 2.0 - HIT;
        let tips = (0..6)
            .map(|k| {
                let sign = if k < 3 { 1.0 } else { -1.0 };
                let axis = std::array::from_fn(|i| if i == k % 3 { sign } else { 0.0 });
                #[allow(clippy::cast_possible_truncation)]
                let at =
                    centre + Vec2::new(dot(axis, right) as f32, -dot(axis, image_up) as f32) * arm;
                Tip {
                    axis,
                    at,
                    depth: dot(axis, forward),
                }
            })
            .collect();
        Some(Self {
            centre,
            tips,
            radius: side / 2.0,
        })
    }

    /// The tap areas: each tip in [`Gimbal::tips`] order, then the centre.
    pub fn zones(&self) -> Vec<Rect> {
        self.tips
            .iter()
            .map(|t| Rect::from_center_size(t.at, Vec2::splat(HIT * 2.0)))
            .chain(std::iter::once(Rect::from_center_size(
                self.centre,
                Vec2::splat(HIT * 1.5),
            )))
            .collect()
    }

    /// Draw it, the ends nearest the viewer last, so on top.
    pub fn paint(&self, painter: &egui::Painter, visuals: &egui::Visuals) {
        let backdrop = visuals.window_fill.gamma_multiply(0.6);
        painter.circle_filled(self.centre, self.radius, backdrop);
        painter.circle_stroke(self.centre, TIP * 0.6, visuals.widgets.inactive.fg_stroke);
        let mut order: Vec<usize> = (0..self.tips.len()).collect();
        order.sort_by(|&a, &b| self.tips[b].depth.total_cmp(&self.tips[a].depth));
        let font = egui::FontId::proportional(TIP * 1.2);
        for k in order {
            let tip = self.tips[k];
            let colour = COLOURS[k % 3];
            if k < 3 {
                painter.line_segment([self.centre, tip.at], Stroke::new(2.5, colour));
                painter.circle_filled(tip.at, TIP, colour);
                painter.text(
                    tip.at,
                    egui::Align2::CENTER_CENTER,
                    t::axis_letters()[k],
                    font.clone(),
                    visuals.extreme_bg_color,
                );
            } else {
                painter.circle_filled(tip.at, TIP * 0.7, backdrop);
                painter.circle_stroke(tip.at, TIP * 0.7, Stroke::new(2.0, colour));
            }
        }
    }
}
