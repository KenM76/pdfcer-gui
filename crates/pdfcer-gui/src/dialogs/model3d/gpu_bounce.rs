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

use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError, channel};

use native_mesh::Value;
use pdfcer_3d::RenderOptions as ModelRenderOptions;
use pdfcer_3d::{AssembledModel, Camera, Exposure, FrameBounce, LampCut, PreparedModel};

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
    /// The kept build is another white point's, rescaled.
    rescaled: bool,
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
        let scaled = self.rescale(key);
        if scaled.is_some() && key.options.lamp_cut == LampCut::Off {
            return scaled.flatten();
        }
        if self.pending.as_ref().is_none_or(|(k, _)| *k != key) {
            let (model, prepared) = inputs();
            let (tx, rx) = channel();
            let (options, exposure) = (key.options, key.exposure);
            std::thread::spawn(move || {
                let built = match super::truelight::lit(&model, exposure, &options) {
                    Some(l) => l.drawn.frame_bounce(&prepared, &model, &camera),
                    None => prepared.frame_bounce(&model, &camera, &options),
                }
                .map_err(|e| e.to_string());
                let _ = tx.send(built);
            });
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
                Err(TryRecvError::Disconnected) => self.pending = None,
            }
        }
        std::mem::take(&mut self.arrived)
    }

    /// The kept build is another white point's, rescaled.
    pub(super) fn rescaled(&self) -> bool {
        self.rescaled
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
}
