//! The card's bounce light: the engine's probe volume for the frame
//! (`PreparedModel::frame_bounce`), the light the processor's picture adds
//! when lit surfaces light their neighbours.
//!
//! One build traces up to about 800,000 rays, so it runs on a worker thread
//! and is kept until what it depends on changes: the model and its painted
//! parts, and every render option but the picture's size. The camera is not
//! in the key, since the bounce of lights fixed in the model does not move
//! with it; a result that says it [`FrameBounce::follows_camera`] is not
//! drawn, because the viewer never asks for camera lights to bounce. Until a
//! build arrives the faces are drawn without bounce, and the frame is drawn
//! again when it does. A new white point alone rescales the kept build
//! instead of tracing again.
//!
//! Where the card renders into float targets the worker only packs the
//! scene and the paint traces it (`gpu_bounce_card`); the result arrives on
//! the same channel either way.

use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError, channel};

use native_mesh::Value;
use pdfcer_3d::RenderOptions as ModelRenderOptions;
use pdfcer_3d::{AssembledModel, Camera, Exposure, FrameBounce, LampCut, PreparedModel};

use super::gpu_bounce_card::{CardJob, CardQueue, Slot};
use super::gpu_uniforms::Uniforms;

/// What one build depends on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Key {
    /// The options with the picture's size zeroed.
    pub(super) options: ModelRenderOptions,
    /// [`super::parts::Parts::version`].
    pub(super) parts: u32,
    /// The highlight colour the parts are painted with.
    pub(super) accent: [u8; 4],
    /// Metered, or fixed in lux: the bounce is lit as the faces are.
    pub(super) exposure: Exposure,
}

use super::gpu_bounce_card::Built;

/// The kept build and the one in flight.
#[derive(Default)]
pub(super) struct Bounces {
    /// The last build that arrived, and what it was built for.
    ready: Option<(Key, Option<Arc<FrameBounce>>)>,
    /// The build in flight.
    pending: Option<(Key, Receiver<Built>)>,
    /// A build arrived since the frame was last drawn.
    arrived: bool,
    /// The kept build is another white point's, rescaled.
    rescaled: bool,
    /// The last build to arrive failed, so the kept bounce is none.
    failed: bool,
    /// Held while the build in flight is wanted; a card job drops itself
    /// once it is not.
    alive: Option<Arc<()>>,
    /// Where a build packed for the card is handed to the paint.
    card: CardQueue,
}

impl Bounces {
    /// Forget every build: the model was replaced.
    pub(super) fn forget(&mut self) {
        let card = std::mem::take(&mut self.card);
        card.close();
        *self = Self {
            card,
            ..Self::default()
        };
    }

    /// The queue the paint advances card builds from.
    pub(super) fn card(&self) -> CardQueue {
        self.card.clone()
    }

    /// The card has bounce work for the next paint.
    pub(super) fn card_busy(&self) -> bool {
        self.card.busy()
    }

    /// Drop a card build in flight: the viewer is closing.
    pub(super) fn close(&self) {
        self.card.close();
    }

    /// The bounce to draw for `key`, starting a build when none is kept or
    /// in flight for it. `inputs` supplies the build's model, painted, and
    /// its prepared form, called only when a build starts.
    pub(super) fn for_frame(
        &mut self,
        key: Key,
        camera: Camera,
        inputs: impl FnOnce() -> (AssembledModel, Arc<PreparedModel>),
    ) -> Option<Arc<FrameBounce>> {
        if let Some((built_for, bounce)) = &self.ready
            && *built_for == key
        {
            return bounce.clone();
        }
        let scaled = self.rescale(key);
        if scaled.is_some() && key.options.lamp_cut == LampCut::Off {
            return scaled.flatten();
        }
        if self.pending.as_ref().is_none_or(|(k, _)| *k != key) {
            let (model, prepared) = inputs();
            let (tx, rx) = channel();
            let alive = Arc::new(());
            let job = Spawned {
                key,
                camera,
                card: self.card.clone(),
                alive: Arc::downgrade(&alive),
            };
            std::thread::spawn(move || build(job, model, prepared, tx));
            self.alive = Some(alive);
            self.pending = Some((key, rx));
        }
        scaled.flatten()
    }

    /// The kept build rescaled for `key` when only the white point differs:
    /// a bounce is linear in its lights, so no ray is traced again
    /// (`FrameBounce::scaled`). Exact with the lamp cut off; under a cut a
    /// lamp's reach depends on its drawn brightness, so the rescaled one is
    /// drawn while an exact build runs behind it.
    fn rescale(&mut self, key: Key) -> Option<Option<Arc<FrameBounce>>> {
        let (built_for, bounce) = self.ready.as_ref()?;
        let (Exposure::Fixed { white_lux: was }, Exposure::Fixed { white_lux: now }) =
            (built_for.exposure, key.exposure)
        else {
            return None;
        };
        if (Key {
            exposure: key.exposure,
            ..*built_for
        }) != key
        {
            return None;
        }
        let scaled = bounce.as_ref().map(|b| Arc::new(b.scaled(was / now)));
        self.ready = Some((key, scaled.clone()));
        self.rescaled = true;
        Some(scaled)
    }

    /// Take a finished build; `true` when one arrived since the last call,
    /// so the frame must be drawn again.
    pub(super) fn poll(&mut self) -> bool {
        if let Some((key, rx)) = &self.pending {
            match rx.try_recv() {
                Ok(built) => {
                    self.failed = built.is_err();
                    let bounce = match built {
                        Ok(b) => b.filter(|b| !b.follows_camera).map(Arc::new),
                        Err(error) => {
                            // ui-text-exempt: diagnostic trace, never displayed
                            crate::diag::trace(|| {
                                format!("model-view-bounce failed error={error:?}")
                            });
                            None
                        }
                    };
                    self.ready = Some((*key, bounce));
                    self.rescaled = false;
                    self.pending = None;
                    self.arrived = true;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    // The build thread died: keep its key as built-with-none so
                    // the next frame does not start it again.
                    crate::diag::trace(|| {
                        "model-view-bounce failed error=build-thread-died".into()
                    }); // ui-text-exempt: diagnostic trace, never displayed
                    self.ready = Some((*key, None));
                    self.failed = true;
                    self.pending = None;
                    self.arrived = true;
                }
            }
        }
        std::mem::take(&mut self.arrived)
    }

    /// The kept build is another white point's, rescaled.
    pub(super) fn rescaled(&self) -> bool {
        self.rescaled
    }

    /// The last build failed and none is in flight, so the faces are drawn
    /// without bounce.
    pub(super) fn failed(&self) -> bool {
        self.failed && self.pending.is_none()
    }

    /// A build is in flight and nothing kept stands in for it, so the
    /// faces are drawn without bounce.
    pub(super) fn waiting(&self) -> bool {
        self.pending.as_ref().is_some_and(|(pending, _)| {
            self.ready
                .as_ref()
                .is_none_or(|(ready, _)| ready != pending)
        })
    }
}

/// What a build thread is started with, besides the model.
struct Spawned {
    key: Key,
    camera: Camera,
    card: CardQueue,
    alive: std::sync::Weak<()>,
}

/// The build thread: pack the scene for the card when it can trace it and
/// hand it over, else trace it here.
fn build(
    job: Spawned,
    model: AssembledModel,
    prepared: Arc<PreparedModel>,
    tx: std::sync::mpsc::Sender<Built>,
) {
    let Spawned {
        key,
        camera,
        card,
        alive,
    } = job;
    let (options, exposure) = (key.options, key.exposure);
    let lit = super::truelight::lit(&model, exposure, &options);
    if card.wanted() {
        let packed = match &lit {
            Some(l) => l.drawn.gpu_bounce_build(&prepared, &model, &camera),
            None => prepared.gpu_bounce_build(&model, &camera, &options),
        };
        match packed {
            Ok(Some(packed)) => {
                let fallback =
                    Box::new(move || on_processor(lit, &prepared, &model, &camera, &options));
                card.hand(Slot::Frame, CardJob::new(packed, tx, alive, fallback));
                return;
            }
            Ok(None) => {}
            Err(error) => {
                let _ = tx.send(Err(error.to_string()));
                return;
            }
        }
    }
    let _ =
        tx.send(on_processor(lit, &prepared, &model, &camera, &options).map_err(|e| e.to_string()));
}

/// The bounce traced on the processor, as the card would trace it.
fn on_processor(
    lit: Option<super::truelight::Lit>,
    prepared: &PreparedModel,
    model: &AssembledModel,
    camera: &Camera,
    options: &ModelRenderOptions,
) -> Result<Option<FrameBounce>, pdfcer_3d::RenderError> {
    let started = std::time::Instant::now();
    let built = match lit {
        Some(l) => l.drawn.frame_bounce(prepared, model, camera),
        None => prepared.frame_bounce(model, camera, options),
    }?;
    if let Some(b) = &built {
        let (ms, mean) = (
            started.elapsed().as_millis(),
            super::gpu_bounce_card::irradiance(b),
        );
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| {
            format!("model-view-bounce built=processor ms={ms} irradiance={mean:.6}")
        });
    }
    Ok(built)
}

/// The key for a frame drawn with `options` at `exposure`. The bounce is
/// traced on every core: it runs off the frame loop, and on one thread a
/// large model waits minutes for it.
pub(super) fn key(
    options: &ModelRenderOptions,
    exposure: Exposure,
    parts: u32,
    accent: egui::Color32,
) -> Key {
    Key {
        exposure,
        options: ModelRenderOptions {
            width: 0,
            height: 0,
            threads: 0,
            ..*options
        },
        parts,
        accent: accent.to_array(),
    }
}

/// The faces' bounce uniforms; `u_bounce` false without one.
pub(super) fn uniforms(bounce: Option<&FrameBounce>) -> Uniforms {
    let Some(b) = bounce else {
        return vec![("u_bounce", Value::Int(0))];
    };
    vec![
        ("u_bounce", Value::Int(1)),
        ("u_bounce_origin", Value::Vec3(b.origin)),
        ("u_bounce_spacing", Value::Vec3(b.spacing)),
        ("u_bounce_dims", Value::IVec3(b.dims)),
        ("u_bounce_strength", Value::Float(b.strength)),
    ]
}

/// The trace fields: `none`, `waiting` or the probe count, and whether the
/// kept build was traced for this white point or rescaled from another.
pub(super) fn trace(bounce: Option<&FrameBounce>, bounces: &Bounces) -> String {
    let waiting = bounces.waiting();
    let state = match bounce {
        Some(b) => format!("{}", b.dims.iter().product::<i32>()),
        None if waiting => "waiting".to_owned(),
        None => "none".to_owned(),
    };
    let from = if bounces.rescaled() {
        "rescaled"
    } else {
        "traced"
    }; // ui-text-exempt: diagnostic trace
    format!(" gpu-bounce={state} gpu-bounce-from={from}") // ui-text-exempt: diagnostic trace, never displayed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(exposure: Exposure) -> Key {
        key(
            &ModelRenderOptions::default(),
            exposure,
            0,
            egui::Color32::WHITE, // NOT A THEME COLOUR: test fixture accent, never drawn
        )
    }

    #[test]
    fn the_bounce_is_traced_on_every_core() {
        assert_eq!(at(Exposure::Metered).options.threads, 0);
    }

    fn kept(k: Key) -> Bounces {
        Bounces {
            ready: Some((k, None)),
            ..Bounces::default()
        }
    }

    const AT_500: Exposure = Exposure::Fixed { white_lux: 500.0 };
    const AT_1000: Exposure = Exposure::Fixed { white_lux: 1000.0 };

    #[test]
    fn a_new_white_point_alone_rescales_the_kept_build() {
        let mut b = kept(at(AT_500));
        assert_eq!(b.rescale(at(AT_1000)), Some(None));
        assert!(b.rescaled());
        assert!(b.ready.as_ref().is_some_and(|(k, _)| *k == at(AT_1000)));
        assert!(!b.waiting());
    }

    #[test]
    fn anything_else_changed_is_traced_again() {
        let mut other = at(AT_1000);
        other.parts = 1;
        assert_eq!(kept(at(AT_500)).rescale(other), None);
        assert_eq!(kept(at(Exposure::Metered)).rescale(at(AT_1000)), None);
        assert_eq!(kept(at(AT_500)).rescale(at(Exposure::Metered)), None);
        assert_eq!(Bounces::default().rescale(at(AT_1000)), None);
    }

    #[test]
    fn an_arrived_build_is_traced() {
        let mut b = kept(at(AT_500));
        b.rescale(at(AT_1000));
        let (tx, rx) = channel();
        b.pending = Some((at(AT_1000), rx));
        tx.send(Ok(None)).unwrap();
        assert!(b.poll());
        assert!(!b.rescaled());
    }

    #[test]
    fn a_failed_build_is_reported_until_one_succeeds() {
        let mut b = Bounces::default();
        let (tx, rx) = channel();
        b.pending = Some((at(AT_500), rx));
        tx.send(Err("no".into())).unwrap();
        assert!(b.poll());
        assert!(b.failed());
        let (tx, rx) = channel();
        b.pending = Some((at(AT_1000), rx));
        assert!(!b.failed(), "a build in flight is waiting, not failed");
        tx.send(Ok(None)).unwrap();
        b.poll();
        assert!(!b.failed());
    }

    #[test]
    fn a_dead_build_thread_is_not_started_again() {
        let mut b = Bounces::default();
        let (tx, rx) = channel::<Built>();
        b.pending = Some((at(AT_500), rx));
        drop(tx);
        assert!(b.poll());
        assert!(b.failed());
        let started = std::cell::Cell::new(false);
        let camera = Camera {
            eye: [0.0, 0.0, 10.0],
            target: [0.0; 3],
            up: [0.0, 1.0, 0.0],
            projection: pdfcer_3d::Projection::Perspective { fov_y: 30.0 },
        };
        let _ = b.for_frame(at(AT_500), camera, || {
            started.set(true);
            let model = AssembledModel::default();
            let prepared = Arc::new(PreparedModel::new(&model));
            (model, prepared)
        });
        assert!(!started.get());
    }
}
