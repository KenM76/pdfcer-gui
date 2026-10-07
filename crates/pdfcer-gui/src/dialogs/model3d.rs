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

use egui::{Sense, TextureHandle, TextureOptions, Ui};
// The 3D renderer's options, not the page renderer's that settings own.
use pdfcer_3d::RenderOptions as ModelRenderOptions;
use pdfcer_3d::{Bounds, render_model};

use crate::app::actions::Action;
use crate::app::actions::attachments::AttachmentAction;
use crate::app::actions::models::Assembled;
use crate::text::panels::models as t;
use pdfcer_core::threed::{ThreeDArtwork, ThreeDSavedView};

mod orbit;
mod parts;

use orbit::{Orbit, PITCH_LIMIT};

/// The picture's published region, for `ui-verify`.
pub const REGION_IMAGE: &str = "model3d.image"; // ui-text-exempt: trace region name, never displayed
/// One named view's button; the suffix is its index in `t::view_names`.
pub const REGION_VIEW_PREFIX: &str = "model3d.view."; // ui-text-exempt: trace region name, never displayed
/// The *File's view* button.
pub const REGION_FILE_VIEW: &str = "model3d.view.file"; // ui-text-exempt: trace region name, never displayed
/// The Fit button.
pub const REGION_FIT: &str = "model3d.fit"; // ui-text-exempt: trace region name, never displayed
/// The Close button.
pub const REGION_CLOSE: &str = "model3d.close"; // ui-text-exempt: trace region name, never displayed
/// The *Use this view on the page* button.
pub const REGION_USE_ON_PAGE: &str = "model3d.use_on_page"; // ui-text-exempt: trace region name, never displayed
/// The *Save picture…* button.
pub const REGION_SAVE_PICTURE: &str = "model3d.save_picture"; // ui-text-exempt: trace region name, never displayed

/// A picture of the current view: unpremultiplied RGBA, row-major from the
/// top.
struct Drawn {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

/// What a picture drawn from the current view is for.
#[derive(Clone, Copy)]
enum PictureFor {
    /// The model's picture on the page.
    Page,
    /// A PNG file the operator picks.
    File,
}
/// The Minimize button.
pub const REGION_MINIMIZE: &str = "model3d.minimize"; // ui-text-exempt: trace region name, never displayed
/// The full-screen button.
pub const REGION_FULL_SCREEN: &str = "model3d.full_screen"; // ui-text-exempt: trace region name, never displayed

/// The viewer's OS window key.
const VIEWPORT_KEY: &str = "model-3d"; // ui-text-exempt: a viewport key, never displayed

/// Radians turned per point dragged.
const ORBIT_RATE: f64 = 0.01;
/// The largest picture rendered, in pixels a side; the CPU renderer's cost
/// grows with the area.
const MAX_SIDE: f32 = 1600.0;
/// The long side, in pixels, of a picture made for the page.
const POSTER_SIDE: f64 = 1200.0;
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
    /// The listing row it was opened on, which a picture for the page names.
    artwork: ThreeDArtwork,
    model: Assembled,
    bounds: Bounds,
    orbit: Orbit,
    /// The file's opening view (`default_3d_view`), the camera it opens on
    /// when that view carries one.
    opening: Option<ThreeDSavedView>,
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
    /// A picture of this view, waiting for the action queue.
    picture: Option<(PictureFor, Drawn)>,
}

impl ModelView {
    /// Open on `model`, decoded from `artwork`, at the file's opening view
    /// when it carries a camera, else at the isometric view.
    #[must_use]
    pub(crate) fn open(artwork: ThreeDArtwork, model: Assembled) -> Self {
        let page_index = artwork.page_index;
        let bounds = Bounds::of(&model.drawn.meshes).unwrap_or(Bounds {
            min: [0.0; 3],
            max: [0.0; 3],
        });
        let opening = model.opening.clone();
        let saved = opening.as_ref().and_then(Orbit::saved);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-view-opened page={page_index} parts={} uncoloured={} triangles={} skipped={} best-fit={} overridden={} textured={} texture-notes={} placed={} file-view={}",
                model.drawn.meshes.len(),
                model.uncoloured(),
                model.triangles,
                model.skipped,
                model.best_fit,
                model.overridden,
                model.drawn.textured,
                model.drawn.texture_notes.len(),
                model.placed,
                u8::from(saved.is_some())
            )
        });
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-view-parts {} error={}",
                parts::trace_fields(&model.tree),
                u8::from(model.tree_error.is_some())
            )
        });
        Self {
            title: t::view_title(page_index),
            artwork,
            model,
            bounds,
            orbit: saved.unwrap_or_else(|| Orbit::named(0, true)),
            opening,
            texture: None,
            rendered: None,
            failed: None,
            close_requested: false,
            minimized: None,
            escape_left_full_screen: false,
            native: false,
            picture: None,
        }
    }

    /// Draw it, queueing a picture for the page when one was asked for.
    /// Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
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
        if let Some((purpose, drawn)) = self.picture.take() {
            self.queue(purpose, drawn, actions);
        }
        let closed = frame.closed && !std::mem::take(&mut self.escape_left_full_screen);
        let button = std::mem::take(&mut self.close_requested);
        if closed || button {
            let how = if button { "button" } else { "window" };
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| format!("model-view-closed how={how}"));
        }
        !closed && !button
    }

    /// *File's view*: back to the camera the file opens on, offered only
    /// when its opening view carries one.
    fn file_view_control(&mut self, ui: &mut Ui) {
        let Some(saved) = self.opening.as_ref().and_then(Orbit::saved) else {
            return;
        };
        let button = ui
            .button(t::view_file())
            .on_hover_text(t::view_file_tooltip());
        crate::diag::ui_rect_visible(REGION_FILE_VIEW, button.rect, ui.clip_rect());
        if button.clicked() {
            self.orbit = saved;
        }
    }

    /// While the file's view fixes the framing, how its scale was read: the
    /// standard gives the orthographic scale no unit.
    fn file_view_note(&self, ui: &mut Ui) {
        if !self.orbit.framed || self.orbit.perspective {
            return;
        }
        let aspect = self.rendered.map_or(1.0, |r| {
            f64::from(r.size[0].max(1)) / f64::from(r.size[1].max(1))
        });
        if let Some(aim) = self.opening.as_ref().and_then(|v| v.aim(aspect)) {
            ui.small(t::view_file_note(&aim.source()));
        }
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
            self.file_view_control(ui);
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
                self.orbit.framed = false;
            }
            ui.checkbox(&mut self.orbit.perspective, t::view_perspective())
                .on_hover_text(t::view_perspective_tooltip());
            self.full_screen_control(ui);
        });
        ui.small(t::view_hint());
        self.file_view_note(ui);
        egui::Panel::left(egui::Id::new("model3d-parts")) // ui-text-exempt: an egui id, never displayed
            .resizable(true)
            .default_size(180.0)
            .show(ui, |ui| {
                parts::show(ui, &self.model.tree, self.model.tree_error.as_deref());
            });

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
            self.model.drawn.meshes.len(),
            self.model.triangles,
        ));
        ui.small(t::view_colour_note(
            self.model.uncoloured(),
            self.model.drawn.meshes.len(),
        ));
        if !self.model.placed {
            ui.small(t::mesh_placement_note());
        }
        if self.model.skipped > 0 {
            ui.small(t::mesh_skipped(self.model.skipped));
        }
        if self.model.best_fit > 0 {
            ui.small(t::mesh_best_fit(self.model.best_fit));
        }
        if self.model.overridden > 0 {
            ui.small(t::view_overridden(self.model.overridden));
        }
        if self.model.drawn.textured > 0 {
            ui.small(t::view_textured(self.model.drawn.textured));
        }
        for (reason, parts) in &self.model.drawn.texture_notes {
            ui.small(t::view_texture_note(reason, *parts));
        }
        ui.horizontal(|ui| {
            let close = ui.button(t::view_close());
            crate::diag::ui_rect_visible(REGION_CLOSE, close.rect, ui.clip_rect());
            if close.clicked() {
                self.close_requested = true;
            }
            if crate::panels::attachments::models::has_own_poster(&self.artwork) {
                self.picture_control(ui, PictureFor::Page);
            }
            self.picture_control(ui, PictureFor::File);
        });
    }

    /// *Use this view on the page* or *Save picture…*: draw this view and
    /// queue it for `purpose`.
    fn picture_control(&mut self, ui: &mut Ui, purpose: PictureFor) {
        let (label, tip, region) = match purpose {
            PictureFor::Page => (
                t::view_use_on_page(),
                t::view_use_on_page_tooltip(),
                REGION_USE_ON_PAGE,
            ),
            PictureFor::File => (
                t::view_save_picture(),
                t::view_save_picture_tooltip(),
                REGION_SAVE_PICTURE,
            ),
        };
        let button = ui.button(label).on_hover_text(tip);
        crate::diag::ui_rect_visible(region, button.rect, ui.clip_rect());
        if !button.clicked() {
            return;
        }
        match self.poster() {
            Ok(drawn) => self.picture = Some((purpose, drawn)),
            Err(said) => self.failed = Some(t::poster_not_drawn(&said)),
        }
    }

    /// Queue `drawn` for `purpose`: the page's picture takes the samples as
    /// they are, a file takes them encoded as PNG.
    fn queue(&mut self, purpose: PictureFor, drawn: Drawn, actions: &mut Vec<Action>) {
        let artwork = self.artwork.clone();
        let action = match purpose {
            PictureFor::Page => AttachmentAction::SetModelPoster {
                artwork,
                width: drawn.width,
                height: drawn.height,
                rgba: drawn.rgba,
            },
            PictureFor::File => match png(&drawn) {
                Ok(png) => AttachmentAction::SaveModelPicture { artwork, png },
                Err(said) => {
                    self.failed = Some(t::poster_not_drawn(&said));
                    return;
                }
            },
        };
        actions.push(Action::Attachment(action));
    }

    /// This view, at the picture's shape, on the white the engine's own
    /// poster uses.
    fn poster(&self) -> Result<Drawn, String> {
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
        let image = self
            .orbit
            .camera(&self.bounds, aspect, self.opening.as_ref())
            .and_then(|camera| render_model(&self.model.drawn, &camera, &options))
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
                self.orbit
                    .zoom_at(&self.bounds, aspect, factor, offset, self.opening.as_ref());
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
        let camera = self.orbit.camera(
            &self.bounds,
            f64::from(width) / f64::from(height),
            self.opening.as_ref(),
        );
        let aimed = camera.as_ref().map(aim_of).unwrap_or_default();
        let drawn = camera.and_then(|camera| render_model(&self.model.drawn, &camera, &options));
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
fn png(drawn: &Drawn) -> Result<Vec<u8>, String> {
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
