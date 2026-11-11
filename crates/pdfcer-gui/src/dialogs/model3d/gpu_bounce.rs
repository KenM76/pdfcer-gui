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
//! again when it does.

use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError, channel};

use native_mesh::Value;
use pdfcer_3d::RenderOptions as ModelRenderOptions;
use pdfcer_3d::{AssembledModel, Camera, FrameBounce, PreparedModel};

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
}

/// A finished build: the bounce, or `None` when the frame has none.
type Built = Result<Option<FrameBounce>, String>;

/// The kept build and the one in flight.
#[derive(Default)]
pub(super) struct Bounces {
    /// The last build that arrived, and what it was built for.
    ready: Option<(Key, Option<Arc<FrameBounce>>)>,
    /// The build in flight.
    pending: Option<(Key, Receiver<Built>)>,
    /// A build arrived since the frame was last drawn.
    arrived: bool,
}

impl Bounces {
    /// Forget every build: the model was replaced.
    pub(super) fn forget(&mut self) {
        *self = Self::default();
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
        if self.pending.as_ref().is_none_or(|(k, _)| *k != key) {
            let (model, prepared) = inputs();
            let (tx, rx) = channel();
            let options = key.options;
            std::thread::spawn(move || {
                let built = prepared
                    .frame_bounce(&model, &camera, &options)
                    .map_err(|e| e.to_string());
                let _ = tx.send(built);
            });
            self.pending = Some((key, rx));
        }
        None
    }

    /// Take a finished build; `true` when one arrived since the last call,
    /// so the frame must be drawn again.
    pub(super) fn poll(&mut self) -> bool {
        if let Some((key, rx)) = &self.pending {
            match rx.try_recv() {
                Ok(built) => {
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
                    self.pending = None;
                    self.arrived = true;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => self.pending = None,
            }
        }
        std::mem::take(&mut self.arrived)
    }

    /// A build is in flight.
    pub(super) fn waiting(&self) -> bool {
        self.pending.is_some()
    }
}

/// The key for a frame drawn with `options`.
pub(super) fn key(options: &ModelRenderOptions, parts: u32, accent: egui::Color32) -> Key {
    Key {
        options: ModelRenderOptions {
            width: 0,
            height: 0,
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

/// The trace field: `none`, `waiting` or the probe count.
pub(super) fn trace(bounce: Option<&FrameBounce>, waiting: bool) -> String {
    let state = match bounce {
        Some(b) => format!("{}", b.dims.iter().product::<i32>()),
        None if waiting => "waiting".to_owned(),
        None => "none".to_owned(),
    };
    format!(" gpu-bounce={state}") // ui-text-exempt: diagnostic trace, never displayed
}
