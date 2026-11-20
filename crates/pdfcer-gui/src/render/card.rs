//! # `render::card` — the canvas's graphics-card drawer
//!
//! Makes the [`pdfcer_gui_base::rendercard::CardDraw`] the canvas's pages are
//! drawn with when Settings ▸ Display ▸ Graphics card is on: the engine's two
//! shaders over `native_mesh::scene::SceneDrawer`, on the context egui paints
//! with. Made once, on the first frame with a context; a shader the driver
//! refuses retires the card for the session with the driver's words.

use std::sync::Arc;

use eframe::glow;
use native_mesh::scene::{SceneDrawer, SceneTexture};
use pdfcer_gui_base::rendercard::{self, CardDraw};

/// The drawer and the context it was made on.
struct GlCard {
    gl: Arc<glow::Context>,
    drawer: SceneDrawer,
}

impl CardDraw for GlCard {
    fn draw(&mut self, scene: &pdfcer_render::GpuScene) -> Result<Vec<u8>, String> {
        let data = &scene.data;
        let texture = SceneTexture {
            width: data.width,
            height: data.height,
            layers: data.layers,
            texels: &data.texels,
        };
        self.drawer
            .draw(&self.gl, texture, [scene.width, scene.height])
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
        })),
        Err(why) => rendercard::refuse(crate::text::status::card_failed(&why)),
    }
}
