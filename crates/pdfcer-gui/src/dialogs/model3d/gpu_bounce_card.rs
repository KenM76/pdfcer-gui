//! The bounce built on the graphics card: the engine's packed scene
//! (`ExposedLighting::gpu_bounce_build`) traced by its `GLSL_ES_300_BOUNCE_*`
//! passes in the paint callback, and the finished atlas read back into the
//! same [`FrameBounce`] the processor's build gives.
//!
//! The worker packs the scene and hands a [`CardJob`] to its [`Slot`]: the
//! frame's bounce, or the light meter's (`super::gpu_meter`); each paint
//! advances one job by row bands within [`BUDGET`], the meter's first, so no single draw comes near
//! the driver's watchdog and the window keeps drawing. The finished bounce
//! goes down the build's own channel, so [`super::gpu_bounce::Bounces`]
//! cannot tell a card build from a processor one. A card that cannot run it
//! hands the build to the processor on a thread, down the same channel. A job
//! whose build was superseded is dropped unfinished.
//!
//! `PDFCER_3D_BOUNCE_CPU=1` keeps every build on the processor, for
//! comparison.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, PoisonError, Weak};
use std::time::{Duration, Instant};

use eframe::egui_glow::glow;
use native_mesh::Value;
use native_mesh::compute::{Bind, Compute, Input};
use pdfcer_3d::{
    FrameBounce, GLSL_ES_300_BOUNCE_BUILD_VERTEX, GLSL_ES_300_BOUNCE_REDUCE_FRAGMENT,
    GLSL_ES_300_BOUNCE_TRACE_FRAGMENT, GpuBounceBuild, RenderError,
};

/// What one build gives, as [`super::gpu_bounce`] receives it.
pub(super) type Built = Result<Option<FrameBounce>, String>;

/// The processor's build of the same bounce, run when the card cannot.
pub(super) type Fallback = Box<dyn FnOnce() -> Result<Option<FrameBounce>, RenderError> + Send>;

/// The card's time per paint: a paint stays well inside a 60 Hz frame.
const BUDGET: Duration = Duration::from_millis(12);
/// The first band's rows; a band then doubles or halves to fit [`BUDGET`].
const FIRST_BAND: u32 = 8;
/// The trace target, then the two atlases a bounce alternates between.
const TRACE: usize = 0;
const ATLAS: [usize; 2] = [1, 2];
/// The trace and reduce programs, in [`Compute::new`]'s order.
const TRACE_PASS: usize = 0;
const REDUCE_PASS: usize = 1;

/// The hand-over between the worker that packs a build and the paint that
/// runs it, and whether the card can.
#[derive(Clone, Default)]
pub(super) struct CardQueue(Arc<Inner>);

/// Which build a job is: each slot holds one, a newer one superseding it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Slot {
    /// The viewer's frame bounce.
    Frame,
    /// The light meter's bounce, which a reading waits on.
    Meter,
}

impl Slot {
    /// The order the paint advances slots in: a reading is a click waited
    /// on, while the frame keeps its old bounce meanwhile.
    const ORDER: [Self; 2] = [Self::Meter, Self::Frame];

    /// The trace event a job in this slot reports under.
    const fn event(self) -> &'static str {
        match self {
            Self::Frame => "model-view-bounce", // ui-text-exempt: a trace event name
            Self::Meter => "model-light-meter-bounce", // ui-text-exempt: a trace event name
        }
    }
}

/// How long a build waits for the first paint to say whether the card can
/// trace it; past this it traces on the processor.
const SUPPORT_WAIT: std::time::Duration = std::time::Duration::from_secs(2);

#[derive(Default)]
struct Inner {
    /// Indexed by `Slot as usize`.
    jobs: Mutex<[Option<CardJob>; 2]>,
    /// 0 not yet known, 1 the card renders into float targets, 2 it cannot.
    support: AtomicU8,
}

impl CardQueue {
    /// A build is queued or part-way through on the card.
    pub(super) fn busy(&self) -> bool {
        self.lock().iter().any(Option::is_some)
    }

    /// Whether a build should be packed for the card. A build started before
    /// the first paint has said waits for it, up to [`SUPPORT_WAIT`]: the
    /// card and the processor trace the same bounce to within rounding, so
    /// a first build on the processor and every later one on the card would
    /// change the picture with nothing changed. Runs on the build thread.
    pub(super) fn wanted(&self) -> bool {
        let started = std::time::Instant::now();
        while self.0.support.load(Ordering::Relaxed) == 0 && started.elapsed() < SUPPORT_WAIT {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        self.0.support.load(Ordering::Relaxed) == 1
            && std::env::var_os("PDFCER_3D_BOUNCE_CPU").is_none_or(|v| v != "1")
    }

    /// Record what the paint found the card can do.
    pub(super) fn set_supported(&self, yes: bool) {
        let _ = self.0.support.compare_exchange(
            0,
            if yes { 1 } else { 2 },
            Ordering::Relaxed,
            Ordering::Relaxed,
        );
    }

    /// Queue `job` in `slot`, dropping any job it supersedes.
    pub(super) fn hand(&self, slot: Slot, mut job: CardJob) {
        job.event = slot.event();
        if let Some(old) = self.lock()[slot as usize].replace(job) {
            old.abandon();
        }
    }

    /// Drop every job, for a viewer closing outside a paint.
    pub(super) fn close(&self) {
        for job in self.lock().iter_mut().filter_map(Option::take) {
            job.abandon();
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, [Option<CardJob>; 2]> {
        self.0.jobs.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// One build on its way through the card.
pub(super) struct CardJob {
    build: GpuBounceBuild,
    tx: Sender<Built>,
    /// The trace event, set by the slot it is handed to.
    event: &'static str,
    fallback: Option<Fallback>,
    /// Alive while the build is still the one wanted.
    alive: Weak<()>,
    compute: Option<Compute>,
    step: Step,
    band: u32,
    bands: u32,
    started: Instant,
}

/// Where a job has got to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    /// Tracing bounce `bounce` from row `row`.
    Trace { bounce: u32, row: u32 },
    /// Reducing bounce `bounce` into its atlas.
    Reduce { bounce: u32 },
}

impl CardJob {
    pub(super) fn new(
        build: GpuBounceBuild,
        tx: Sender<Built>,
        alive: Weak<()>,
        fallback: Fallback,
    ) -> Self {
        Self {
            build,
            tx,
            fallback: Some(fallback),
            event: Slot::Frame.event(),
            alive,
            compute: None,
            step: Step::Trace { bounce: 0, row: 0 },
            band: FIRST_BAND,
            bands: 0,
            started: Instant::now(),
        }
    }

    /// A superseded job: nothing to send, and GL objects only a paint frees.
    fn abandon(mut self) {
        if let Some(c) = self.compute.take() {
            retire(c);
        }
    }

    /// The processor's build of this bounce, on a thread, down the channel.
    fn hand_to_processor(mut self, gl: &glow::Context, why: &str) {
        // ui-text-exempt: diagnostic trace, never displayed
        let event = self.event;
        crate::diag::trace(|| format!("{event} built=processor reason={why:?}"));
        if let Some(c) = self.compute.take() {
            c.destroy(gl);
        }
        let (Some(fallback), tx) = (self.fallback.take(), self.tx) else {
            return;
        };
        std::thread::spawn(move || {
            let _ = tx.send(fallback().map_err(|e| e.to_string()));
        });
    }
}

/// GL objects of jobs dropped outside a paint; the next paint deletes them.
static RETIRED: Mutex<Vec<Compute>> = Mutex::new(Vec::new());

fn retire(c: Compute) {
    RETIRED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(c);
}

/// Advance the queued job by one paint's budget, on the paint's context,
/// after deleting what dropped jobs left.
pub(super) fn advance(gl: &glow::Context, queue: &CardQueue) {
    for c in RETIRED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .drain(..)
    {
        c.destroy(gl);
    }
    let mut jobs = queue.lock();
    let Some(index) = Slot::ORDER
        .into_iter()
        .map(|s| s as usize)
        .find(|&i| jobs[i].is_some())
    else {
        return;
    };
    let slot = &mut jobs[index];
    let Some(job) = slot.as_mut() else { return };
    if job.alive.strong_count() == 0 {
        if let Some(c) = job.compute.take() {
            c.destroy(gl);
        }
        *slot = None;
        return;
    }
    if job.compute.is_none() {
        match start(gl, &job.build) {
            Ok(c) => job.compute = Some(c),
            Err(error) => {
                if let Some(job) = slot.take() {
                    job.hand_to_processor(gl, &error);
                }
                return;
            }
        }
    }
    let paint = Instant::now();
    while paint.elapsed() < BUDGET {
        if !step(gl, job) {
            if let Some(job) = slot.take() {
                finish(gl, job);
            }
            return;
        }
    }
}

/// Upload the build and allocate its targets.
fn start(gl: &glow::Context, b: &GpuBounceBuild) -> Result<Compute, String> {
    fn float(t: &pdfcer_3d::FloatTexture) -> Input<'_> {
        Input::Floats {
            width: t.width,
            data: &t.texels,
        }
    }
    let atlas = [b.bounce.atlas_width, b.bounce.atlas_height];
    Compute::new(
        gl,
        GLSL_ES_300_BOUNCE_BUILD_VERTEX,
        &[
            GLSL_ES_300_BOUNCE_TRACE_FRAGMENT,
            GLSL_ES_300_BOUNCE_REDUCE_FRAGMENT,
        ],
        &[
            float(&b.nodes),
            float(&b.triangles),
            float(&b.uvs),
            float(&b.materials),
            Input::Bytes {
                width: b.pictures.width,
                height: b.pictures.height,
                layers: b.pictures.layers,
                data: &b.pictures.texels,
            },
            float(&b.lamps),
            float(&b.probes),
            float(&b.glow),
        ],
        &[[b.trace_width, b.trace_height], atlas, atlas],
    )
}

/// Draw the job's next band or pass; `false` when every bounce is reduced.
fn step(gl: &glow::Context, job: &mut CardJob) -> bool {
    let b = &job.build;
    let Some(compute) = job.compute.as_mut() else {
        return false;
    };
    match job.step {
        Step::Trace { bounce, row } => {
            let rows = row..(row + job.band).min(b.trace_height);
            let drawn = Instant::now();
            compute.draw(
                gl,
                TRACE_PASS,
                TRACE,
                rows.clone(),
                &trace_samplers(bounce),
                &trace_uniforms(b, bounce),
            );
            compute.finish(gl);
            job.bands += 1;
            job.band = fit(job.band, drawn.elapsed());
            job.step = if rows.end >= b.trace_height {
                Step::Reduce { bounce }
            } else {
                Step::Trace {
                    bounce,
                    row: rows.end,
                }
            };
            true
        }
        Step::Reduce { bounce } => {
            let samplers = [
                ("u_trace", Bind::Target(TRACE)),
                ("u_probes", Bind::Input(6)),
            ];
            compute.draw(
                gl,
                REDUCE_PASS,
                atlas(bounce),
                0..b.bounce.atlas_height,
                &samplers,
                &reduce_uniforms(b),
            );
            compute.finish(gl);
            let next = bounce + 1;
            if next >= b.bounces {
                return false;
            }
            job.step = Step::Trace {
                bounce: next,
                row: 0,
            };
            true
        }
    }
}

/// The next band's rows from how long this one took.
fn fit(band: u32, took: Duration) -> u32 {
    if took < BUDGET / 4 {
        (band * 2).min(1024)
    } else if took > BUDGET / 2 {
        (band / 2).max(1)
    } else {
        band
    }
}

/// The atlas bounce `bounce` reduces into.
const fn atlas(bounce: u32) -> usize {
    ATLAS[(bounce % 2) as usize]
}

fn trace_samplers(bounce: u32) -> [(&'static str, Bind); 9] {
    // The first bounce reads no atlas; its sampler still needs a texture.
    let previous = atlas(bounce + 1);
    [
        ("u_nodes", Bind::Input(0)),
        ("u_tris", Bind::Input(1)),
        ("u_uvs", Bind::Input(2)),
        ("u_materials", Bind::Input(3)),
        ("u_pictures", Bind::Input(4)),
        ("u_lamps", Bind::Input(5)),
        ("u_probes", Bind::Input(6)),
        ("u_glow", Bind::Input(7)),
        ("u_bounce_atlas", Bind::Target(previous)),
    ]
}

fn trace_uniforms(b: &GpuBounceBuild, bounce: u32) -> Vec<(&'static str, Value)> {
    vec![
        ("u_lamp_count", Value::Int(b.lamp_count as i32)),
        ("u_glow_count", Value::Int(b.glow_count as i32)),
        ("u_probe_count", Value::Int(b.probe_count as i32)),
        ("u_far", Value::Float(b.far)),
        ("u_eps", Value::Float(b.eps)),
        ("u_sky", Value::Vec3(b.sky)),
        ("u_rays", Value::Vec3s(b.rays.concat())),
        ("u_first", Value::Int(i32::from(bounce == 0))),
        ("u_bounce_origin", Value::Vec3([0.0; 3])),
        ("u_bounce_spacing", Value::Vec3(b.bounce.spacing)),
        ("u_bounce_dims", Value::IVec3(b.bounce.dims)),
    ]
}

fn reduce_uniforms(b: &GpuBounceBuild) -> Vec<(&'static str, Value)> {
    vec![
        ("u_probe_count", Value::Int(b.probe_count as i32)),
        ("u_rays", Value::Vec3s(b.rays.concat())),
        ("u_texels", Value::Vec3s(b.texels.concat())),
        ("u_far", Value::Float(b.far)),
        ("u_cap", Value::Float(b.cap)),
    ]
}

/// Read the last atlas back, send the bounce and free the GL objects.
fn finish(gl: &glow::Context, mut job: CardJob) {
    let Some(compute) = job.compute.take() else {
        return;
    };
    let last = atlas(job.build.bounces.saturating_sub(1));
    let atlas = compute.read(gl, last);
    compute.destroy(gl);
    let mut bounce = job.build.bounce;
    bounce.atlas = atlas;
    let (ms, bands, event) = (job.started.elapsed().as_millis(), job.bands, job.event);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "{event} built=card ms={ms} bands={bands} irradiance={:.6}",
            irradiance(&bounce)
        )
    });
    let _ = job.tx.send(Ok(Some(bounce)));
}

/// The mean of the atlas's irradiance texels (the left 8 × 8 of each probe's
/// tile), RGB averaged: one figure to compare a card and a processor build by.
pub(super) fn irradiance(b: &FrameBounce) -> f64 {
    let width = b.atlas_width as usize;
    let (mut sum, mut n) = (0.0_f64, 0_u64);
    for (i, texel) in b.atlas.chunks_exact(4).enumerate() {
        if (i % width) % 16 < 8 && texel[3] > 0.0 {
            sum += f64::from(texel[0] + texel[1] + texel[2]) / 3.0;
            n += 1;
        }
    }
    if n == 0 { 0.0 } else { sum / n as f64 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_band_grows_when_fast_and_shrinks_when_slow() {
        assert_eq!(fit(8, Duration::from_millis(1)), 16);
        assert_eq!(fit(8, Duration::from_millis(10)), 4);
        assert_eq!(fit(8, Duration::from_millis(4)), 8);
        assert_eq!(fit(1, Duration::from_millis(40)), 1);
        assert_eq!(fit(1024, Duration::ZERO), 1024);
    }

    #[test]
    fn bounces_alternate_atlases_and_read_the_other() {
        assert_ne!(atlas(0), atlas(1));
        assert_eq!(atlas(0), atlas(2));
        assert_eq!(trace_samplers(1)[8].1, Bind::Target(atlas(0)));
    }
}
