//! The Measure tab's surface measure: two picked surfaces are measured by the
//! engine's `measure_surfaces`, a distance when their planes are parallel
//! within the operator's tolerance (or he forces one), the angle otherwise,
//! and the angle said either way. What each fitted plane rests on is said
//! with the result, off the picture. Nothing is written to the file.

use egui::{RichText, Ui};
use pdfcer_3d::vec3::{dot, scale, sub};
use pdfcer_3d::{
    DEFAULT_PARALLEL_DEGREES, Hit, SurfaceMeasure, SurfacePick, SurfaceRelation, SurfaceTolerance,
    TriangleMesh, measure_surfaces,
};

use super::ModelView;
use super::measure::{SURFACES, Said, units_word};
use super::measure_tab::REGION_START_AGAIN;
use crate::text::model3dmeasure as t;

/// The surface measure's button in the kind row.
pub const REGION_SURFACES: &str = "model3d.measure.surfaces"; // ui-text-exempt: trace region name, never displayed
/// The parallel tolerance.
pub const REGION_PARALLEL: &str = "model3d.surface.parallel"; // ui-text-exempt: trace region name, never displayed
/// *Always measure a distance*.
pub const REGION_FORCE: &str = "model3d.surface.force"; // ui-text-exempt: trace region name, never displayed

/// The headline's text size, in points.
const VALUE_SIZE: f32 = 16.0;
/// A row that says nothing and keeps its height.
const BLANK: &str = " "; // ui-text-exempt: an empty row, no words

/// A measurement, or the engine's reason there is none.
type Answer = Result<SurfaceMeasure, String>;

/// The surface measure's state; the tolerance outlives the picks.
#[derive(Debug, Clone)]
pub(super) struct Surfaces {
    /// Planes this near parallel, in degrees, are measured for distance.
    pub parallel_degrees: f64,
    pub force: bool,
    picks: Vec<SurfacePick>,
    answer: Option<Answer>,
}

impl Default for Surfaces {
    fn default() -> Self {
        Self {
            parallel_degrees: DEFAULT_PARALLEL_DEGREES,
            force: false,
            picks: Vec::new(),
            answer: None,
        }
    }
}

impl Surfaces {
    /// Forget the picks and the measurement.
    pub(super) fn clear(&mut self) {
        self.picks.clear();
        self.answer = None;
    }

    fn holds(&self) -> bool {
        !self.picks.is_empty() || self.answer.is_some()
    }

    /// The picked points, to mark on the picture.
    pub(super) fn marks(&self) -> impl Iterator<Item = [f64; 3]> + '_ {
        self.picks.iter().map(|p| p.point)
    }

    /// A distance's line: from the second point to its foot on the first
    /// surface's plane.
    pub(super) fn leader(&self) -> Option<([f64; 3], [f64; 3])> {
        let (Some(Ok(m)), [_, b]) = (&self.answer, self.picks.as_slice()) else {
            return None;
        };
        let SurfaceRelation::Parallel { .. } = m.relation else {
            return None;
        };
        let along = dot(sub(b.point, m.a.centroid), m.a.normal);
        Some((b.point, sub(b.point, scale(m.a.normal, along))))
    }

    fn tolerance(&self) -> SurfaceTolerance {
        let mut tolerance = SurfaceTolerance::default();
        tolerance.parallel_degrees = self.parallel_degrees;
        tolerance.force_parallel = self.force;
        tolerance
    }

    /// Take a pick; the second asks the engine. A third starts again.
    fn pick(&mut self, meshes: &[TriangleMesh], hit: &Hit) {
        if self.picks.len() >= 2 {
            self.clear();
        }
        self.picks.push(SurfacePick::from(hit));
        let n = self.picks.len();
        crate::diag::trace(|| {
            let ([x, y, z], [i, j, k]) = (hit.point, hit.normal);
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-surface-pick n={n} mesh={} triangle={} at={x:.6},{y:.6},{z:.6} normal={i:.6},{j:.6},{k:.6}",
                hit.mesh, hit.triangle
            )
        });
        self.measure(meshes);
    }

    /// Measure the two picks, when there are two.
    fn measure(&mut self, meshes: &[TriangleMesh]) {
        let [a, b] = self.picks.as_slice() else {
            return;
        };
        let answer = measure_surfaces(meshes, a, b, &self.tolerance()).map_err(|e| e.to_string());
        trace(&answer, self.parallel_degrees);
        self.answer = Some(answer);
    }
}

fn trace(answer: &Answer, tolerance: f64) {
    crate::diag::trace(|| match answer {
        Ok(m) => {
            let (relation, distance, forced) = match m.relation {
                SurfaceRelation::Parallel { distance, forced } => {
                    ("parallel", format!("{distance:.6}"), u8::from(forced))
                }
                _ => ("angled", "-".to_owned(), 0),
            };
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-surface-result relation={relation} distance={distance} forced={forced} \
                 angle={:.6} normals={:.6} tolerance={tolerance:.6} tri-a={} tri-b={} \
                 whole-a={} whole-b={} faces={} flat-a={:.6} flat-b={:.6} pick-distance={:.6}",
                m.angle_degrees,
                m.normals_degrees,
                m.a.triangles,
                m.b.triangles,
                u8::from(m.a.whole_face),
                u8::from(m.b.whole_face),
                u8::from(m.a.face.is_some() && m.b.face.is_some()),
                m.a.flatness_degrees,
                m.b.flatness_degrees,
                m.pick_distance,
            )
        }
        // ui-text-exempt: diagnostic trace, never displayed
        Err(why) => format!("model-surface-refused why={why}"),
    });
}

/// The headline, how the planes lie, and what they were fitted from.
fn lines(m: &SurfaceMeasure, tolerance: f64, unit: &str) -> [String; 3] {
    let (head, forced) = match m.relation {
        SurfaceRelation::Parallel { distance, forced } => {
            (t::surface_distance(distance, unit), Some(forced))
        }
        _ => (t::surface_angle(m.angle_degrees), None),
    };
    let relation = t::surface_relation(m.angle_degrees, tolerance, forced);
    let partial = usize::from(!m.a.whole_face) + usize::from(!m.b.whole_face);
    let faces = m.a.face.is_some() && m.b.face.is_some();
    let fit = t::surface_fit([m.a.triangles, m.b.triangles], partial, faces);
    [head, relation, fit]
}

/// A row of one line that never wraps, its whole text on hover, so the
/// readout keeps its height.
pub(super) fn row(ui: &mut Ui, text: &str) {
    let label = ui.add(egui::Label::new(RichText::new(text).small()).truncate());
    if text != BLANK {
        label.on_hover_text(text);
    }
}

impl ModelView {
    /// A click on the model while the surface measure is chosen.
    pub(super) fn surface_pick(&mut self, hit: &Hit) {
        self.measure.surfaces.pick(&self.model.drawn.meshes, hit);
    }

    /// The surface measure's button, after the light meter.
    pub(super) fn surface_button(&mut self, ui: &mut Ui) {
        let chosen = self.measure.kind == SURFACES;
        let button = ui
            .add(egui::Button::new(t::SURFACES).selected(chosen))
            .on_hover_text(t::SURFACES_TIP);
        crate::diag::ui_rect_visible(REGION_SURFACES, button.rect, ui.clip_rect());
        if button.clicked() && !chosen {
            self.measure.kind = SURFACES;
            self.measure.clear();
            self.meter.clear();
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| "model-surface-chosen".to_owned());
        }
    }

    /// The tab's rows while the surface measure is chosen: the tolerance and
    /// the force-parallel switch. A change measures the picks again.
    pub(super) fn surface_controls(&mut self, ui: &mut Ui) {
        let s = &mut self.measure.surfaces;
        let (was, forced) = (s.parallel_degrees, s.force);
        ui.horizontal_wrapped(|ui| {
            ui.label(t::SURFACE_PARALLEL);
            let drag = ui
                .add(
                    egui::DragValue::new(&mut s.parallel_degrees)
                        .speed(0.05)
                        .range(0.0..=90.0)
                        .max_decimals(2)
                        .suffix("\u{b0}"),
                )
                .on_hover_text(t::SURFACE_PARALLEL_TIP);
            crate::diag::ui_rect_visible(REGION_PARALLEL, drag.rect, ui.clip_rect());
        });
        let force = ui
            .checkbox(&mut s.force, t::SURFACE_FORCE)
            .on_hover_text(t::SURFACE_FORCE_TIP);
        crate::diag::ui_rect_visible(REGION_FORCE, force.rect, ui.clip_rect());
        if s.parallel_degrees != was || s.force != forced {
            let (parallel, force) = (s.parallel_degrees, s.force);
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| {
                format!(
                    "model-surface-tolerance parallel={parallel:.6} force={}",
                    u8::from(force)
                )
            });
            s.measure(&self.model.drawn.meshes);
        }
    }

    /// The whole of the measurement held, in the Measure tab.
    pub(super) fn surface_details(&self, ui: &mut Ui) {
        let s = &self.measure.surfaces;
        let Some(Ok(m)) = &s.answer else {
            return;
        };
        let unit = units_word(self.model.drawn.units);
        let [head, relation, fit] = lines(m, s.parallel_degrees, unit);
        ui.label(RichText::new(head).size(VALUE_SIZE));
        ui.small(relation);
        ui.small(fit);
        for (which, p) in [(t::SURFACE_FIRST, &m.a), (t::SURFACE_SECOND, &m.b)] {
            ui.small(t::surface_detail(
                which,
                p.triangles,
                p.area,
                unit,
                p.flatness_degrees,
            ));
        }
        ui.small(t::surface_sides(m.normals_degrees));
        ui.small(t::surface_points(m.pick_distance, unit));
        let [x, y, z] = m.delta;
        ui.small(t::along_axes(x, y, z));
        if unit.is_empty() {
            ui.small(t::NO_UNIT);
        }
    }

    /// The measurement under the picture, and forgetting it: the same rows
    /// whatever it holds, so the picture keeps its size from one pick to the
    /// next.
    pub(super) fn surface_readout(&mut self, ui: &mut Ui) {
        let s = &self.measure.surfaces;
        let unit = units_word(self.model.drawn.units);
        let blank = || BLANK.to_owned();
        let [head, relation, fit] = match &s.answer {
            None => [t::surface_next(s.picks.len()).to_owned(), blank(), blank()],
            Some(Err(why)) => [t::surface_refused(why), blank(), blank()],
            Some(Ok(m)) => lines(m, s.parallel_degrees, unit),
        };
        let fit = if self.measure.said == Some(Said::Missed) {
            t::MISSED.to_owned()
        } else {
            fit
        };
        ui.add(egui::Label::new(RichText::new(head).size(VALUE_SIZE)).truncate());
        row(ui, &relation);
        row(ui, &fit);
        let again = ui
            .add_enabled(s.holds(), egui::Button::new(t::START_AGAIN))
            .on_hover_text(t::START_AGAIN_TOOLTIP)
            .on_disabled_hover_text(t::surface_next(0));
        crate::diag::ui_rect_visible(REGION_START_AGAIN, again.rect, ui.clip_rect());
        if again.clicked() {
            self.measure.clear();
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| "model-surface-cleared".to_owned());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_measure_starts_at_the_engines_tolerance_and_holds_nothing() {
        let s = Surfaces::default();
        assert_eq!(s.tolerance().parallel_degrees, DEFAULT_PARALLEL_DEGREES);
        assert!(!s.tolerance().force_parallel);
        assert!(!s.holds());
        assert!(s.leader().is_none());
    }
}
