//! # `dialogs::model3d` — **the 3D model viewer**
//!
//! Opened by *View…* on a PRC row of the Attachments panel's 3D models
//! section. The engine's software renderer draws the placed model from a
//! camera this window moves: drag orbits, right-drag pans, scroll zooms about
//! the point under the pointer, and five named views jump to a side. The
//! window maximises from its title bar and fills the screen from its button or
//! F11; Escape leaves full screen before it closes the window. A still image is
//! rendered only when the camera or the picture's size changes.
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
/// The Close button.
pub const REGION_CLOSE: &str = "model3d.close"; // ui-text-exempt: trace region name, never displayed
/// The Minimize button.
pub const REGION_MINIMIZE: &str = "model3d.minimize"; // ui-text-exempt: trace region name, never displayed
/// The full-screen button.
pub const REGION_FULL_SCREEN: &str = "model3d.full_screen"; // ui-text-exempt: trace region name, never displayed

/// The viewer's OS window key.
const VIEWPORT_KEY: &str = "model-3d"; // ui-text-exempt: a viewport key, never displayed

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
/// The zoom range, as a multiple of the fitted view.
const ZOOM_RANGE: (f64, f64) = (0.05, 50.0);

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

    /// Multiply the zoom by `factor`, keeping the model point under the
    /// pointer where it is on the picture.
    ///
    /// `offset` is the pointer's position from the picture's centre, right and
    /// up, in picture heights. Exact on the plane through the target facing
    /// the camera, the plane the pan moves in.
    fn zoom_at(&mut self, bounds: &Bounds, aspect: f64, factor: f64, offset: [f64; 2]) {
        let before = self.zoom;
        let after = (before * factor).clamp(ZOOM_RANGE.0, ZOOM_RANGE.1);
        let fitted = Self {
            zoom: 1.0,
            pan: [0.0, 0.0],
            ..*self
        };
        // The fitted view's visible height, in model radii: what one picture
        // height spans at zoom 1.
        let span = fitted
            .camera(bounds, aspect)
            .map_or(2.0, |c| visible_height(&c))
            / radius(bounds);
        for (pan, off) in self.pan.iter_mut().zip(offset) {
            *pan += off * span * (1.0 / before - 1.0 / after);
        }
        self.zoom = after;
    }

    /// The camera for `bounds` in an image of `aspect` (width / height).
    fn camera(&self, bounds: &Bounds, aspect: f64) -> Result<Camera, pdfcer_3d::RenderError> {
        let dir = self.direction();
        let up = [0.0, 0.0, 1.0];
        let mut camera = Camera::fit(bounds, dir, up, self.perspective, aspect)?;
        let radius = radius(bounds);
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

/// The model's radius, or 1 for a model with no extent.
fn radius(bounds: &Bounds) -> f64 {
    match bounds.radius() {
        r if r.is_finite() && r > 0.0 => r,
        _ => 1.0,
    }
}

/// How much of the target plane the picture shows vertically, in model units.
fn visible_height(camera: &Camera) -> f64 {
    match camera.projection {
        Projection::Orthographic { height } => height,
        Projection::Perspective { fov_y } => {
            let distance = (0..3)
                .map(|i| (camera.eye[i] - camera.target[i]).powi(2))
                .sum::<f64>()
                .sqrt();
            2.0 * distance * (fov_y.to_radians() / 2.0).tan()
        }
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
    /// The minimised state last traced.
    minimized: Option<bool>,
    /// This frame's Escape left full screen, so it must not close the window.
    escape_left_full_screen: bool,
    /// The host drew a real OS window, the only kind that can fill the screen.
    native: bool,
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
            minimized: None,
            escape_left_full_screen: false,
            native: false,
        }
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            VIEWPORT_KEY,
            &self.title,
            egui::vec2(720.0, 600.0),
            egui::vec2(360.0, 320.0),
        )
        .maximizable()
        .minimizable()
        .show(ctx, |ui| self.body(ui));
        self.native = frame.class == egui::ViewportClass::Immediate;
        let closed = frame.closed && !std::mem::take(&mut self.escape_left_full_screen);
        let button = std::mem::take(&mut self.close_requested);
        if closed || button {
            let how = if button { "button" } else { "window" };
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| format!("model-view-closed how={how}"));
        }
        !closed && !button
    }

    /// Full screen from the button or F11, and Escape out of it: the
    /// conventional viewer keys. Only on a real OS window; an embedded
    /// fallback has no screen of its own to fill.
    fn full_screen_control(&mut self, ui: &mut Ui) {
        use pdfcer_gui_base::windowshape;
        if !self.native {
            return;
        }
        let ctx = ui.ctx().clone();
        self.minimize_control(ui);
        let full = windowshape::fullscreen_believed(&ctx);
        let (label, tip) = if full {
            (
                t::view_full_screen_leave(),
                t::view_full_screen_leave_tooltip(),
            )
        } else {
            (t::view_full_screen(), t::view_full_screen_tooltip())
        };
        let button = ui.button(label).on_hover_text(tip);
        crate::diag::ui_rect_visible(REGION_FULL_SCREEN, button.rect, ui.clip_rect());
        let (f11, escape, closing) = ui.input(|i| {
            (
                i.key_pressed(egui::Key::F11),
                i.key_pressed(egui::Key::Escape),
                i.viewport().close_requested(),
            )
        });
        let wanted = if button.clicked() || f11 {
            Some(!full)
        } else if full && escape && !closing {
            self.escape_left_full_screen = true;
            Some(false)
        } else {
            None
        };
        if let Some(on) = wanted {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "model-view-full-screen asked={on} escape={}",
                    full && escape
                )
            });
            windowshape::set_fullscreen(&ctx, on);
        }
    }

    /// A Minimize button, since full screen has no title bar, and a trace of
    /// the window's minimised state as the OS reports it, with the camera.
    fn minimize_control(&mut self, ui: &mut Ui) {
        let button = ui
            .button(t::view_minimize())
            .on_hover_text(t::view_minimize_tooltip());
        crate::diag::ui_rect_visible(REGION_MINIMIZE, button.rect, ui.clip_rect());
        if button.clicked() {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }
        let minimized = ui.input(|i| i.viewport().minimized);
        if minimized.is_some() && minimized != self.minimized {
            self.minimized = minimized;
            let o = &self.orbit;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "model-view-window minimized={} yaw={:.3} pitch={:.3} zoom={:.3} pan={:.4},{:.4}",
                    minimized == Some(true),
                    o.yaw,
                    o.pitch,
                    o.zoom,
                    o.pan[0],
                    o.pan[1]
                )
            });
        }
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
            self.full_screen_control(ui);
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
        let close = ui.button(t::view_close());
        crate::diag::ui_rect_visible(REGION_CLOSE, close.rect, ui.clip_rect());
        if close.clicked() {
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
        if let Some(at) = response.hover_pos() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                let rect = response.rect;
                let height = f64::from(rect.height().max(1.0));
                let offset = [
                    f64::from(at.x - rect.center().x) / height,
                    f64::from(rect.center().y - at.y) / height,
                ];
                let aspect = f64::from(rect.width().max(1.0)) / height;
                let factor = f64::from(scroll / 200.0).exp();
                self.orbit.zoom_at(&self.bounds, aspect, factor, offset);
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
                        "model-view-rendered w={width} h={height} yaw={:.3} pitch={:.3} zoom={:.3} pan={:.4},{:.4} perspective={} covered={covered} hash={hash:016x}",
                        self.orbit.yaw,
                        self.orbit.pitch,
                        self.orbit.zoom,
                        self.orbit.pan[0],
                        self.orbit.pan[1],
                        self.orbit.perspective
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

    /// The model point on the target plane at `offset` (picture heights,
    /// right and up from the centre).
    fn under(orbit: &Orbit, bounds: &Bounds, aspect: f64, offset: [f64; 2]) -> [f64; 3] {
        let camera = orbit.camera(bounds, aspect).expect("a view forms");
        let dir = orbit.direction();
        let right = normalise(cross(dir, [0.0, 0.0, 1.0]));
        let up = cross(right, dir);
        let h = visible_height(&camera);
        std::array::from_fn(|i| camera.target[i] + (right[i] * offset[0] + up[i] * offset[1]) * h)
    }

    #[test]
    fn a_wheel_zoom_keeps_the_point_under_the_pointer_still() {
        let bounds = Bounds {
            min: [-1.0, -2.0, -0.5],
            max: [3.0, 1.0, 2.0],
        };
        for perspective in [true, false] {
            let mut orbit = Orbit::named(0, perspective);
            orbit.pan = [0.1, -0.2];
            let (aspect, offset) = (1.6, [0.3, -0.15]);
            let before = under(&orbit, &bounds, aspect, offset);
            orbit.zoom_at(&bounds, aspect, 2.5, offset);
            let after = under(&orbit, &bounds, aspect, offset);
            for i in 0..3 {
                assert!(
                    (before[i] - after[i]).abs() < 1e-9,
                    "{before:?} vs {after:?}"
                );
            }
            assert!((orbit.zoom - 2.5).abs() < 1e-12);
        }
    }

    #[test]
    fn a_wheel_zoom_at_the_centre_does_not_pan() {
        let bounds = Bounds {
            min: [-1.0; 3],
            max: [1.0; 3],
        };
        let mut orbit = Orbit::named(1, true);
        orbit.zoom_at(&bounds, 1.0, 3.0, [0.0, 0.0]);
        assert_eq!(orbit.pan, [0.0, 0.0]);
    }
}
