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
/// How many tap areas [`Gimbal::zones`] returns: six tips and the centre.
pub(super) const ZONES: usize = 7;

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
        let tips = (0..ZONES - 1)
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
    /// The centre's area belongs to the centre: a tip's area is cut back to
    /// the side of it facing away from the centre, and a tip lying inside it
    /// (the axis the view looks along or straight away from) gets none. So no
    /// two areas overlap and the order callers search them in does not matter.
    pub fn zones(&self) -> Vec<Rect> {
        let centre = Rect::from_center_size(self.centre, Vec2::splat(HIT * 1.5));
        self.tips
            .iter()
            .map(|t| outside(Rect::from_center_size(t.at, Vec2::splat(HIT * 2.0)), centre))
            .chain(std::iter::once(centre))
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

/// `area` cut back to the side of `keep` its centre lies on, along the
/// direction it is furthest off; empty when its centre is inside `keep`.
fn outside(area: Rect, keep: Rect) -> Rect {
    if !area.intersects(keep) {
        return area;
    }
    if keep.contains(area.center()) {
        return Rect::NOTHING;
    }
    let off = area.center() - keep.center();
    let mut cut = area;
    if off.x.abs() >= off.y.abs() {
        if off.x > 0.0 {
            cut.min.x = keep.max.x;
        } else {
            cut.max.x = keep.min.x;
        }
    } else if off.y > 0.0 {
        cut.min.y = keep.max.y;
    } else {
        cut.max.y = keep.min.y;
    }
    cut
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gimbal(offsets: &[Vec2]) -> (Gimbal, egui::Pos2) {
        let centre = egui::pos2(100.0, 100.0);
        let tips = offsets
            .iter()
            .map(|&off| Tip {
                axis: [1.0, 0.0, 0.0],
                at: centre + off,
                depth: 0.0,
            })
            .collect();
        let radius = 50.0;
        (
            Gimbal {
                centre,
                tips,
                radius,
            },
            centre,
        )
    }

    fn hit(zones: &[Rect], p: egui::Pos2) -> Option<usize> {
        zones.iter().position(|z| z.contains(p))
    }

    #[test]
    fn a_tip_on_the_centre_leaves_the_centre_its_taps() {
        let (g, c) = gimbal(&[Vec2::ZERO, Vec2::new(40.0, 0.0)]);
        let zones = g.zones();
        assert_eq!(hit(&zones, c), Some(2));
        assert_eq!(hit(&zones, c + Vec2::new(40.0, 0.0)), Some(1));
    }

    #[test]
    fn a_tip_near_the_centre_keeps_only_its_outer_side() {
        let (g, c) = gimbal(&[Vec2::new(14.0, 3.0)]);
        let zones = g.zones();
        assert_eq!(
            hit(&zones, c + Vec2::new(8.0, 2.0)),
            Some(1),
            "inside the centre"
        );
        assert_eq!(
            hit(&zones, c + Vec2::new(20.0, 3.0)),
            Some(0),
            "the tip's far side"
        );
    }
}
