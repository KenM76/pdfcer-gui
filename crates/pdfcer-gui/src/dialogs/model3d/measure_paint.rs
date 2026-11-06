//! Draws measurements over the 3D picture from their stored anchors: one path
//! for a saved view's measurements and one not yet kept, so a measurement looks
//! the same before and after it is saved. Lines are sampled because a fisheye
//! bends them. The drawing is the viewer's overlay and is not in a saved picture.

use egui::{Color32, FontId, Painter, Pos2, Rect, Shape, Stroke, Ui, Vec2};
use pdfcer_3d::Camera;
use pdfcer_3d::vec3::{add, cross, dot, length, normalize, scale, sub};
use pdfcer_core::threed::{ThreeDMeasureKind as Kind, ThreeDMeasurement};

use super::ModelView;
use super::measure::{aspect_of, whole};

/// Samples along one straight line.
const LINE_STEPS: usize = 24;
/// Samples around a circle.
const CIRCLE_STEPS: usize = 72;
/// Samples along an angle's arc.
const ARC_STEPS: usize = 24;
/// The stroke's width and its dark backing's, in points.
const WIDTHS: [f32; 2] = [1.5, 3.5];
/// A picked point's radius, in points.
const DOT: f32 = 3.5;
/// The label's text size, in points.
const LABEL: f32 = 13.0;
/// How far above its anchor a label sits, in points.
const LABEL_LIFT: f32 = 8.0;
/// A measurement with no colour of its own.
const INK: Color32 = Color32::from_rgb(230, 60, 40); // NOT A THEME COLOUR: drawn over the model's own colours, not the window
/// What every stroke sits on, so it reads on any part.
const BACKING: Color32 = Color32::from_rgb(20, 20, 20); // NOT A THEME COLOUR: drawn over the model's own colours, not the window
/// The label's text.
const LABEL_INK: Color32 = Color32::from_rgb(245, 245, 245); // NOT A THEME COLOUR: drawn over the model's own colours, not the window

/// A camera and the picture it draws on.
struct Pen<'a> {
    painter: &'a Painter,
    camera: &'a Camera,
    origin: Pos2,
    size: [u32; 2],
    ink: Color32,
}

impl Pen<'_> {
    fn at(&self, point: [f64; 3]) -> Option<Pos2> {
        let [x, y] = self.camera.pixel_of(point, self.size[0], self.size[1])?;
        #[allow(clippy::cast_possible_truncation)]
        Some(self.origin + Vec2::new(x as f32, y as f32))
    }

    /// A path through `points`, broken wherever a point cannot be placed.
    fn path(&self, points: impl Iterator<Item = [f64; 3]>) {
        let mut run: Vec<Pos2> = Vec::new();
        for point in points {
            match self.at(point) {
                Some(p) => run.push(p),
                None => self.stroke(&mut run),
            }
        }
        self.stroke(&mut run);
    }

    fn stroke(&self, run: &mut Vec<Pos2>) {
        if run.len() >= 2 {
            let backing = Stroke::new(WIDTHS[1], BACKING);
            self.painter.add(Shape::line(run.clone(), backing));
            self.painter
                .add(Shape::line(run.clone(), Stroke::new(WIDTHS[0], self.ink)));
        }
        run.clear();
    }

    fn line(&self, from: [f64; 3], to: [f64; 3]) {
        self.path((0..=LINE_STEPS).map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let t = i as f64 / LINE_STEPS as f64;
            add(from, scale(sub(to, from), t))
        }));
    }

    fn dot(&self, point: [f64; 3]) {
        if let Some(p) = self.at(point) {
            self.painter.circle_filled(p, DOT + 1.0, BACKING);
            self.painter.circle_filled(p, DOT, self.ink);
        }
    }

    fn label(&self, at: [f64; 3], text: &str) {
        let Some(p) = self.at(at) else {
            return;
        };
        let galley =
            self.painter
                .layout_no_wrap(text.to_owned(), FontId::proportional(LABEL), LABEL_INK);
        // Beside the point rather than on it, which would hide a short line.
        let centre = p - Vec2::new(0.0, galley.size().y / 2.0 + LABEL_LIFT);
        let area = Rect::from_center_size(centre, galley.size()).expand(3.0);
        self.painter.rect_filled(area, 3.0, BACKING);
        self.painter
            .galley(area.min + Vec2::splat(3.0), galley, LABEL_INK);
    }
}

/// An angular measurement's corner: where its two sides, each from an anchor
/// back along its direction, come closest. `None` for parallel sides.
pub(super) fn vertex_of(m: &ThreeDMeasurement) -> Option<[f64; 3]> {
    let (a1, a2, d1, d2) = (m.a1?, m.a2?, m.d1?, m.d2?);
    let across = dot(d1, d2);
    let denominator = 1.0 - across * across;
    if denominator.abs() < 1e-12 {
        return None;
    }
    let gap = sub(a1, a2);
    let t = (dot(d1, gap) - across * dot(d2, gap)) / denominator;
    Some(sub(a1, scale(d1, t)))
}

/// A perpendicular measurement's foot: `a1` moved by the measured distance,
/// along the normal the plane and leader hold.
fn foot_of(m: &ThreeDMeasurement) -> Option<[f64; 3]> {
    let (a1, a2) = (m.a1?, m.a2?);
    let normal = normalize(cross(m.plane?, m.d1?))?;
    Some(add(a1, scale(normal, dot(sub(a2, a1), normal))))
}

fn draw_angle(pen: &Pen<'_>, m: &ThreeDMeasurement) {
    let (Some(a1), Some(a2), Some(d1), Some(d2), Some(corner)) =
        (m.a1, m.a2, m.d1, m.d2, vertex_of(m))
    else {
        return;
    };
    pen.line(corner, a1);
    pen.line(corner, a2);
    let radius = length(sub(a1, corner)).min(length(sub(a2, corner))) * 0.5;
    pen.path((0..=ARC_STEPS).filter_map(|i| {
        #[allow(clippy::cast_precision_loss)]
        let t = i as f64 / ARC_STEPS as f64;
        let way = normalize(add(scale(d1, 1.0 - t), scale(d2, t)))?;
        Some(add(corner, scale(way, radius)))
    }));
}

fn draw_circle(pen: &Pen<'_>, m: &ThreeDMeasurement) {
    let (Some(centre), Some(rim)) = (m.a1, m.a2) else {
        return;
    };
    pen.line(centre, rim);
    let spoke = sub(rim, centre);
    let Some(side) = m.plane.and_then(normalize).map(|n| cross(n, spoke)) else {
        return;
    };
    pen.path((0..=CIRCLE_STEPS).map(|i| {
        #[allow(clippy::cast_precision_loss)]
        let turn = std::f64::consts::TAU * i as f64 / CIRCLE_STEPS as f64;
        add(
            centre,
            add(scale(spoke, turn.cos()), scale(side, turn.sin())),
        )
    }));
}

fn draw(pen: &Pen<'_>, m: &ThreeDMeasurement) {
    match m.kind {
        Kind::Angular => draw_angle(pen, m),
        Kind::Radial => draw_circle(pen, m),
        Kind::Perpendicular => {
            if let (Some(a1), Some(a2), Some(foot)) = (m.a1, m.a2, foot_of(m)) {
                pen.line(a1, foot);
                pen.line(foot, a2);
            }
        }
        Kind::Comment => {
            if let Some(a1) = m.a1 {
                pen.dot(a1);
            }
        }
        _ => {
            if let (Some(a1), Some(a2)) = (m.a1, m.a2) {
                pen.line(a1, a2);
            }
        }
    }
    if let Some(text) = m.value_text().or_else(|| m.name.clone())
        && let Some(at) = m.text_anchor.or(m.a1)
    {
        pen.label(at, &text);
    }
}

/// `color` as the pen's ink.
fn ink_of(color: Option<[f64; 3]>) -> Color32 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    color.map_or(INK, |[r, g, b]| {
        let byte = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        // DOCUMENT COLOUR: the colour the file gives this measurement.
        Color32::from_rgb(byte(r), byte(g), byte(b))
    })
}

impl ModelView {
    /// The saved view's measurements, the one just made, and while measuring
    /// the picks and what the next would make.
    pub(super) fn paint_measures(&mut self, ui: &Ui, rect: Rect) {
        let saved = self
            .active_view()
            .and_then(|i| self.saved.views.get(i))
            .map_or(&[][..], |v| v.measurements.as_slice());
        let count = saved.len() + usize::from(self.measure.result.is_some());
        if count != self.measure.painted {
            self.measure.painted = count;
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| {
                let values: Vec<_> = saved.iter().filter_map(|m| m.value_text()).collect();
                format!(
                    "model-measure-drawn count={count} saved={}",
                    values.join("|")
                )
            });
        }
        if count == 0 && !self.measure.on {
            return;
        }
        let Ok(camera) = self.view_camera(aspect_of(rect)) else {
            return;
        };
        let painter = ui.painter().with_clip_rect(rect);
        let mut pen = Pen {
            painter: &painter,
            camera: &camera,
            origin: rect.min,
            size: whole(rect),
            ink: INK,
        };
        let drawn = saved.iter().chain(&self.measure.result);
        for m in drawn.chain(&self.measure.preview) {
            pen.ink = ink_of(m.color);
            draw(&pen, m);
        }
        pen.ink = INK;
        if self.measure.on {
            for pick in &self.measure.picked {
                pen.dot(pick.point);
            }
            if let Some(pick) = &self.measure.hover {
                pen.dot(pick.point);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_angle_finds_its_corner_again() {
        let corner = [1.0, 2.0, 3.0];
        let m =
            ThreeDMeasurement::angular(corner, [4.0, 2.0, 3.0], [1.0, 7.0, 3.0], [0.0, 0.0, 1.0])
                .unwrap();
        let found = vertex_of(&m).unwrap();
        assert!(length(sub(found, corner)) < 1e-9, "{found:?}");
    }

    #[test]
    fn a_perpendicular_foot_is_the_measured_distance_from_its_start() {
        let m = ThreeDMeasurement::perpendicular(
            [0.0; 3],
            [3.0, 0.0, 4.0],
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0],
            "mm",
        )
        .unwrap();
        let foot = foot_of(&m).unwrap();
        assert!((length(foot) - m.value.unwrap()).abs() < 1e-9);
        assert!((foot[2] - 4.0).abs() < 1e-9, "{foot:?}");
    }
}
