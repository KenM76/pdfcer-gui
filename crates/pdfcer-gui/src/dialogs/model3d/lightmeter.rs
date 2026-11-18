//! The Measure tab's light meter: a tap on a surface asks the engine's
//! `LightMeter` how much light arrives there, at the point or as the mean over
//! a circle round it, under the lights the picture is drawn with and with the
//! parts hidden that the picture hides.
//!
//! A reading is worked out off the frame loop from a copy of the painted
//! model. It is said in the tab and under the picture, with the engine's
//! reasons when it is not in lux and its notes on what it assumed. Nothing is
//! written to the file.

use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError, channel};

use egui::{Pos2, Rect, RichText, Ui};
use pdfcer_3d::RenderOptions as ModelRenderOptions;
use pdfcer_3d::vec3::{add, cross, normalize, scale};
use pdfcer_3d::{
    AssembledModel, Camera, GlowLight, Hit, Illuminance, IlluminanceError, IlluminanceQuery,
    IlluminanceUnit, LightMeter, MeterSettings, PreparedModel,
};

use super::ModelView;
use super::measure::{LIGHT, Said, aspect_of, units_word, whole};
use super::measure_tab::REGION_START_AGAIN;
use crate::text::model3dmeasure as t;

/// The meter's button in the kind row.
pub const REGION_LIGHT: &str = "model3d.measure.light"; // ui-text-exempt: trace region name, never displayed
/// *At the point*.
pub const REGION_POINT: &str = "model3d.light.point"; // ui-text-exempt: trace region name, never displayed
/// *Average over a circle*.
pub const REGION_CIRCLE: &str = "model3d.light.circle"; // ui-text-exempt: trace region name, never displayed
/// The circle's radius.
pub const REGION_RADIUS: &str = "model3d.light.radius"; // ui-text-exempt: trace region name, never displayed
/// *Count glowing parts*.
pub const REGION_GLOW: &str = "model3d.light.glow"; // ui-text-exempt: trace region name, never displayed

/// Samples a circle is averaged over, tried in turn until one is within the
/// engine's cap on a reading's work.
const SAMPLES: [usize; 3] = [256, 64, 16];
/// The lights listed under a reading, strongest first.
const SHARES: usize = 8;
/// A circle's first radius, as a part of the model's reach.
const FIRST_RADIUS: f64 = 0.05;
/// The headline's text size, in points.
const VALUE_SIZE: f32 = 16.0;
/// A row that says nothing and keeps its height.
const BLANK: &str = " "; // ui-text-exempt: an empty row, no words
/// The corners a circle is drawn with.
const RING: u32 = 48;

/// One reading and what the engine said about it.
#[derive(Debug, Clone)]
pub(super) struct Reading {
    pub point: [f64; 3],
    /// The side measured.
    pub normal: [f64; 3],
    /// The circle's radius in model units; 0 for the point alone.
    pub radius: f64,
    pub light: Illuminance,
    /// Why it is not in lux; empty when it is.
    pub reasons: Vec<String>,
    pub notes: Vec<String>,
}

impl Reading {
    /// The circle averaged over, as points round it; empty for a point.
    pub(super) fn ring(&self) -> Vec<[f64; 3]> {
        let Some(n) = normalize(self.normal).filter(|_| self.radius > 0.0) else {
            return Vec::new();
        };
        let axis = if n[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let Some(u) = normalize(cross(n, axis)) else {
            return Vec::new();
        };
        let v = cross(n, u);
        (0..=RING)
            .map(|i| {
                let turn = std::f64::consts::TAU * f64::from(i) / f64::from(RING);
                let (sin, cos) = turn.sin_cos();
                let out = add(scale(u, self.radius * cos), scale(v, self.radius * sin));
                add(self.point, out)
            })
            .collect()
    }
}

/// A reading, or the engine's reason there is none.
type Answer = Result<Reading, String>;

/// The meter's state.
#[derive(Default)]
pub(super) struct Meter {
    /// The circle's radius in model units; 0 reads the point alone.
    pub radius: f64,
    /// Whether glowing parts light what they face.
    pub glow: GlowLight,
    pub reading: Option<Answer>,
    pending: Option<Receiver<Answer>>,
    /// The last pick, read again when the glow switch changes.
    last: Option<(Rect, Pos2, Hit)>,
}

impl Meter {
    /// Forget the reading and any in flight.
    pub(super) fn clear(&mut self) {
        self.reading = None;
        self.pending = None;
        self.last = None;
    }

    /// The reading to draw on the picture.
    pub(super) fn drawn(&self) -> Option<&Reading> {
        self.reading.as_ref().and_then(|r| r.as_ref().ok())
    }

    /// Whether there is anything *Start again* would forget.
    pub(super) fn holds(&self) -> bool {
        self.reading.is_some() || self.pending.is_some()
    }
}

/// Everything one reading is worked out from.
struct Job {
    model: AssembledModel,
    prepared: Arc<PreparedModel>,
    camera: Camera,
    options: ModelRenderOptions,
    query: IlluminanceQuery,
    glow: GlowLight,
}

/// Work out `job`'s reading: a circle with as many samples as the engine's
/// cap allows.
fn read(job: &Job) -> Answer {
    let mut settings = MeterSettings::default();
    settings.glow = job.glow;
    let meter = LightMeter::new(
        &job.model,
        &job.prepared,
        &job.camera,
        &job.options,
        &settings,
    )
    .map_err(|e| e.to_string())?;
    let tries = if job.query.radius > 0.0 {
        &SAMPLES[..]
    } else {
        &SAMPLES[..1]
    };
    let mut refused = String::new();
    for &samples in tries {
        match meter.read(&job.query.over_disc(job.query.radius, samples)) {
            Ok(light) => {
                return Ok(Reading {
                    point: job.query.point,
                    normal: job.query.normal,
                    radius: job.query.radius,
                    light,
                    reasons: meter.reasons().iter().map(ToString::to_string).collect(),
                    notes: meter.notes().iter().map(ToString::to_string).collect(),
                });
            }
            Err(e @ IlluminanceError::TooMuchWork { .. }) => refused = e.to_string(),
            Err(e) => return Err(e.to_string()),
        }
    }
    Err(refused)
}

fn trace(answer: &Answer) {
    crate::diag::trace(|| match answer {
        Ok(r) => {
            let l = &r.light;
            let lit = l.lights.iter().filter(|s| s.direct > 0.0).count();
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-light-reading unit={} total={:.6} direct={:.6} emitted={:.6} bounced={:.6} \
                 ambient={:.6} radius={:.6} on={} off={} lights={} lit={lit} reasons={} notes={}",
                if l.unit == IlluminanceUnit::Lux {
                    "lux"
                } else {
                    "relative"
                },
                l.total,
                l.direct,
                l.emitted,
                l.bounced,
                l.ambient,
                r.radius,
                l.on_surface,
                l.off_edge,
                l.lights.len(),
                r.reasons.len(),
                r.notes.len(),
            )
        }
        // ui-text-exempt: diagnostic trace, never displayed
        Err(why) => format!("model-light-refused why={why:?}"),
    });
}

impl ModelView {
    /// Ask for the light at `hit`, picked at `at` on the picture `rect`.
    pub(super) fn meter_ask(&mut self, rect: Rect, at: Pos2, hit: &Hit) {
        let Some(ray) = self.ray_at(rect, at) else {
            return;
        };
        let Ok(camera) = self.view_camera(aspect_of(rect)) else {
            return;
        };
        let [width, height] = whole(rect);
        let options = ModelRenderOptions {
            width,
            height,
            background: self.shot.screen,
            threads: 0,
            ..self.look().options()
        };
        let radius = self.meter.radius.max(0.0);
        let query = IlluminanceQuery::from_pick(hit, &ray).over_disc(radius, SAMPLES[0]);
        let (model, prepared) = self.meter_inputs();
        crate::diag::trace(|| {
            let ([x, y, z], [i, j, k]) = (query.point, query.normal);
            let mesh = hit.mesh;
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-light-asked mesh={mesh} radius={radius:.6} at={x:.4},{y:.4},{z:.4} n={i:.3},{j:.3},{k:.3}"
            )
        });
        let glow = self.meter.glow;
        let job = Job {
            model,
            prepared,
            camera,
            options,
            query,
            glow,
        };
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let _ = tx.send(read(&job));
        });
        self.meter.reading = None;
        self.meter.pending = Some(rx);
        self.meter.last = Some((rect, at, *hit));
    }

    /// Take a finished reading; while one is in flight, look again shortly.
    pub(super) fn meter_poll(&mut self, ctx: &egui::Context) {
        let Some(rx) = &self.meter.pending else {
            return;
        };
        match rx.try_recv() {
            Ok(answer) => {
                trace(&answer);
                self.meter.reading = Some(answer);
                self.meter.pending = None;
            }
            Err(TryRecvError::Empty) => {
                ctx.request_repaint_after(std::time::Duration::from_millis(50));
            }
            Err(TryRecvError::Disconnected) => self.meter.pending = None,
        }
    }

    /// The meter's button, after the kinds.
    pub(super) fn meter_button(&mut self, ui: &mut Ui) {
        let chosen = self.measure.light();
        let button = ui
            .add(egui::Button::new(t::LIGHT).selected(chosen))
            .on_hover_text(t::LIGHT_TIP);
        crate::diag::ui_rect_visible(REGION_LIGHT, button.rect, ui.clip_rect());
        if button.clicked() && !chosen {
            self.measure.kind = LIGHT;
            self.measure.clear();
            self.meter.clear();
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| "model-light-chosen".to_owned());
        }
    }

    /// The tab's rows while the meter is chosen: the point or a circle.
    pub(super) fn meter_controls(&mut self, ui: &mut Ui) {
        let circle = self.meter.radius > 0.0;
        ui.horizontal_wrapped(|ui| {
            let point = ui.radio(!circle, t::LIGHT_POINT);
            crate::diag::ui_rect_visible(REGION_POINT, point.rect, ui.clip_rect());
            if point.clicked() && circle {
                self.meter_radius(0.0);
            }
            let round = ui
                .radio(circle, t::LIGHT_CIRCLE)
                .on_hover_text(t::LIGHT_CIRCLE_TIP);
            crate::diag::ui_rect_visible(REGION_CIRCLE, round.rect, ui.clip_rect());
            if round.clicked() && !circle {
                self.meter_radius(self.reach * FIRST_RADIUS);
            }
        });
        if self.lights.has_glowing_surfaces() {
            self.meter_glow_switch(ui);
        }
        if circle {
            ui.horizontal_wrapped(|ui| {
                ui.label(t::LIGHT_RADIUS);
                let mut radius = self.meter.radius;
                let drag = ui.add(
                    egui::DragValue::new(&mut radius)
                        .speed(self.reach * 0.002)
                        .range(self.reach * 1e-4..=self.reach)
                        .max_decimals(4),
                );
                crate::diag::ui_rect_visible(REGION_RADIUS, drag.rect, ui.clip_rect());
                ui.label(units_word(self.model.drawn.units));
                if radius != self.meter.radius {
                    self.meter_radius(radius);
                }
            });
        }
    }

    /// *Count glowing parts*, drawn only for a model that has some. A change
    /// reads the last pick again.
    fn meter_glow_switch(&mut self, ui: &mut Ui) {
        let mut counted = self.meter.glow == GlowLight::Counted;
        let switch = ui
            .checkbox(&mut counted, t::LIGHT_GLOW)
            .on_hover_text(t::LIGHT_GLOW_TIP);
        crate::diag::ui_rect_visible(REGION_GLOW, switch.rect, ui.clip_rect());
        if !switch.changed() {
            return;
        }
        self.meter.glow = if counted {
            GlowLight::Counted
        } else {
            GlowLight::LeftOut
        };
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| format!("model-light-glow counted={counted}"));
        if let Some((rect, at, hit)) = self.meter.last {
            self.meter_ask(rect, at, &hit);
        }
    }

    fn meter_radius(&mut self, radius: f64) {
        self.meter.radius = radius;
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| format!("model-light-radius radius={radius:.6}"));
    }

    /// The reading under the picture, and forgetting it: the same rows
    /// whatever it holds, so the picture keeps its size from one pick to the
    /// next and a second click on a spot lands on the same point. What it is
    /// made of is on the amount's hover and in the Measure tab.
    pub(super) fn meter_readout(&mut self, ui: &mut Ui) {
        let blank = || BLANK.to_owned();
        let (head, parts, circle) = match &self.meter.reading {
            None if self.meter.pending.is_some() => (t::LIGHT_WORKING.to_owned(), blank(), blank()),
            None => (t::LIGHT_PICK.to_owned(), blank(), blank()),
            Some(Err(why)) => (t::light_refused(why), blank(), blank()),
            Some(Ok(reading)) => self.reading_lines(reading),
        };
        let amount = ui.label(RichText::new(head).size(VALUE_SIZE));
        if let Some(reading) = self.meter.drawn() {
            amount.on_hover_ui(|ui| self.reading_details(ui, reading));
        }
        ui.small(parts);
        if self.measure.said == Some(Said::Missed) {
            ui.small(t::MISSED);
        } else {
            ui.small(circle);
        }
        let again = ui
            .add_enabled(self.meter.holds(), egui::Button::new(t::START_AGAIN))
            .on_hover_text(t::START_AGAIN_TOOLTIP)
            .on_disabled_hover_text(t::LIGHT_PICK);
        crate::diag::ui_rect_visible(REGION_START_AGAIN, again.rect, ui.clip_rect());
        if again.clicked() {
            self.meter.clear();
            self.measure.clear();
        }
    }

    /// The whole of the reading held, in the Measure tab.
    pub(super) fn meter_details(&self, ui: &mut Ui) {
        let Some(reading) = self.meter.drawn() else {
            return;
        };
        let (head, parts, circle) = self.reading_lines(reading);
        ui.label(RichText::new(head).size(VALUE_SIZE));
        ui.small(parts);
        if reading.radius > 0.0 {
            ui.small(circle);
        }
        self.reading_details(ui, reading);
    }

    /// A reading's amount, what it is made of, and its circle (a space for
    /// a point).
    fn reading_lines(&self, reading: &Reading) -> (String, String, String) {
        let light = &reading.light;
        let lux = light.unit == IlluminanceUnit::Lux;
        let amount = |v: f64| amount(v, lux);
        let glow = (self.meter.glow == GlowLight::Counted && self.lights.has_glowing_surfaces())
            .then(|| amount(light.emitted));
        let parts = t::light_parts(
            &amount(light.direct),
            glow.as_deref(),
            &amount(light.bounced),
            &amount(light.ambient),
        );
        let circle = if reading.radius > 0.0 {
            t::light_circle(
                reading.radius,
                units_word(self.model.drawn.units),
                light.on_surface,
                light.off_edge,
            )
        } else {
            BLANK.to_owned()
        };
        (t::light_total(light.total, lux), parts, circle)
    }

    /// Each light's part of a reading, why it is not in lux, and what the
    /// engine assumed.
    fn reading_details(&self, ui: &mut Ui, reading: &Reading) {
        let light = &reading.light;
        let lux = light.unit == IlluminanceUnit::Lux;
        let mut shares: Vec<_> = light.lights.iter().filter(|s| s.direct > 0.0).collect();
        shares.sort_by(|a, b| b.direct.total_cmp(&a.direct));
        if !shares.is_empty() {
            ui.small(t::LIGHT_SHARES);
            for share in shares.iter().take(SHARES) {
                ui.small(t::light_share(&share.name, &amount(share.direct, lux)));
            }
            if shares.len() > SHARES {
                ui.small(t::light_more(shares.len() - SHARES));
            }
        }
        if !reading.reasons.is_empty() {
            ui.small(t::LIGHT_RELATIVE_WHY);
            for reason in &reading.reasons {
                ui.small(reason);
            }
        }
        for note in &reading.notes {
            ui.small(note);
        }
    }
}

/// One of a reading's numbers as said; a minus zero is said as zero.
fn amount(v: f64, lux: bool) -> String {
    let v = v + 0.0;
    if lux {
        t::light_number(v)
    } else {
        format!("{v:.3}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_meter_with_nothing_read_holds_nothing() {
        let mut meter = Meter::default();
        assert!(!meter.holds());
        meter.reading = Some(Err("off the surface".to_owned()));
        assert!(meter.holds());
        assert!(meter.drawn().is_none());
        meter.clear();
        assert!(!meter.holds());
    }
}
