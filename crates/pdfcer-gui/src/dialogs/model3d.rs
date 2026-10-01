//! # `dialogs::model3d` — **the 3D model viewer**
//!
//! Opened by *View…* on a PRC row of the Attachments panel's 3D models
//! section. The engine's software renderer draws the placed model from a
//! camera this window moves: drag orbits, right-drag pans, scroll zooms, and
//! five named views jump to a side. A still image is rendered only when the
//! camera or the picture's size changes.
//!
//! Design: `docs/modules/pdfcer-gui/dialogs/model3d.md`.

use std::f64::consts::FRAC_PI_2;

use egui::{Sense, TextureHandle, TextureOptions, Ui};
// The 3D renderer's options, not the page renderer's that settings own.
use pdfcer_3d::RenderOptions as ModelRenderOptions;
use pdfcer_3d::{Bounds, Camera, Projection, render};

use crate::app::actions::models::Assembled;
use crate::text::panels::models as t;

/// The picture's published region, for `ui-verify`.
pub const REGION_IMAGE: &str = "model3d.image"; // ui-text-exempt: trace region name, never displayed
/// One named view's button; the suffix is its index in [`VIEWS`].
pub const REGION_VIEW_PREFIX: &str = "model3d.view."; // ui-text-exempt: trace region name, never displayed
/// The Fit button.
pub const REGION_FIT: &str = "model3d.fit"; // ui-text-exempt: trace region name, never displayed

/// The named views as (yaw, pitch) in degrees, in `t::view_names` order.
/// Yaw 0 looks along +y (the front of a z-up model); pitch looks down.
pub const VIEWS: [(f64, f64); 5] = [
    (45.0, 35.264),
    (0.0, 0.0),
    (90.0, 0.0),
    (0.0, 89.0),
    (180.0, 0.0),
];

/// Radians turned per point dragged.
const ORBIT_RATE: f64 = 0.01;
/// Kept off ±90° so the view never looks straight along the z-up axis.
const PITCH_LIMIT: f64 = FRAC_PI_2 - 0.01;
/// The largest picture rendered, in pixels a side; the CPU renderer's cost
/// grows with the area.
const MAX_SIDE: f32 = 1600.0;

/// Where the camera is, relative to a fitted view.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Orbit {
    /// Radians around z.
    yaw: f64,
    /// Radians above the horizon, looking down when positive.
    pitch: f64,
    /// 1 frames the whole model.
    zoom: f64,
    /// The target's shift along the image's right and up, in model radii.
    pan: [f64; 2],
    perspective: bool,
}

impl Orbit {
    fn named(index: usize, perspective: bool) -> Self {
        let (yaw, pitch) = VIEWS[index];
        Self {
            yaw: yaw.to_radians(),
            pitch: pitch.to_radians(),
            zoom: 1.0,
            pan: [0.0, 0.0],
            perspective,
        }
    }

    /// The unit direction the camera looks along.
    fn direction(&self) -> [f64; 3] {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        [-sy * cp, cy * cp, -sp]
    }

    /// The camera for `bounds` in an image of `aspect` (width / height).
    fn camera(&self, bounds: &Bounds, aspect: f64) -> Result<Camera, pdfcer_3d::RenderError> {
        let dir = self.direction();
        let up = [0.0, 0.0, 1.0];
        let mut camera = Camera::fit(bounds, dir, up, self.perspective, aspect)?;
        let radius = match bounds.radius() {
            r if r.is_finite() && r > 0.0 => r,
            _ => 1.0,
        };
        let right = normalise(cross(dir, up));
        let image_up = cross(right, dir);
        let shift: [f64; 3] =
            std::array::from_fn(|i| (right[i] * self.pan[0] + image_up[i] * self.pan[1]) * radius);
        let target: [f64; 3] = std::array::from_fn(|i| camera.target[i] + shift[i]);
        camera.eye =
            std::array::from_fn(|i| target[i] + (camera.eye[i] - camera.target[i]) / self.zoom);
        camera.target = target;
        if let Projection::Orthographic { height } = camera.projection {
            camera.projection = Projection::Orthographic {
                height: height / self.zoom,
            };
        }
        Ok(camera)
    }
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalise(v: [f64; 3]) -> [f64; 3] {
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if length > 0.0 {
        v.map(|c| c / length)
    } else {
        [1.0, 0.0, 0.0]
    }
}

/// What the current picture was rendered for.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Rendered {
    orbit: Orbit,
    size: [u32; 2],
    background: egui::Color32,
}

/// The 3D viewer's state.
pub(crate) struct ModelView {
    title: String,
    model: Assembled,
    bounds: Bounds,
    orbit: Orbit,
    texture: Option<TextureHandle>,
    rendered: Option<Rendered>,
    failed: Option<String>,
    close_requested: bool,
}

impl ModelView {
    /// Open on `model`, from page `page_index`, at the isometric view.
    #[must_use]
    pub(crate) fn open(page_index: usize, model: Assembled) -> Self {
        let bounds = Bounds::of(&model.meshes).unwrap_or(Bounds {
            min: [0.0; 3],
            max: [0.0; 3],
        });
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-view-opened page={page_index} parts={} triangles={} skipped={} placed={}",
                model.meshes.len(),
                model.triangles,
                model.skipped,
                model.placed
            )
        });
        Self {
            title: t::view_title(page_index),
            model,
            bounds,
            orbit: Orbit::named(0, true),
            texture: None,
            rendered: None,
            failed: None,
            close_requested: false,
        }
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "model-3d", // ui-text-exempt: a viewport key, never displayed.
            &self.title,
            egui::vec2(720.0, 600.0),
            egui::vec2(360.0, 320.0),
        )
        .show(ctx, |ui| self.body(ui));
        !frame.closed && !std::mem::take(&mut self.close_requested)
    }

    fn body(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            for (i, name) in t::view_names().into_iter().enumerate() {
                let button = ui.button(name);
                crate::diag::ui_rect_visible(
                    // ui-text-exempt: trace region name, never displayed
                    &format!("{REGION_VIEW_PREFIX}{i}"),
                    button.rect,
                    ui.clip_rect(),
                );
                if button.clicked() {
                    self.orbit = Orbit::named(i, self.orbit.perspective);
                }
            }
            let fit = ui
                .button(t::view_reset())
                .on_hover_text(t::view_reset_tooltip());
            crate::diag::ui_rect_visible(REGION_FIT, fit.rect, ui.clip_rect());
            if fit.clicked() {
                self.orbit.zoom = 1.0;
                self.orbit.pan = [0.0, 0.0];
            }
            ui.checkbox(&mut self.orbit.perspective, t::view_perspective())
                .on_hover_text(t::view_perspective_tooltip());
        });
        ui.small(t::view_hint());

        let footer = ui.spacing().interact_size.y * 3.0;
        let area = egui::vec2(
            ui.available_width(),
            (ui.available_height() - footer).max(64.0),
        );
        let (rect, response) = ui.allocate_exact_size(area, Sense::click_and_drag());
        crate::diag::ui_rect_visible(REGION_IMAGE, rect, ui.clip_rect());
        self.steer(ui, &response);

        let background = ui.visuals().extreme_bg_color;
        let scale = ui.ctx().pixels_per_point();
        let side = |points: f32| (points * scale).clamp(1.0, MAX_SIDE).round() as u32;
        let size = [side(rect.width()), side(rect.height())];
        let wanted = Rendered {
            orbit: self.orbit,
            size,
            background,
        };
        if self.rendered != Some(wanted) {
            self.render(ui.ctx(), wanted);
        }
        match (&self.texture, &self.failed) {
            (_, Some(said)) => {
                ui.put(rect, egui::Label::new(said.as_str()));
            }
            (Some(texture), None) => {
                ui.painter().image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE, // NOT A THEME COLOUR: pass-through tint over the rendered picture
                );
            }
            (None, None) => {}
        }

        ui.small(t::view_census(
            self.model.meshes.len(),
            self.model.triangles,
        ));
        ui.small(t::view_flat_note());
        if !self.model.placed {
            ui.small(t::mesh_placement_note());
        }
        if self.model.skipped > 0 {
            ui.small(t::mesh_skipped(self.model.skipped));
        }
        if ui.button(t::view_close()).clicked() {
            self.close_requested = true;
        }
    }

    /// Turn the camera from this frame's pointer on the picture.
    fn steer(&mut self, ui: &Ui, response: &egui::Response) {
        let delta = response.drag_delta();
        if response.dragged_by(egui::PointerButton::Primary) && delta != egui::Vec2::ZERO {
            self.orbit.yaw -= f64::from(delta.x) * ORBIT_RATE;
            self.orbit.pitch = (self.orbit.pitch + f64::from(delta.y) * ORBIT_RATE)
                .clamp(-PITCH_LIMIT, PITCH_LIMIT);
        }
        if (response.dragged_by(egui::PointerButton::Secondary)
            || response.dragged_by(egui::PointerButton::Middle))
            && delta != egui::Vec2::ZERO
        {
            let per_point = 2.0 / f64::from(response.rect.height().max(1.0)) / self.orbit.zoom;
            self.orbit.pan[0] -= f64::from(delta.x) * per_point;
            self.orbit.pan[1] += f64::from(delta.y) * per_point;
        }
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                self.orbit.zoom =
                    (self.orbit.zoom * f64::from(scroll / 200.0).exp()).clamp(0.05, 50.0);
            }
        }
    }

    fn render(&mut self, ctx: &egui::Context, wanted: Rendered) {
        let [width, height] = wanted.size;
        let options = ModelRenderOptions {
            width,
            height,
            background: wanted.background.to_array(),
            ..ModelRenderOptions::default()
        };
        let drawn = self
            .orbit
            .camera(&self.bounds, f64::from(width) / f64::from(height))
            .and_then(|camera| render(&self.model.meshes, &camera, &options));
        self.rendered = Some(wanted);
        match drawn {
            Ok(image) => {
                let covered = image
                    .rgba
                    .chunks_exact(4)
                    .filter(|px| *px != options.background)
                    .count();
                let hash = image.rgba.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
                    (h ^ u64::from(*b)).wrapping_mul(0x100_0000_01b3)
                });
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!(
                        "model-view-rendered w={width} h={height} yaw={:.3} pitch={:.3} zoom={:.3} perspective={} covered={covered} hash={hash:016x}",
                        self.orbit.yaw, self.orbit.pitch, self.orbit.zoom, self.orbit.perspective
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_named_views_look_the_way_their_names_say() {
        let front = Orbit::named(1, false).direction();
        assert!(
            (front[1] - 1.0).abs() < 1e-9,
            "front looks along +y: {front:?}"
        );
        let top = Orbit::named(3, false).direction();
        assert!(top[2] < -0.99, "top looks down: {top:?}");
        let iso = Orbit::named(0, false).direction();
        assert!(iso[0] < 0.0 && iso[1] > 0.0 && iso[2] < 0.0, "{iso:?}");
    }

    #[test]
    fn zoom_brings_the_eye_closer_and_pan_moves_the_target() {
        let bounds = Bounds {
            min: [-1.0; 3],
            max: [1.0; 3],
        };
        let mut orbit = Orbit::named(1, true);
        let fitted = orbit.camera(&bounds, 1.0).expect("a view forms");
        orbit.zoom = 2.0;
        orbit.pan = [0.5, 0.0];
        let moved = orbit.camera(&bounds, 1.0).expect("a view forms");
        let gap = |c: &Camera| {
            (0..3)
                .map(|i| (c.eye[i] - c.target[i]).powi(2))
                .sum::<f64>()
                .sqrt()
        };
        assert!((gap(&moved) - gap(&fitted) / 2.0).abs() < 1e-9);
        assert!(moved.target != fitted.target);
        assert!(
            moved.target[2].abs() < 1e-9,
            "a sideways pan keeps the height"
        );
    }
}
