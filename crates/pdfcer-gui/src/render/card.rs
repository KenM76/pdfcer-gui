//! # `render::card` — the canvas's graphics-card drawer
//!
//! Makes the [`pdfcer_gui_base::rendercard::CardDraw`] the canvas's pages are
//! drawn with when Settings ▸ Display ▸ Graphics card is on: the engine's two
//! shaders over `native_mesh::scene::SceneDrawer`, on the context egui paints
//! with. Made once, on the first frame with a context; a shader the driver
//! refuses retires the card for the session with the driver's words.

use std::collections::HashMap;
use std::sync::Arc;

use eframe::glow;
use native_mesh::scene::{SceneDrawer, SceneJob, SceneTexture, Step};
use pdfcer_gui_base::rendercard::{self, CardDraw};

fn texture(scene: &pdfcer_render::GpuScene) -> SceneTexture<'_> {
    let data = &scene.data;
    SceneTexture {
        width: data.width,
        height: data.height,
        layers: data.layers,
        texels: &data.texels,
    }
}

/// The drawer, the context it was made on, and its jobs by number.
struct GlCard {
    gl: Arc<glow::Context>,
    drawer: SceneDrawer,
    jobs: HashMap<u64, SceneJob>,
    next: u64,
}

impl CardDraw for GlCard {
    fn begin(&mut self, scene: &pdfcer_render::GpuScene) -> Result<u64, String> {
        let job = self
            .drawer
            .begin(&self.gl, texture(scene), [scene.width, scene.height])?;
        self.next += 1;
        self.jobs.insert(self.next, job);
        Ok(self.next)
    }

    fn step(
        &mut self,
        id: u64,
        scene: &pdfcer_render::GpuScene,
    ) -> Result<Option<Vec<u8>>, String> {
        let Some(job) = self.jobs.get_mut(&id) else {
            return Err(format!("no job {id}")); // ui-text-exempt: driver-words slot
        };
        let stepped = self.drawer.step(&self.gl, job, texture(scene), false);
        match stepped {
            Ok(Step::Pending) => Ok(None),
            done => {
                self.abandon(id);
                done.map(|step| match step {
                    Step::Done(bytes) => Some(bytes),
                    Step::Pending => None,
                })
            }
        }
    }

    fn abandon(&mut self, id: u64) {
        if let Some(job) = self.jobs.remove(&id) {
            job.destroy(&self.gl);
        }
    }
}

/// Once a frame, first thing: make the drawer if there is none, and ask for a
/// frame when one has just failed so the canvas asks the processor at once.
pub fn keep(ctx: &egui::Context, gl: Option<&Arc<glow::Context>>) {
    if rendercard::take_retirement() {
        ctx.request_repaint();
    }
    if rendercard::installed() {
        return;
    }
    let Some(gl) = gl else {
        return;
    };
    match SceneDrawer::new(
        gl,
        pdfcer_render::GLSL_ES_300_VERTEX,
        pdfcer_render::GLSL_ES_300_FRAGMENT,
    ) {
        Ok(drawer) => rendercard::install(Box::new(GlCard {
            gl: Arc::clone(gl),
            drawer,
            jobs: HashMap::new(),
            next: 0,
        })),
        Err(why) => rendercard::refuse(crate::text::status::card_failed(&why)),
    }
}
