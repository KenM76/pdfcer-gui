//! The viewer's pictures: the one on screen, redrawn when the camera, its
//! size or the shown parts change, and the one made for the page or a file.

use egui::TextureOptions;
// The 3D renderer's options, not the page renderer's that settings own.
use pdfcer_3d::RenderOptions as ModelRenderOptions;
use pdfcer_3d::render_model;

use super::effects::Effects;
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
            threads: 0,
            ..self.look().options()
        };
        let camera = self.view_camera(aspect).map_err(|e| e.to_string())?;
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
            threads: 0,
            ..wanted.look.options()
        };
        let camera = self.view_camera(f64::from(width) / f64::from(height));
        let started = std::time::Instant::now();
        let head = self.trace_head(&wanted, camera.as_ref(), &options);
        let accent = Some(wanted.accent.to_array());
        let parts = &self.parts;
        let model = &mut self.model.drawn;
        let drawn = camera.and_then(|camera| {
            parts.painted(model, accent, |model| {
                render_model(model, &camera, &options)
            })
        });
        let ms = started.elapsed().as_secs_f64() * 1000.0;
        self.rendered = Some(wanted);
        match drawn {
            Ok(image) => {
                let fields = pixel_fields(&image.rgba, options.background);
                crate::diag::trace(|| format!("{head} ms={ms:.1}{fields}")); // ui-text-exempt: diagnostic trace, never displayed
                self.show_image(ctx, [width, height], &image.rgba);
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

    /// The `model-view-rendered` trace line up to its timing: the size, the
    /// orbit, the aim, the pivot, the walker's eye, the lighting, and the
    /// shadow and occlusion `options` draws.
    pub(super) fn trace_head(
        &self,
        wanted: &Rendered,
        camera: Result<&pdfcer_3d::Camera, &pdfcer_3d::RenderError>,
        options: &ModelRenderOptions,
    ) -> String {
        let [width, height] = wanted.size;
        let aimed = camera.map(aim_of).unwrap_or_default();
        let pivot = camera
            .ok()
            .zip(self.orbit.anchor)
            .map(|(c, a)| pivot_shown(c, a.point))
            .unwrap_or_default();
        let effects = camera
            .ok()
            .and_then(|c| {
                let lights = &self.model.drawn.lights;
                Effects::new(c, options, &self.bounds, lights, wanted.size).ok()
            })
            .map_or_else(String::new, |e| e.trace(options));
        let declined = self.gpu.declined().map_or("", super::looks::declined_word);
        let lit = self.lighting_trace(options.lighting)
            + &effects
            + &super::looks::trace(options)
            + declined;
        let eye = camera_eye(wanted.walk.as_ref());
        format!(
            // ui-text-exempt: diagnostic trace, never displayed
            "model-view-rendered w={width} h={height} yaw={:.3} pitch={:.3} zoom={:.3} pan={:.4},{:.4} perspective={} framed={} {aimed}{pivot}{eye}{lit}",
            self.orbit.yaw,
            self.orbit.pitch,
            self.orbit.zoom,
            self.orbit.pan[0],
            self.orbit.pan[1],
            self.orbit.perspective,
            self.orbit.framed
        )
    }

    /// Puts a drawn `[width, height]` RGBA picture on screen.
    fn show_image(&mut self, ctx: &egui::Context, [width, height]: [u32; 2], rgba: &[u8]) {
        let colour =
            egui::ColorImage::from_rgba_unmultiplied([width as usize, height as usize], rgba);
        crate::render::pressure::record_other(
            ctx,
            crate::render::pressure::Surface::ModelView,
            width,
            height,
        );
        match &mut self.texture {
            Some(texture) => texture.set(colour, TextureOptions::LINEAR),
            None => {
                self.texture = Some(ctx.load_texture("model-3d", colour, TextureOptions::LINEAR)); // ui-text-exempt: texture id, never displayed
            }
        }
        self.failed = None;
    }
}

/// A camera's unit look direction and up, the point it looks at and its
/// distance from it, as `dir=x,y,z up=x,y,z target=x,y,z gap=d`, for the
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
    let gap = look.iter().map(|c: &f64| c * c).sum::<f64>().sqrt();
    let t = camera.target;
    let fov = match camera.projection {
        pdfcer_3d::Projection::Perspective { fov_y } => format!(" fov-y={fov_y:.3}"), // ui-text-exempt: diagnostic trace
        pdfcer_3d::Projection::Fisheye { fov, mapping } => {
            format!(" lens=fisheye-{mapping:?} fov={fov:.3}").to_lowercase() // ui-text-exempt: diagnostic trace
        }
        pdfcer_3d::Projection::Orthographic { .. } => String::new(),
    };
    format!(
        "dir={} up={} target={:.4},{:.4},{:.4} gap={gap:.4}{fov}", // ui-text-exempt: diagnostic trace
        unit(look),
        unit(camera.up),
        t[0],
        t[1],
        t[2]
    )
}

/// ` pivot=x,y,z pivot-shown=r,u`: the point an orbit turns about and where
/// it shows, right and up of the picture's centre as a share of the visible
/// height at its depth, for the trace a driven check reads.
fn pivot_shown(camera: &pdfcer_3d::Camera, p: [f64; 3]) -> String {
    let sub = |a: [f64; 3], b: [f64; 3]| -> [f64; 3] { std::array::from_fn(|i| a[i] - b[i]) };
    let dot = |a: [f64; 3], b: [f64; 3]| a.iter().zip(b).map(|(x, y)| x * y).sum::<f64>();
    let unit = |v: [f64; 3]| {
        let l = dot(v, v).sqrt().max(f64::MIN_POSITIVE);
        v.map(|c| c / l)
    };
    let cross = |a: [f64; 3], b: [f64; 3]| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let f = unit(sub(camera.target, camera.eye));
    let r = unit(cross(f, camera.up));
    let u = cross(r, f);
    let v = sub(p, camera.eye);
    let h = match camera.projection {
        pdfcer_3d::Projection::Perspective { fov_y } => {
            2.0 * dot(v, f) * (fov_y / 2.0).to_radians().tan()
        }
        pdfcer_3d::Projection::Fisheye { fov, .. } => {
            2.0 * dot(v, f) * (fov.min(179.0) / 2.0).to_radians().tan()
        }
        pdfcer_3d::Projection::Orthographic { height } => height,
    }
    .max(f64::MIN_POSITIVE);
    format!(
        // ui-text-exempt: diagnostic trace, never displayed
        " pivot={:.4},{:.4},{:.4} pivot-shown={:.5},{:.5}",
        p[0],
        p[1],
        p[2],
        dot(v, r) / h,
        dot(v, u) / h
    )
}

/// ` walk=1 eye=x,y,z` while walking, empty while orbiting, for the trace.
fn camera_eye(walk: Option<&super::walk::Walk>) -> String {
    walk.map_or_else(String::new, |w| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed
            " walk=1 eye={:.4},{:.4},{:.4}",
            w.eye[0], w.eye[1], w.eye[2]
        )
    })
}

/// ` covered=n chromatic=n hues=n hash=h` of a drawn picture on
/// `background`, for the trace a driven check reads.
pub(super) fn pixel_fields(rgba: &[u8], background: [u8; 4]) -> String {
    let covered = rgba.chunks_exact(4).filter(|px| *px != background).count();
    let (chromatic, hues) = colourfulness(rgba);
    let hash = rgba.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100_0000_01b3)
    });
    // ui-text-exempt: diagnostic trace fields, never displayed
    format!(" covered={covered} chromatic={chromatic} hues={hues} hash={hash:016x}")
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

/// `drawn` as PNG file bytes at `dpi` pixels per inch, for *Save picture…*.
pub(super) fn png(drawn: &Drawn, dpi: f32) -> Result<Vec<u8>, String> {
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
    pdfcer_render::export::encode_png(&pixmap, Some(dpi)).map_err(|e| e.to_string())
}

/// The picture's pixel size for an area of `points` at `scale` pixels a
/// point: both sides shrunk by one factor so the longer fits `max_side`,
/// keeping the area's shape, since the camera's aspect comes from this size
/// and the picture is shown stretched over the whole area.
pub(super) fn render_size(points: [f32; 2], scale: f32, max_side: f32) -> [u32; 2] {
    let px = points.map(|p| (p * scale).max(1.0));
    let k = (max_side / px[0].max(px[1])).min(1.0);
    px.map(|p| (p * k).round().max(1.0) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_capped_picture_keeps_the_areas_shape() {
        for (w, h) in [
            (1920.0, 1000.0),
            (2560.0, 1300.0),
            (3840.0, 2100.0),
            (500.0, 400.0),
        ] {
            let [pw, ph] = render_size([w, h], 1.0, 1600.0);
            assert!(pw.max(ph) <= 1600, "{w}x{h} -> {pw}x{ph}");
            let want = f64::from(w / h);
            let got = f64::from(pw) / f64::from(ph);
            assert!(
                (want - got).abs() < 2.0 / f64::from(ph),
                "{w}x{h} -> {pw}x{ph}"
            );
        }
        assert_eq!(render_size([500.0, 400.0], 2.0, 1600.0), [1000, 800]);
    }

    #[test]
    fn grey_is_not_colour_and_two_hues_count_twice() {
        let grey = [90, 90, 90, 255, 200, 200, 200, 255];
        assert_eq!(colourfulness(&grey), (0, 0));
        let red_and_blue = [220, 30, 30, 255, 30, 30, 220, 255, 128, 128, 128, 255];
        assert_eq!(colourfulness(&red_and_blue), (2, 2));
    }
}
