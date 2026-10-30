//! The viewer's pictures: the one on screen, redrawn when the camera, its
//! size or the shown parts change, and the one made for the page or a file.

use egui::TextureOptions;
// The 3D renderer's options, not the page renderer's that settings own.
use pdfcer_3d::RenderOptions as ModelRenderOptions;
use pdfcer_3d::render_model;

use super::{Drawn, ModelView, POSTER_SIDE, Rendered};
use crate::text::panels::models as t;

impl ModelView {
    /// This view, at the picture's shape, on the white the engine's own
    /// poster uses.
    pub(super) fn poster(&mut self) -> Result<Drawn, String> {
        let [w, h] = self.rendered.map_or([4, 3], |r| r.size);
        let aspect = f64::from(w.max(1)) / f64::from(h.max(1));
        let (width, height) = if aspect >= 1.0 {
            (POSTER_SIDE, POSTER_SIDE / aspect)
        } else {
            (POSTER_SIDE * aspect, POSTER_SIDE)
        };
        let options = ModelRenderOptions {
            width: (width.round() as u32).max(1),
            height: (height.round() as u32).max(1),
            ..ModelRenderOptions::default()
        };
        let camera = self
            .orbit
            .camera(&self.bounds, aspect, self.aimed.as_ref())
            .map_err(|e| e.to_string())?;
        let image = self
            .parts
            .painted(&mut self.model.drawn, None, |model| {
                render_model(model, &camera, &options)
            })
            .map_err(|e| e.to_string())?;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-view-poster w={} h={} yaw={:.3} pitch={:.3} bytes={}",
                image.width,
                image.height,
                self.orbit.yaw,
                self.orbit.pitch,
                image.rgba.len()
            )
        });
        Ok(Drawn {
            width: image.width,
            height: image.height,
            rgba: image.rgba,
        })
    }

    pub(super) fn render(&mut self, ctx: &egui::Context, wanted: Rendered) {
        let [width, height] = wanted.size;
        let options = ModelRenderOptions {
            width,
            height,
            background: wanted.background.to_array(),
            ..ModelRenderOptions::default()
        };
        let camera = self.orbit.camera(
            &self.bounds,
            f64::from(width) / f64::from(height),
            self.aimed.as_ref(),
        );
        let aimed = camera.as_ref().map(aim_of).unwrap_or_default();
        let accent = Some(wanted.accent.to_array());
        let parts = &self.parts;
        let model = &mut self.model.drawn;
        let drawn = camera.and_then(|camera| {
            parts.painted(model, accent, |model| {
                render_model(model, &camera, &options)
            })
        });
        self.rendered = Some(wanted);
        match drawn {
            Ok(image) => {
                let covered = image
                    .rgba
                    .chunks_exact(4)
                    .filter(|px| *px != options.background)
                    .count();
                let (chromatic, hues) = colourfulness(&image.rgba);
                let hash = image.rgba.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
                    (h ^ u64::from(*b)).wrapping_mul(0x100_0000_01b3)
                });
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!(
                        "model-view-rendered w={width} h={height} yaw={:.3} pitch={:.3} zoom={:.3} pan={:.4},{:.4} perspective={} framed={} {aimed} covered={covered} chromatic={chromatic} hues={hues} hash={hash:016x}",
                        self.orbit.yaw,
                        self.orbit.pitch,
                        self.orbit.zoom,
                        self.orbit.pan[0],
                        self.orbit.pan[1],
                        self.orbit.perspective,
                        self.orbit.framed
                    )
                });
                crate::render::pressure::record_other(
                    ctx,
                    crate::render::pressure::Surface::ModelView,
                    width,
                    height,
                );
                let colour = egui::ColorImage::from_rgba_unmultiplied(
                    [width as usize, height as usize],
                    &image.rgba,
                );
                match &mut self.texture {
                    Some(texture) => texture.set(colour, TextureOptions::LINEAR),
                    None => {
                        self.texture =
                            Some(ctx.load_texture("model-3d", colour, TextureOptions::LINEAR)); // ui-text-exempt: texture id, never displayed
                    }
                }
                self.failed = None;
            }
            Err(error) => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!("model-view-render-failed error={error:?}")
                });
                self.failed = Some(t::view_render_failed(&error.to_string()));
            }
        }
    }
}

/// A camera's unit look direction and up, as `dir=x,y,z up=x,y,z`, for the
/// trace a driven check reads.
fn aim_of(camera: &pdfcer_3d::Camera) -> String {
    let unit = |v: [f64; 3]| {
        let length = v
            .iter()
            .map(|c| c * c)
            .sum::<f64>()
            .sqrt()
            .max(f64::MIN_POSITIVE);
        format!(
            "{:.4},{:.4},{:.4}",
            v[0] / length,
            v[1] / length,
            v[2] / length
        )
    };
    let look = std::array::from_fn(|i| camera.target[i] - camera.eye[i]);
    // ui-text-exempt: diagnostic trace, never displayed
    format!("dir={} up={}", unit(look), unit(camera.up))
}

/// A channel spread at or above this is a colour, not a shade of grey.
const CHROMA: f32 = 48.0;

/// How many pixels of `rgba` are coloured rather than grey, and how many of
/// twelve 30-degree hue sectors hold at least 1% of them, for the trace a
/// driven check reads.
fn colourfulness(rgba: &[u8]) -> (usize, usize) {
    let mut sectors = [0_usize; 12];
    let mut chromatic = 0;
    for px in rgba.chunks_exact(4) {
        let (r, g, b) = (f32::from(px[0]), f32::from(px[1]), f32::from(px[2]));
        let (max, min) = (r.max(g).max(b), r.min(g).min(b));
        let spread = max - min;
        if spread < CHROMA {
            continue;
        }
        chromatic += 1;
        let top = px[0].max(px[1]).max(px[2]);
        let hue = if px[0] == top {
            ((g - b) / spread).rem_euclid(6.0)
        } else if px[1] == top {
            (b - r) / spread + 2.0
        } else {
            (r - g) / spread + 4.0
        };
        // `hue` is in [0, 6): two sectors per unit.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let sector = ((hue * 2.0) as usize).min(11);
        sectors[sector] += 1;
    }
    let floor = (chromatic / 100).max(1);
    (chromatic, sectors.iter().filter(|n| **n >= floor).count())
}

/// `drawn` as PNG file bytes, for *Save picture…*.
pub(super) fn png(drawn: &Drawn) -> Result<Vec<u8>, String> {
    use pdfcer_render::tiny_skia::{ColorU8, IntSize, Pixmap};
    let premultiplied = drawn
        .rgba
        .chunks_exact(4)
        .flat_map(|p| {
            let c = ColorU8::from_rgba(p[0], p[1], p[2], p[3]).premultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect();
    let size = IntSize::from_wh(drawn.width, drawn.height).ok_or_else(String::new)?;
    let pixmap = Pixmap::from_vec(premultiplied, size).ok_or_else(String::new)?;
    pdfcer_render::export::encode_png(&pixmap, None).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grey_is_not_colour_and_two_hues_count_twice() {
        let grey = [90, 90, 90, 255, 200, 200, 200, 255];
        assert_eq!(colourfulness(&grey), (0, 0));
        let red_and_blue = [220, 30, 30, 255, 30, 30, 220, 255, 128, 128, 128, 255];
        assert_eq!(colourfulness(&red_and_blue), (2, 2));
    }
}
