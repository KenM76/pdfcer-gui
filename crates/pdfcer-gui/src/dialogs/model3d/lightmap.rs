//! *Colour by light level*: every surface drawn in the colour of the light
//! arriving on it, stepped at the recommended lighting levels.
//!
//! The figures are the engine's (`LightMeter::map`), measured as the light
//! meter reads a point: the same lights, options and glow switch. The side
//! measured is the side the camera sees (`MapSide::FacingCamera`), so a
//! moved camera is measured again once it has stood still for [`SETTLE`];
//! until then the last map is drawn from the new view, and until the first
//! arrives the model is drawn as usual. A map is drawn unlit on the
//! processor (`render_light_map`), for the live picture and for every
//! picture saved from it. The legend goes under the picture, never on it.
//!
//! In lux the bands are `LuxBands::canada_ohs_steps`, whose source is
//! cited in the legend. A relative map is drawn in the same colours over
//! its own range, and the legend says its figures are not lux and why.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::time::{Duration, Instant};

use egui::Ui;
use pdfcer_3d::RenderOptions as ModelRenderOptions;
use pdfcer_3d::{
    AssembledModel, BandBlend, CANADA_OHS, CANADA_OHS_SOURCE, Camera, GlowLight, IlluminanceError,
    IlluminanceMap, IlluminanceUnit, Image, LevelLimit, LightMeter, LuxBand, LuxBands, MapSide,
    MeterSettings, PreparedModel, RenderError, render_light_map,
};

use super::ModelView;
use crate::text::model3dmap as t;

/// The switch.
pub const REGION_MAP: &str = "model3d.light_map"; // ui-text-exempt: trace region name, never displayed
/// The blend switch.
pub const REGION_MAP_BLEND: &str = "model3d.light_map.blend"; // ui-text-exempt: trace region name, never displayed

/// How long the camera stands still before a moved view is measured again.
const SETTLE: Duration = Duration::from_millis(600);
/// How often the progress is redrawn while a map is measured.
const PROGRESS_EVERY: Duration = Duration::from_millis(250);
/// The longest edge measured, as a share of the model's diagonal: the
/// engine's own default density.
const PER_DIAGONAL: f64 = 64.0;
/// How many times a coarser density is tried when the engine refuses one
/// as too many points or too much work.
const COARSER_TRIES: u32 = 4;

/// What one map is measured for.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Key {
    /// [`super::parts::Visibility::version`].
    parts: u32,
    /// The look's options with the picture's size zeroed.
    options: ModelRenderOptions,
    glow: GlowLight,
    camera: Camera,
}

impl Key {
    /// The same map but for the camera.
    fn same_but_camera(&self, other: &Key) -> bool {
        Key {
            camera: other.camera,
            ..*self
        } == *other
    }
}

/// A map being measured; dropping it stops the work.
struct Pending {
    key: Key,
    rx: Receiver<Result<IlluminanceMap, String>>,
    done: Arc<AtomicUsize>,
    total: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
    started: Instant,
}

impl Drop for Pending {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

/// The switch, its blend, the map drawn and the one being measured.
#[derive(Default)]
pub(super) struct LightMap {
    pub on: bool,
    /// Shade between levels instead of stepping.
    pub blend: bool,
    ready: Option<(Key, Arc<IlluminanceMap>)>,
    failed: Option<(Key, String)>,
    pending: Option<Pending>,
    /// The camera last seen and since when.
    still: Option<(Camera, Instant)>,
    /// Counts the maps that arrived, so a new one draws a new picture.
    version: u32,
}

impl LightMap {
    /// Forget every map: the model was replaced.
    pub(super) fn forget(&mut self) {
        *self = Self {
            on: self.on,
            blend: self.blend,
            ..Self::default()
        };
    }

    /// The map and its bands, when a map is drawn.
    pub(super) fn drawn(&self) -> Option<(Arc<IlluminanceMap>, LuxBands)> {
        if !self.on {
            return None;
        }
        let (_, map) = self.ready.as_ref()?;
        Some((Arc::clone(map), bands(map, self.blend)))
    }

    /// What the picture is drawn for: 0 for the model as usual, else
    /// changes with each map and the blend.
    pub(super) fn stamp(&self) -> u32 {
        if self.on && self.ready.is_some() {
            self.version.wrapping_mul(2).wrapping_add(1) + u32::from(self.blend) * 2
        } else {
            0
        }
    }

    /// Take a finished map.
    fn poll(&mut self) {
        let Some(p) = &self.pending else {
            return;
        };
        let arrived = match p.rx.try_recv() {
            Ok(arrived) => arrived,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => Err(String::new()),
        };
        let (key, ms) = (p.key, p.started.elapsed().as_secs_f64() * 1000.0);
        self.pending = None;
        trace_arrived(&arrived, ms);
        match arrived {
            Ok(map) => {
                self.ready = Some((key, Arc::new(map)));
                self.failed = None;
                self.version = self.version.wrapping_add(1);
            }
            Err(why) => self.failed = Some((key, why)),
        }
    }

    /// Whether `key` needs measuring now: nothing is drawn or measured
    /// for it, and either nothing else is either, the change is more than
    /// the camera, or the camera has stood still for [`SETTLE`].
    fn wants(&mut self, key: &Key, now: Instant) -> bool {
        let pending = self.pending.as_ref().map(|p| p.key);
        let ready = self.ready.as_ref().map(|(k, _)| *k);
        let failed = self.failed.as_ref().map(|(k, _)| *k);
        if [pending, ready, failed].contains(&Some(*key)) {
            self.still = None;
            return false;
        }
        let Some(held) = pending.or(ready).or(failed) else {
            return true;
        };
        if !held.same_but_camera(key) {
            return true;
        }
        match self.still {
            Some((camera, since)) if camera == key.camera => now.duration_since(since) >= SETTLE,
            _ => {
                self.still = Some((key.camera, now));
                false
            }
        }
    }
}

/// The bands `map` is drawn in: the recommended levels in lux, the
/// same colours over the map's own range when relative (`LuxBands::for_map`).
fn bands(map: &IlluminanceMap, blend: bool) -> LuxBands {
    LuxBands::for_map(
        map,
        if blend {
            BandBlend::Graded
        } else {
            BandBlend::Stepped
        },
    )
}

/// Everything one map is measured from.
struct Job {
    model: AssembledModel,
    prepared: Arc<PreparedModel>,
    options: ModelRenderOptions,
    camera: Camera,
    glow: GlowLight,
    spacing: f64,
}

/// Measure `job`, coarser each time the engine refuses the density.
fn measure(
    job: &Job,
    done: &AtomicUsize,
    total: &AtomicUsize,
    stop: &AtomicBool,
) -> Result<IlluminanceMap, String> {
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
    let mut spacing = job.spacing;
    let mut refused = String::new();
    for _ in 0..COARSER_TRIES {
        let mut progress = |p: pdfcer_3d::MapProgress| {
            done.store(p.done, Ordering::Relaxed);
            total.store(p.total, Ordering::Relaxed);
            !stop.load(Ordering::Relaxed)
        };
        match meter.map(Some(spacing), MapSide::FacingCamera, &mut progress) {
            Ok(map) => return Ok(map),
            Err(
                e
                @ (IlluminanceError::TooManySamples { .. } | IlluminanceError::TooMuchWork { .. }),
            ) => {
                refused = e.to_string();
                spacing *= 2.0;
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    Err(refused)
}

fn trace_arrived(arrived: &Result<IlluminanceMap, String>, ms: f64) {
    crate::diag::trace(|| match arrived {
        Ok(map) => {
            let (lo, hi) = map.range().unwrap_or((0.0, 0.0));
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-light-map-ready unit={} lo={lo:.3} hi={hi:.3} triangles={} reasons={} notes={} ms={ms:.0}",
                if map.unit == IlluminanceUnit::Lux {
                    "lux"
                } else {
                    "relative"
                },
                map.triangles().count(),
                map.reasons.len(),
                map.notes.len(),
            )
        }
        // ui-text-exempt: diagnostic trace, never displayed
        Err(why) => format!("model-light-map-failed why={why:?}"),
    });
}

impl ModelView {
    /// Each frame: take a finished map and start one the view needs.
    /// `aspect` is the picture's, for the camera measured from.
    pub(super) fn light_map_frame(&mut self, ctx: &egui::Context, aspect: f64) {
        if !self.light_map.on {
            self.light_map.pending = None;
            self.light_map.still = None;
            return;
        }
        self.light_map.poll();
        let Ok(camera) = self.view_camera(aspect) else {
            return;
        };
        let key = Key {
            parts: self.parts.version,
            options: ModelRenderOptions {
                width: 0,
                height: 0,
                ..self.look().options()
            },
            glow: self.meter.glow,
            camera,
        };
        if self.light_map.wants(&key, Instant::now()) {
            self.light_map_start(key);
        } else if self.light_map.still.is_some() {
            ctx.request_repaint_after(SETTLE);
        }
        if self.light_map.pending.is_some() {
            ctx.request_repaint_after(PROGRESS_EVERY);
        }
    }

    fn light_map_start(&mut self, key: Key) {
        let (model, prepared) = self.meter_inputs();
        let job = Job {
            model,
            prepared,
            options: key.options,
            camera: key.camera,
            glow: key.glow,
            spacing: 2.0 * self.bounds.radius() / PER_DIAGONAL,
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-light-map-asked spacing={:.6} blend={}",
                job.spacing,
                u8::from(self.light_map.blend)
            )
        });
        let done = Arc::new(AtomicUsize::new(0));
        let total = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let (tx, rx) = channel();
        let (d, n, s) = (Arc::clone(&done), Arc::clone(&total), Arc::clone(&stop));
        std::thread::spawn(move || {
            let _ = tx.send(measure(&job, &d, &n, &s));
        });
        self.light_map.still = None;
        self.light_map.pending = Some(Pending {
            key,
            rx,
            done,
            total,
            stop,
            started: Instant::now(),
        });
    }

    /// The map drawn from `camera` at `options`' size and background, when
    /// one is drawn.
    pub(super) fn light_map_image(
        &self,
        camera: &Camera,
        options: &ModelRenderOptions,
    ) -> Option<Result<Image, RenderError>> {
        let (map, bands) = self.light_map.drawn()?;
        Some(render_light_map(&map, &bands, camera, options))
    }

    /// The switch and its blend, on the View tab.
    pub(super) fn light_map_control(&mut self, ui: &mut Ui) {
        let before = (self.light_map.on, self.light_map.blend);
        let switch = ui
            .checkbox(&mut self.light_map.on, t::map_switch())
            .on_hover_text(t::map_switch_tooltip());
        crate::diag::ui_rect_visible(REGION_MAP, switch.rect, ui.clip_rect());
        if self.light_map.on {
            let blend = ui
                .checkbox(&mut self.light_map.blend, t::map_blend())
                .on_hover_text(t::map_blend_tooltip());
            crate::diag::ui_rect_visible(REGION_MAP_BLEND, blend.rect, ui.clip_rect());
        }
        let after = (self.light_map.on, self.light_map.blend);
        if after != before {
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| {
                format!(
                    "model-light-map-switch on={} blend={}",
                    u8::from(after.0),
                    u8::from(after.1)
                )
            });
        }
    }

    /// Under the picture: the progress, why there is no map, or the legend.
    pub(super) fn light_map_legend(&self, ui: &mut Ui) {
        let m = &self.light_map;
        if !m.on {
            return;
        }
        let measuring = m.pending.as_ref().map(|p| {
            t::map_measuring(
                p.done.load(Ordering::Relaxed),
                p.total.load(Ordering::Relaxed),
            )
        });
        let Some((map, bands)) = m.drawn() else {
            if let Some(line) = measuring {
                ui.small(line);
            } else if let Some((_, why)) = &m.failed {
                ui.small(t::map_failed(why));
            }
            return;
        };
        let lux = map.unit == IlluminanceUnit::Lux;
        ui.horizontal_wrapped(|ui| {
            ui.small(if lux {
                t::map_legend_lux()
            } else {
                t::map_legend_relative()
            });
            for band in bands.bands() {
                swatch(ui, band, lux);
            }
            if let Some(line) = measuring {
                ui.weak(line);
            }
        });
        if let Some((lo, hi)) = map.range() {
            ui.small(t::map_range(lo, hi, lux));
        }
        let join = |parts: Vec<String>| parts.join(crate::text::model3dlight::NOTES_SEPARATOR);
        if !map.reasons.is_empty() {
            ui.small(t::map_not_lux(&join(
                map.reasons.iter().map(ToString::to_string).collect(),
            )));
        }
        if !map.notes.is_empty() {
            ui.small(t::map_notes(&join(
                map.notes.iter().map(ToString::to_string).collect(),
            )));
        }
        if lux {
            let s = CANADA_OHS_SOURCE;
            ui.small(t::map_source(s.title, s.citation, s.current_to))
                .on_hover_text(t::map_source_tooltip(s.reproduced, s.scope, s.url));
        }
    }
}

/// One band: its colour and lower bound; in lux, the areas recommended at
/// that level on hover.
fn swatch(ui: &mut Ui, band: &LuxBand, lux: bool) {
    let side = ui.text_style_height(&egui::TextStyle::Small);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(side, side), egui::Sense::hover());
    let [r, g, b] = band.colour;
    // NOT A THEME COLOUR: the engine's band colour, the legend for the picture
    ui.painter()
        .rect_filled(rect, 1.0, egui::Color32::from_rgb(r, g, b));
    let label = if lux {
        t::map_band_from(band.from)
    } else {
        format!("{:.3}", band.from) // ui-text-exempt: a bare number
    };
    let response = ui.small(label);
    if lux {
        let areas: Vec<String> = CANADA_OHS
            .iter()
            .filter(|l| (l.lux - band.from).abs() < 0.5)
            .map(|l| t::map_area(l.area, l.limit == LevelLimit::Maximum))
            .collect();
        response.on_hover_text(t::map_band_tooltip(band.from, &areas));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(camera: Camera) -> Key {
        Key {
            parts: 0,
            options: ModelRenderOptions::default(),
            glow: GlowLight::default(),
            camera,
        }
    }

    fn camera(direction: [f64; 3]) -> Camera {
        let bounds = pdfcer_3d::Bounds {
            min: [0.0; 3],
            max: [1.0; 3],
        };
        Camera::fit(&bounds, direction, [0.0, 0.0, 1.0], true, 1.0).unwrap()
    }

    fn cameras() -> (Camera, Camera) {
        (camera([1.0, 0.0, 0.0]), camera([0.0, 1.0, 0.0]))
    }

    #[test]
    fn the_first_map_is_measured_at_once() {
        let mut m = LightMap::default();
        assert!(m.wants(&key(camera([1.0, 0.0, 0.0])), Instant::now()));
    }

    #[test]
    fn a_moved_camera_waits_until_it_stands_still() {
        let (a, b) = cameras();
        let mut m = LightMap {
            failed: Some((key(a), String::new())),
            ..LightMap::default()
        };
        let now = Instant::now();
        assert!(!m.wants(&key(a), now));
        assert!(!m.wants(&key(b), now));
        assert!(!m.wants(&key(b), now + SETTLE / 2));
        assert!(m.wants(&key(b), now + SETTLE));
    }

    #[test]
    fn a_change_beside_the_camera_is_measured_at_once() {
        let mut m = LightMap {
            failed: Some((key(camera([1.0, 0.0, 0.0])), String::new())),
            ..LightMap::default()
        };
        let mut other = key(camera([1.0, 0.0, 0.0]));
        other.parts = 1;
        assert!(m.wants(&other, Instant::now()));
    }

    #[test]
    fn nothing_is_drawn_until_a_map_arrives_and_the_switch_is_on() {
        let m = LightMap {
            on: true,
            ..LightMap::default()
        };
        assert_eq!(m.stamp(), 0);
        assert!(m.drawn().is_none());
    }
}
