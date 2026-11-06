//! The 3D viewer's measure tool: points picked on the model's surface become a
//! `ThreeDMeasurement`, built and valued by the engine. A tap is taken here while
//! the tool is on; a drag still steers the view. [`super::measure_paint`] draws
//! the result, [`super::measure_tab`] offers it and keeps it.
//!
//! Design: `docs/modules/pdfcer-gui/dialogs/model3d.md`.

use egui::{Pos2, Rect, Response};
use pdfcer_3d::vec3::{cross, length, normalize, sub};
use pdfcer_3d::{Hit, ModelUnits};
use pdfcer_3d_nav::{Orbit, Walk};
use pdfcer_core::threed::{ThreeDMeasureKind as Kind, ThreeDMeasurement};

use super::ModelView;
use crate::text::model3dmeasure as t;

/// The kinds offered, in [`t::KINDS`]' order.
pub(super) const KINDS: [Kind; 5] = [
    Kind::Linear,
    Kind::Perpendicular,
    Kind::Angular,
    Kind::Radial,
    Kind::Comment,
];
/// How near a triangle's corner a pick snaps to it, in points; a finger gets
/// the larger reach.
const SNAP: [f32; 2] = [10.0, 22.0];
/// A second tap this soon and this near is the same press of a double click:
/// seconds and points.
const SAME_TAP: (f64, f32) = (0.4, 4.0);
/// Below this sine a direction counts as along the view, so a plane facing the
/// camera would hold no measurement of it.
const ALONG_VIEW: f64 = 0.05;

/// A point picked on the model and the surface's normal there.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Pick {
    pub point: [f64; 3],
    pub normal: [f64; 3],
}

/// Why the last tap made nothing, said off the canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Said {
    Missed,
    Refused,
    NoComment,
    NotKept,
}

/// What the pointer's hover was last worked out for.
type HoverKey = (Pos2, Orbit, Option<Walk>);

/// The tool's state.
#[derive(Default)]
pub(super) struct Measure {
    pub on: bool,
    /// The kind chosen, an index in [`KINDS`].
    pub kind: usize,
    pub picked: Vec<Pick>,
    pub comment: String,
    /// A tap waiting for the picture's rectangle.
    pub tap: Option<Pos2>,
    last_tap: Option<(f64, Pos2)>,
    /// The finished measurement, not yet kept.
    pub result: Option<ThreeDMeasurement>,
    /// What the next pick would make, at the pointer.
    pub preview: Option<ThreeDMeasurement>,
    /// The point under the pointer.
    pub hover: Option<Pick>,
    hover_key: Option<HoverKey>,
    pub said: Option<Said>,
    /// A keep was queued this frame.
    pub keeping: bool,
    /// The document's edit epoch the saved views were last read at.
    pub epoch: Option<u64>,
    /// The number of measurements painted, as last traced.
    pub painted: usize,
}

impl Measure {
    /// The kind chosen.
    pub fn kind(&self) -> Kind {
        KINDS[self.kind.min(KINDS.len() - 1)]
    }

    /// Forget the picks and whatever they made.
    pub fn clear(&mut self) {
        self.picked.clear();
        self.result = None;
        self.preview = None;
        self.hover = None;
        self.hover_key = None;
        self.said = None;
    }

    /// Turn the tool on or off; off drops a measurement not kept.
    pub fn switch(&mut self, on: bool) {
        self.on = on;
        self.clear();
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| format!("model-measure-on on={on}"));
    }

    /// Take a tap on the picture, unless it is a double click's second press.
    pub fn take_tap(&mut self, now: f64, at: Pos2) {
        let again = self.last_tap.is_some_and(|(then, was)| {
            now - then <= SAME_TAP.0 && (at - was).length() <= SAME_TAP.1
        });
        self.last_tap = Some((now, at));
        if !again {
            self.tap = Some(at);
        }
    }

    /// The hover must be worked out afresh.
    pub fn stale_hover(&mut self) {
        self.hover_key = None;
    }
}

/// The picks `kind` takes.
pub(super) fn picks_needed(kind: Kind) -> usize {
    match kind {
        Kind::Linear | Kind::Perpendicular => 2,
        Kind::Angular | Kind::Radial => 3,
        _ => 1,
    }
}

/// A plane normal for a measurement along `along`: facing the camera along
/// `toward`, unless that looks down the measured direction, which would hold
/// nothing; then any normal square to `along`.
pub(super) fn plane_beside(along: [f64; 3], toward: [f64; 3]) -> [f64; 3] {
    let (Some(a), Some(v)) = (normalize(along), normalize(toward)) else {
        return toward;
    };
    if length(cross(a, v)) > ALONG_VIEW {
        return toward;
    }
    let axis = if a[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    cross(a, axis)
}

/// The measurement `picks` make, by the engine's builder for `kind`. `None`
/// where the builder refuses them or they are the wrong number.
pub(super) fn make(
    kind: Kind,
    picks: &[Pick],
    toward: [f64; 3],
    units: &str,
    comment: &str,
) -> Option<ThreeDMeasurement> {
    match (kind, picks) {
        (Kind::Linear, [a, b]) => {
            let plane = plane_beside(sub(b.point, a.point), toward);
            ThreeDMeasurement::linear(a.point, b.point, plane, units)
        }
        (Kind::Perpendicular, [a, b]) => {
            let plane = plane_beside(a.normal, toward);
            ThreeDMeasurement::perpendicular(a.point, b.point, a.normal, plane, units)
        }
        (Kind::Angular, [v, a, b]) => ThreeDMeasurement::angular(v.point, a.point, b.point, toward),
        (Kind::Radial, [a, b, c]) => ThreeDMeasurement::radial(a.point, b.point, c.point, units),
        (Kind::Comment, [a]) => ThreeDMeasurement::comment(a.point, comment.trim()),
        _ => None,
    }
}

/// The unit word a value carries: empty unless the model's file states the unit
/// and it is one with a word.
pub(super) fn units_word(units: Option<ModelUnits>) -> &'static str {
    units
        .filter(|u| u.from_cad_file)
        .map_or("", |u| t::unit_word(u.mm_per_unit))
}

/// A picture's width and height in whole points, as [`ModelView::hit_at`] asks
/// its rays.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(super) fn whole(rect: Rect) -> [u32; 2] {
    [rect.width(), rect.height()].map(|v| v.max(1.0).round() as u32)
}

/// A picture's width over its height.
pub(super) fn aspect_of(rect: Rect) -> f64 {
    f64::from(rect.width().max(1.0) / rect.height().max(1.0))
}

impl ModelView {
    /// Turn this frame's tap and hover into picks and a measurement.
    pub(super) fn measure_input(&mut self, ctx: &egui::Context, response: &Response, rect: Rect) {
        if !self.measure.on {
            return;
        }
        let reach = SNAP[usize::from(egui_shell::ribbon::phone::is_phone(ctx))];
        if let Some(at) = self.measure.tap.take() {
            self.pick_at(rect, at, reach);
        }
        self.measure_hover(response, rect, reach);
    }

    fn pick_at(&mut self, rect: Rect, at: Pos2, reach: f32) {
        let Some(hit) = self.hit_at(rect, at) else {
            self.measure.said = Some(Said::Missed);
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| "model-measure-missed".to_owned());
            return;
        };
        if self.measure.result.take().is_some() {
            self.measure.picked.clear();
        }
        self.measure.said = None;
        self.measure.stale_hover();
        let pick = self.pick_of(&hit, rect, at, reach);
        self.measure.picked.push(pick);
        let n = self.measure.picked.len();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-measure-pick n={n} at={:.6},{:.6},{:.6} snapped={}",
                pick.point[0],
                pick.point[1],
                pick.point[2],
                u8::from(pick.point != hit.point)
            )
        });
        if n >= picks_needed(self.measure.kind()) {
            self.finish_measure(rect);
        }
    }

    /// The picks are all in: build the measurement or say why not.
    fn finish_measure(&mut self, rect: Rect) {
        let picks = std::mem::take(&mut self.measure.picked);
        let built = self.built(&picks, rect);
        match built {
            Some(m) => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!(
                        "model-measure-result kind={:?} value={:.6} text={}",
                        m.kind,
                        m.value.unwrap_or(f64::NAN),
                        m.value_text().unwrap_or_default()
                    )
                });
                self.measure.result = Some(m);
                self.measure.picked = picks;
            }
            None => {
                let empty = self.measure.comment.trim().is_empty();
                self.measure.said = Some(if self.measure.kind() == Kind::Comment && empty {
                    Said::NoComment
                } else {
                    Said::Refused
                });
                // ui-text-exempt: diagnostic trace, never displayed
                crate::diag::trace(|| "model-measure-refused".to_owned());
            }
        }
    }

    /// The measurement `picks` make from the current view.
    fn built(&self, picks: &[Pick], rect: Rect) -> Option<ThreeDMeasurement> {
        let camera = self.view_camera(aspect_of(rect)).ok()?;
        let toward = sub(camera.eye, camera.target);
        let units = units_word(self.model.drawn.units);
        make(
            self.measure.kind(),
            picks,
            toward,
            units,
            &self.measure.comment,
        )
    }

    /// `hit` as a pick, on the triangle's corner when one is within `reach`.
    fn pick_of(&self, hit: &Hit, rect: Rect, at: Pos2, reach: f32) -> Pick {
        Pick {
            point: self
                .nearest_corner(hit, rect, at, reach)
                .unwrap_or(hit.point),
            normal: hit.normal,
        }
    }

    fn nearest_corner(&self, hit: &Hit, rect: Rect, at: Pos2, reach: f32) -> Option<[f64; 3]> {
        let mesh = self.model.drawn.meshes.get(hit.mesh)?;
        let corners = mesh.triangles.get(hit.triangle)?;
        let camera = self.view_camera(aspect_of(rect)).ok()?;
        let [w, h] = whole(rect);
        let (x, y) = (f64::from(at.x - rect.left()), f64::from(at.y - rect.top()));
        corners
            .iter()
            .filter_map(|c| {
                let point = *mesh.positions.get(*c as usize)?;
                let [px, py] = camera.pixel_of(point, w, h)?;
                let off = (px - x).hypot(py - y);
                (off <= f64::from(reach)).then_some((off, point))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, point)| point)
    }

    /// The point under the pointer and, with one pick left, what it would
    /// make; worked out only when the pointer or the camera moved.
    fn measure_hover(&mut self, response: &Response, rect: Rect, reach: f32) {
        let Some(at) = response.hover_pos() else {
            self.measure.hover = None;
            self.measure.preview = None;
            self.measure.stale_hover();
            return;
        };
        let key = (at, self.orbit, self.nav.walk);
        if self.measure.hover_key == Some(key) {
            return;
        }
        self.measure.hover_key = Some(key);
        let pick = self
            .hit_at(rect, at)
            .map(|hit| self.pick_of(&hit, rect, at, reach));
        self.measure.hover = pick;
        self.measure.preview = None;
        let picked = &self.measure.picked;
        let last = picked.len() + 1 == picks_needed(self.measure.kind());
        if let (Some(pick), true, None) = (pick, last, &self.measure.result) {
            let mut picks = picked.clone();
            picks.push(pick);
            self.measure.preview = self.built(&picks, rect);
            if let Some(m) = &self.measure.preview {
                // ui-text-exempt: diagnostic trace, never displayed
                crate::diag::trace(|| format!("model-measure-preview value={:?}", m.value));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_3d::vec3::dot;

    fn at(point: [f64; 3]) -> Pick {
        Pick {
            point,
            normal: [0.0, 0.0, 1.0],
        }
    }

    #[test]
    fn a_plane_faces_the_camera_unless_that_looks_down_the_measurement() {
        let toward = [0.0, 0.0, 5.0];
        assert_eq!(plane_beside([1.0, 0.0, 0.0], toward), toward);
        let square = plane_beside([0.0, 0.0, 3.0], toward);
        assert!(length(square) > 0.5);
        assert!(dot(square, [0.0, 0.0, 1.0]).abs() < 1e-9);
    }

    #[test]
    fn looking_down_a_distance_or_a_normal_still_measures() {
        let toward = [0.0, 0.0, 1.0];
        let picks = [at([0.0; 3]), at([0.0, 0.0, 4.0])];
        let linear = make(Kind::Linear, &picks, toward, "mm", "").unwrap();
        assert!((linear.value.unwrap() - 4.0).abs() < 1e-9);
        let perpendicular = make(Kind::Perpendicular, &picks, toward, "mm", "").unwrap();
        assert!((perpendicular.value.unwrap() - 4.0).abs() < 1e-9);
    }

    #[test]
    fn picks_that_make_nothing_make_nothing() {
        let toward = [0.0, 0.0, 1.0];
        let same = [at([1.0; 3]), at([1.0; 3])];
        assert!(make(Kind::Linear, &same, toward, "mm", "").is_none());
        let line = [at([0.0; 3]), at([1.0, 0.0, 0.0]), at([2.0, 0.0, 0.0])];
        assert!(make(Kind::Radial, &line, toward, "mm", "").is_none());
        assert!(make(Kind::Comment, &[at([0.0; 3])], toward, "", "  ").is_none());
        assert!(make(Kind::Linear, &[at([0.0; 3])], toward, "mm", "").is_none());
    }

    #[test]
    fn a_value_with_no_stated_unit_can_still_be_written() {
        let picks = [at([0.0; 3]), at([3.0, 4.0, 0.0])];
        let m = make(Kind::Linear, &picks, [0.0, 0.0, 1.0], "", "").unwrap();
        assert_eq!(m.value_text().as_deref(), Some("5.000"));
        let view = pdfcer_core::threed::ThreeDSavedView::new(
            "v",
            [1., 0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 5.],
        )
        .with_measurement(m);
        assert_eq!(view.measurements.len(), 1);
    }

    #[test]
    fn a_double_click_is_one_pick() {
        let mut measure = Measure::default();
        let spot = egui::pos2(10.0, 10.0);
        measure.take_tap(1.0, spot);
        assert_eq!(measure.tap.take(), Some(spot));
        measure.take_tap(1.2, spot);
        assert_eq!(measure.tap.take(), None);
        measure.take_tap(2.0, spot);
        assert_eq!(measure.tap.take(), Some(spot));
    }
}
