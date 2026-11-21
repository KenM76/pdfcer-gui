//! *Colour by light level* measured on the graphics card. The engine lays
//! the map out and packs it (`LightMeter::gpu_map`); the paint draws
//! `GLSL_ES_300_LIGHT_METER_MAP_FRAGMENT` over it in the map slot of
//! [`super::gpu_bounce_card::CardQueue`], once every bounce job is done, by
//! row bands within the card's budget and by runs of [`RUN`] gather
//! directions whose read-backs add up; the measuring thread then finishes
//! the map (`GpuLightMap::finish`) from the sum.
//!
//! The thread measures on the processor instead when the viewer is not
//! drawing on the card, the card cannot render into float targets, the
//! scene is past the engine's 4,096 texture side, the card has not finished
//! within [`WAIT`], or under `PDFCER_3D_BOUNCE_CPU=1`. A map switched off or
//! superseded drops its job between two bands.
//!
//! Trace: `model-light-map-card built=card ms= bands= runs= points=` when the
//! card finished, `built=processor reason=` when it was passed over.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{RecvTimeoutError, Sender, channel};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use eframe::egui_glow::glow;
use native_mesh::Value;
use native_mesh::compute::{Bind, Compute, Input};
use pdfcer_3d::{
    GLSL_ES_300_BOUNCE_BUILD_VERTEX, GLSL_ES_300_LIGHT_METER_MAP_FRAGMENT, GpuLightMap,
    IlluminanceError, IlluminanceMap, LightMeter, MapSide,
};
use pdfcer_gui_base::cardstatus::{Outcome, Path, note};

use super::gpu_bounce_card::{BOUNCE_CPU, BUDGET, CardQueue, FIRST_BAND, fit, forced_cpu, retire};

/// Gather directions one draw traces per point; a point's figure is the sum
/// of its runs, since the shader weights each run by every direction.
const RUN: u32 = 32;
/// The longest the measuring thread waits on the card.
const WAIT: Duration = Duration::from_secs(60);
/// How often the waiting thread looks whether the map was cancelled.
const POLL: Duration = Duration::from_millis(50);
/// Texels a row of the map's points, as the shader indexes them.
const ROW: usize = 2048;
/// The map's only target.
const TARGET: usize = 0;
/// The atlas input of a map with no bounce: the sampler needs a texture.
const NO_ATLAS: [f32; 4] = [0.0; 4];
const EVENT: &str = "model-light-map-card"; // ui-text-exempt: a trace event name

/// What the paint sends back: the map with its summed read-back, or `None`
/// when the card could not run the pass.
type Read = Option<(GpuLightMap, Vec<f32>)>;

/// Why the card did not give the map.
pub(super) enum NotOnCard {
    /// The engine refused or the map was cancelled; the processor would
    /// refuse the same.
    Engine(IlluminanceError),
    /// The processor is to measure it.
    Processor,
}

/// One map on its way through the card.
pub(super) struct MapJob {
    map: GpuLightMap,
    tx: Sender<Read>,
    /// Alive while the map is still wanted.
    alive: Weak<()>,
    /// Points measured so far, summed over every run, for the progress line.
    done: Arc<AtomicUsize>,
    compute: Option<Compute>,
    runs: u32,
    run: u32,
    row: u32,
    band: u32,
    bands: u32,
    sum: Vec<f32>,
    started: Instant,
}

impl MapJob {
    /// A job superseded outside a paint: its GL objects wait for the next.
    pub(super) fn abandon(mut self) {
        if let Some(c) = self.compute.take() {
            retire(c);
        }
    }

    /// Whether the map is still wanted.
    pub(super) fn wanted(&self) -> bool {
        self.alive.strong_count() > 0
    }
}

/// Measure `meter`'s map at `spacing` on `card`, from the measuring thread,
/// updating `done` and `total` as the card goes.
///
/// # Errors
///
/// [`NotOnCard`]: the engine's refusal, or the processor to measure it.
pub(super) fn measure(
    meter: &LightMeter<'_>,
    card: &CardQueue,
    spacing: f64,
    progress: (&Arc<AtomicUsize>, &AtomicUsize, &AtomicBool),
) -> Result<IlluminanceMap, NotOnCard> {
    let (done, total, stop) = progress;
    if forced_cpu() {
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| format!("{EVENT} built=processor reason=\"forced\""));
        note(Path::Map, Outcome::ForcedOff(BOUNCE_CPU));
        return Err(NotOnCard::Processor);
    }
    if !card.supported() {
        return Err(passed_over(
            crate::text::cardstatus::bounce_unsupported().to_owned(),
        ));
    }
    let Some(map) = meter
        .gpu_map(Some(spacing), MapSide::FacingCamera)
        .map_err(NotOnCard::Engine)?
    else {
        return Err(passed_over(
            crate::text::cardstatus::map_too_large().to_owned(),
        ));
    };
    done.store(0, Ordering::Relaxed);
    total.store(
        map.point_count as usize * runs(&map) as usize,
        Ordering::Relaxed,
    );
    let (tx, rx) = channel();
    let alive = Arc::new(());
    card.hand_map(MapJob {
        runs: runs(&map),
        map,
        tx,
        alive: Arc::downgrade(&alive),
        done: Arc::clone(done),
        compute: None,
        run: 0,
        row: 0,
        band: FIRST_BAND,
        bands: 0,
        sum: Vec::new(),
        started: Instant::now(),
    });
    let asked = Instant::now();
    // Returning drops `alive`, so the next paint drops the job.
    loop {
        match rx.recv_timeout(POLL) {
            Ok(Some((map, read))) => {
                note(Path::Map, Outcome::InUse);
                return map.finish(&read).map_err(NotOnCard::Engine);
            }
            Ok(None) => {
                return Err(passed_over(
                    crate::text::cardstatus::meter_refused().to_owned(),
                ));
            }
            Err(RecvTimeoutError::Disconnected) => {
                return Err(NotOnCard::Engine(IlluminanceError::Cancelled));
            }
            Err(RecvTimeoutError::Timeout) if stop.load(Ordering::Relaxed) => {
                return Err(NotOnCard::Engine(IlluminanceError::Cancelled));
            }
            Err(RecvTimeoutError::Timeout) if asked.elapsed() > WAIT => {
                return Err(passed_over(crate::text::cardstatus::meter_timeout(
                    WAIT.as_secs(),
                )));
            }
            Err(RecvTimeoutError::Timeout) => {}
        }
    }
}

/// Record and trace that the processor measures the map, and why.
pub(super) fn passed_over(why: String) -> NotOnCard {
    // ui-text-exempt: diagnostic trace, never displayed
    crate::diag::trace(|| format!("{EVENT} built=processor reason={why:?}"));
    note(Path::Map, Outcome::Processor(why));
    NotOnCard::Processor
}

/// How many draws of the gather a pass takes: one when nothing is gathered.
fn runs(map: &GpuLightMap) -> u32 {
    if map.bounce.is_none() && map.glow_count == 0 {
        1
    } else {
        map.direction_count.div_ceil(RUN).max(1)
    }
}

/// Advance the map in `slot` by what is left of the paint's budget since
/// `paint`, on the paint's context.
pub(super) fn advance(gl: &glow::Context, slot: &mut Option<MapJob>, paint: Instant) {
    let Some(job) = slot.as_mut() else { return };
    if !job.wanted() {
        if let Some(c) = job.compute.take() {
            c.destroy(gl);
        }
        *slot = None;
        return;
    }
    if job.compute.is_none() {
        match start(gl, &job.map) {
            Ok(c) => job.compute = Some(c),
            Err(_) => {
                if let Some(job) = slot.take() {
                    let _ = job.tx.send(None);
                }
                return;
            }
        }
    }
    while paint.elapsed() < BUDGET {
        if !step(gl, job) {
            if let Some(job) = slot.take() {
                finish(gl, job);
            }
            return;
        }
    }
}

/// Upload the map and allocate its target.
fn start(gl: &glow::Context, m: &GpuLightMap) -> Result<Compute, String> {
    fn float(t: &pdfcer_3d::FloatTexture) -> Input<'_> {
        Input::Floats {
            width: t.width,
            data: &t.texels,
        }
    }
    let atlas = m.bounce.as_ref().map_or(
        Input::Floats {
            width: 1,
            data: &NO_ATLAS,
        },
        |b| Input::Floats {
            width: b.atlas_width,
            data: &b.atlas,
        },
    );
    Compute::new(
        gl,
        GLSL_ES_300_BOUNCE_BUILD_VERTEX,
        &[GLSL_ES_300_LIGHT_METER_MAP_FRAGMENT],
        &[
            float(&m.nodes),
            float(&m.triangles),
            float(&m.uvs),
            float(&m.materials),
            Input::Bytes {
                width: m.pictures.width,
                height: m.pictures.height,
                layers: m.pictures.layers,
                data: &m.pictures.texels,
            },
            float(&m.lamps),
            float(&m.glow),
            float(&m.points),
            float(&m.normals),
            float(&m.directions),
            atlas,
        ],
        &[[m.target_width.max(1), m.target_height.max(1)]],
    )
}

/// Draw the job's next band; `false` once the last run is read back.
fn step(gl: &glow::Context, job: &mut MapJob) -> bool {
    let m = &job.map;
    let Some(compute) = job.compute.as_mut() else {
        return false;
    };
    let height = m.target_height.max(1);
    let rows = job.row..(job.row + job.band).min(height);
    let drawn = Instant::now();
    compute.draw(
        gl,
        0,
        TARGET,
        rows.clone(),
        &SAMPLERS,
        &uniforms(m, job.run, job.runs),
    );
    compute.finish(gl);
    job.bands += 1;
    job.band = fit(job.band, drawn.elapsed());
    job.row = rows.end;
    let measured = (rows.end as usize * ROW).min(m.point_count as usize);
    let before = job.run as usize * m.point_count as usize;
    job.done.store(before + measured, Ordering::Relaxed);
    if job.row < height {
        return true;
    }
    add(&mut job.sum, &compute.read(gl, TARGET));
    job.row = 0;
    job.run += 1;
    job.run < job.runs
}

/// Add one run's read-back into the sum.
fn add(sum: &mut Vec<f32>, read: &[f32]) {
    if sum.is_empty() {
        sum.extend_from_slice(read);
    } else {
        for (s, r) in sum.iter_mut().zip(read) {
            *s += r;
        }
    }
}

/// Send the map and its sum, and free the GL objects.
fn finish(gl: &glow::Context, mut job: MapJob) {
    if let Some(c) = job.compute.take() {
        c.destroy(gl);
    }
    let (ms, bands, runs) = (job.started.elapsed().as_millis(), job.bands, job.runs);
    let points = job.map.point_count;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!("{EVENT} built=card ms={ms} bands={bands} runs={runs} points={points}")
    });
    let _ = job.tx.send(Some((job.map, job.sum)));
}

/// The samplers, on [`start`]'s inputs in order.
const SAMPLERS: [(&str, Bind); 11] = [
    ("u_nodes", Bind::Input(0)),
    ("u_tris", Bind::Input(1)),
    ("u_uvs", Bind::Input(2)),
    ("u_materials", Bind::Input(3)),
    ("u_pictures", Bind::Input(4)),
    ("u_lamps", Bind::Input(5)),
    ("u_glow", Bind::Input(6)),
    ("u_points", Bind::Input(7)),
    ("u_normals", Bind::Input(8)),
    ("u_directions", Bind::Input(9)),
    ("u_bounce_atlas", Bind::Input(10)),
];

/// The first and stop direction of run `run` of `runs`: `(0, 0)`, every
/// direction, for a single run.
fn directions(run: u32, runs: u32) -> (i32, i32) {
    if runs <= 1 {
        (0, 0)
    } else {
        ((run * RUN) as i32, ((run + 1) * RUN) as i32)
    }
}

fn uniforms(m: &GpuLightMap, run: u32, runs: u32) -> Vec<(&'static str, Value)> {
    let (first, stop) = directions(run, runs);
    let int = |v: u32| Value::Int(v as i32);
    let mut u = vec![
        ("u_lamp_count", int(m.lamp_count)),
        ("u_bounce_first", int(m.bounce_first)),
        ("u_bounce_count", int(m.bounce_count)),
        ("u_glow_count", int(m.glow_count)),
        ("u_point_count", int(m.point_count)),
        ("u_direction_count", int(m.direction_count)),
        ("u_direction_first", Value::Int(first)),
        ("u_direction_stop", Value::Int(stop)),
        ("u_far", Value::Float(m.far)),
        ("u_gather_far", Value::Float(m.gather_far)),
        ("u_eps", Value::Float(m.eps)),
        ("u_sky", Value::Vec3(m.sky)),
        ("u_bounce", Value::Int(i32::from(m.bounce.is_some()))),
    ];
    if let Some(b) = &m.bounce {
        u.extend([
            ("u_bounce_origin", Value::Vec3(b.origin)),
            ("u_bounce_spacing", Value::Vec3(b.spacing)),
            ("u_bounce_dims", Value::IVec3(b.dims)),
            ("u_bounce_strength", Value::Float(b.strength)),
        ]);
    }
    u
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_single_run_draws_every_direction_and_runs_tile_them() {
        assert_eq!(directions(0, 1), (0, 0));
        let r = RUN as i32;
        assert_eq!(directions(0, 3), (0, r));
        assert_eq!(directions(2, 3), (2 * r, 3 * r));
    }

    #[test]
    fn runs_add_up_texel_by_texel() {
        let mut sum = Vec::new();
        add(&mut sum, &[1.0, 2.0]);
        add(&mut sum, &[0.5, 0.25]);
        assert_eq!(sum, [1.5, 2.25]);
    }
}
