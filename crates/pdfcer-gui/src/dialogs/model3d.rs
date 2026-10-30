//! # `dialogs::model3d` — **the 3D model viewer**
//!
//! Opened by *View…* on a PRC row of the Attachments panel's 3D models
//! section. The engine's software renderer draws the placed model from a
//! camera this window moves: drag orbits, right-drag pans, scroll zooms about
//! the point under the pointer, and five named views jump to a side, measured
//! about the axis the operator says is up and the Front look he picks. The
//! window maximises from its title bar and fills the screen from its button or
//! F11; Escape leaves full screen before it closes the window. A still image is
//! rendered only when the camera or the picture's size changes.
//!
//! Design: `docs/modules/pdfcer-gui/dialogs/model3d.md`.

use egui::{Sense, TextureHandle, Ui};
use pdfcer_3d::Bounds;

use crate::app::actions::Action;
use crate::app::actions::attachments::AttachmentAction;
use crate::app::actions::models::Assembled;
use crate::text::panels::models as t;
use pdfcer_core::threed::{ThreeDArtwork, ThreeDLighting, ThreeDSavedView};

mod axes;
mod fit;
mod orbit;
mod parts;
mod picture;
mod saved;

use axes::{Axis, Upright};
use orbit::{Orbit, PITCH_LIMIT};

/// The picture's published region, for `ui-verify`.
pub const REGION_IMAGE: &str = "model3d.image"; // ui-text-exempt: trace region name, never displayed
/// One named view's button; the suffix is its index in `t::view_names`.
pub const REGION_VIEW_PREFIX: &str = "model3d.view."; // ui-text-exempt: trace region name, never displayed
/// The *File's view* button.
pub const REGION_FILE_VIEW: &str = "model3d.view.file"; // ui-text-exempt: trace region name, never displayed
/// The up-axis drop-down; its items add `.<index in axes::ALL>`.
pub const REGION_UP: &str = "model3d.up"; // ui-text-exempt: trace region name, never displayed
/// The Front-look drop-down; its items add `.<index in axes::ALL>`.
pub const REGION_FRONT: &str = "model3d.front"; // ui-text-exempt: trace region name, never displayed
/// The Fit button.
pub const REGION_FIT: &str = "model3d.fit"; // ui-text-exempt: trace region name, never displayed
/// The Close button.
pub const REGION_CLOSE: &str = "model3d.close"; // ui-text-exempt: trace region name, never displayed
/// The *Use this view on the page* button.
pub const REGION_USE_ON_PAGE: &str = "model3d.use_on_page"; // ui-text-exempt: trace region name, never displayed
/// The *Save picture…* button.
pub const REGION_SAVE_PICTURE: &str = "model3d.save_picture"; // ui-text-exempt: trace region name, never displayed
/// The saved views' lighting drop-down; its CAD item adds `.cad`.
pub const REGION_VIEWS_LIGHTING: &str = "model3d.views_lighting"; // ui-text-exempt: trace region name, never displayed
/// *Save views in the file*'s published region.
pub const REGION_SAVE_VIEWS: &str = "model3d.save_views"; // ui-text-exempt: trace region name, never displayed

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
    /// The highlighted part's colour, the theme's selection fill.
    accent: egui::Color32,
    /// [`parts::Visibility::version`].
    parts: u32,
}

/// The 3D viewer's state.
pub(crate) struct ModelView {
    title: String,
    /// The listing row it was opened on, which a picture for the page names.
    artwork: ThreeDArtwork,
    model: Assembled,
    bounds: Bounds,
    orbit: Orbit,
    /// The axes the named views are measured in.
    upright: Upright,
    /// The named view last chosen, re-applied when the axes change.
    named: usize,
    /// The file's opening view (`default_3d_view`), the camera it opens on
    /// when that view carries one.
    opening: Option<ThreeDSavedView>,
    /// The saved view whose framing the camera follows while
    /// `orbit.framed`: the opening view, or the one last chosen from
    /// *Saved views*.
    aimed: Option<ThreeDSavedView>,
    /// Every saved view in the file and its default (`saved_3d_views`).
    saved: pdfcer_core::threed::SavedViews,
    /// The saved view last chosen, marked in the menu.
    saved_chosen: Option<usize>,
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
    /// Why the mesh fit last picked was not applied.
    fit_refused: Option<String>,
    /// Views to write into the file and the one to open on, waiting for
    /// the action queue.
    views: Option<(Vec<ThreeDSavedView>, usize)>,
    /// Which parts are shown and which is highlighted.
    parts: parts::Visibility,
    /// The lighting every saved view asks for; `None` writes none.
    view_lighting: Option<ThreeDLighting>,
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
        saved::trace_listed(&model.saved);
        let saved_list = model.saved.clone();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-view-opened page={page_index} parts={} uncoloured={} triangles={} skipped={} best-fit={} overridden={} textured={} texture-notes={} placed={} file-view={} {}",
                model.drawn.meshes.len(),
                model.uncoloured(),
                model.triangles,
                model.skipped,
                model.best_fit,
                model.overridden,
                model.drawn.textured,
                model.drawn.texture_notes.len(),
                model.placed,
                u8::from(saved.is_some()),
                fit::trace_fields(&model)
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
        let parts = parts::Visibility::stored(shown_tree(&model));
        Self {
            title: t::view_title(page_index),
            artwork,
            model,
            bounds,
            orbit: saved.unwrap_or_else(|| Orbit::named(0, true, Upright::default())),
            upright: Upright::default(),
            named: 0,
            aimed: opening.clone(),
            saved: saved_list,
            saved_chosen: None,
            opening,
            texture: None,
            rendered: None,
            failed: None,
            close_requested: false,
            minimized: None,
            escape_left_full_screen: false,
            native: false,
            picture: None,
            fit_refused: None,
            views: None,
            parts,
            view_lighting: None,
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
        if let Some((views, default)) = self.views.take() {
            actions.push(Action::Attachment(AttachmentAction::SetModelViews {
                artwork: self.artwork.clone(),
                views,
                default,
            }));
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
            self.aimed = self.opening.clone();
            self.saved_chosen = None;
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
        if let Some(aim) = self.aimed.as_ref().and_then(|v| v.aim(aspect)) {
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

    /// *Up* and *Front looks along*. A change re-applies the named view last
    /// chosen in the new axes.
    fn axes_control(&mut self, ui: &mut Ui) {
        let was = self.upright;
        let up = axis_choice(
            ui,
            (t::view_up(), t::view_up_tooltip()),
            REGION_UP,
            was.up,
            axes::ALL.into_iter(),
        );
        let front = axis_choice(
            ui,
            (t::view_front(), t::view_front_tooltip()),
            REGION_FRONT,
            was.front,
            was.fronts(),
        );
        let chosen = match (up, front) {
            (Some(up), _) => was.with_up(up),
            (None, Some(front)) => was.with_front(front),
            (None, None) => return,
        };
        if chosen == was {
            return;
        }
        self.upright = chosen;
        self.orbit = Orbit::named(self.named, self.orbit.perspective, chosen);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-view-axes up={} front={} view={}",
                chosen.up.token(),
                chosen.front.token(),
                self.named
            )
        });
    }

    fn body(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            self.file_view_control(ui);
            self.saved_views_control(ui);
            for (i, name) in t::view_names().into_iter().enumerate() {
                let button = ui.button(name);
                crate::diag::ui_rect_visible(
                    // ui-text-exempt: trace region name, never displayed
                    &format!("{REGION_VIEW_PREFIX}{i}"),
                    button.rect,
                    ui.clip_rect(),
                );
                if button.clicked() {
                    self.named = i;
                    self.orbit = Orbit::named(i, self.orbit.perspective, self.upright);
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
            self.axes_control(ui);
            self.mesh_fit_control(ui);
            self.full_screen_control(ui);
        });
        ui.small(t::view_hint());
        self.file_view_note(ui);
        egui::Panel::left(egui::Id::new("model3d-parts")) // ui-text-exempt: an egui id, never displayed
            .resizable(true)
            .default_size(180.0)
            .show(ui, |ui| {
                let steerable = !self.model.drawn.tree.is_empty();
                parts::show(
                    ui,
                    shown_tree(&self.model),
                    self.model.tree_error.as_deref(),
                    &mut self.parts,
                    steerable,
                );
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
            accent: egui_shell::theme::Theme::canvas_selection_ink(ui.ctx()),
            parts: self.parts.version,
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
        fit::disclose(ui, &self.model, self.fit_refused.as_deref());
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
                self.views_control(ui);
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

    /// *Save views in the file*: the named views about the chosen axes, and
    /// this view when it is none of them, written into the model's file so a
    /// PDF reader lists them and opens the model on this one.
    fn views_control(&mut self, ui: &mut Ui) {
        let button = ui
            .button(t::view_save_views())
            .on_hover_text(t::view_save_views_tooltip());
        crate::diag::ui_rect_visible(REGION_SAVE_VIEWS, button.rect, ui.clip_rect());
        self.view_lighting_control(ui);
        if !button.clicked() {
            return;
        }
        let views = self.file_views();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            match &views {
                Some((list, default)) => {
                    format!(
                        "model-view-views count={} default={default} lighting={:?}",
                        list.len(),
                        self.view_lighting
                    )
                }
                None => "model-view-views count=0 default=none".to_owned(),
            }
        });
        match views {
            Some(views) => self.views = Some(views),
            None => self.failed = Some(t::views_not_built().to_owned()),
        }
    }

    /// The lighting the saved views ask a reader for, beside their button.
    fn view_lighting_control(&mut self, ui: &mut Ui) {
        use crate::text::model3d as m;
        let shown = self
            .view_lighting
            .map_or_else(m::lighting_unset, m::lighting);
        let combo = egui::ComboBox::from_id_salt(REGION_VIEWS_LIGHTING)
            .selected_text(shown)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.view_lighting, None, m::lighting_unset());
                for scheme in m::lighting_schemes() {
                    let item = ui.selectable_value(
                        &mut self.view_lighting,
                        Some(scheme),
                        m::lighting(scheme),
                    );
                    if scheme == ThreeDLighting::Cad {
                        // ui-text-exempt: trace region name, never displayed
                        let region = format!("{REGION_VIEWS_LIGHTING}.cad");
                        crate::diag::ui_rect_visible(&region, item.rect, ui.clip_rect());
                    }
                }
            });
        crate::diag::ui_rect_visible(REGION_VIEWS_LIGHTING, combo.response.rect, ui.clip_rect());
        combo.response.on_hover_text(m::views_lighting_tooltip());
    }

    /// Each named view fitted to the model as the page frames it, then this
    /// view when it is not the named one last chosen; with the index of
    /// this view. `None` when a view cannot be formed.
    fn file_views(&self) -> Option<(Vec<ThreeDSavedView>, usize)> {
        let [w, h] = self.rendered.map_or([4, 3], |r| r.size);
        let aspect = self
            .model
            .aspect
            .unwrap_or_else(|| f64::from(w.max(1)) / f64::from(h.max(1)));
        let perspective = self.orbit.perspective;
        let mut views = Vec::new();
        for (i, name) in t::view_names().into_iter().enumerate() {
            let orbit = Orbit::named(i, perspective, self.upright);
            let camera = orbit.camera(&self.bounds, aspect, None).ok()?;
            views.push(self.lit(ThreeDSavedView::from_camera(name, &camera, aspect)?));
        }
        if self.orbit == Orbit::named(self.named, perspective, self.upright) {
            return Some((views, self.named));
        }
        let camera = self
            .orbit
            .camera(&self.bounds, aspect, self.aimed.as_ref())
            .ok()?;
        views.push(self.lit(ThreeDSavedView::from_camera(
            t::view_custom(),
            &camera,
            aspect,
        )?));
        let custom = views.len() - 1;
        Some((views, custom))
    }

    /// `view` with the chosen lighting.
    fn lit(&self, view: ThreeDSavedView) -> ThreeDSavedView {
        match self.view_lighting {
            Some(scheme) => view.with_lighting(scheme),
            None => view,
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
            PictureFor::File => match picture::png(&drawn) {
                Ok(png) => AttachmentAction::SaveModelPicture { artwork, png },
                Err(said) => {
                    self.failed = Some(t::poster_not_drawn(&said));
                    return;
                }
            },
        };
        actions.push(Action::Attachment(action));
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
                    .zoom_at(&self.bounds, aspect, factor, offset, self.aimed.as_ref());
            }
        }
    }
}

/// The tree the part list shows: the assembly's, whose placement ranges name
/// the meshes, or the file's when the parts were not placed.
fn shown_tree(model: &Assembled) -> &[pdfcer_3d::ModelNode] {
    if model.drawn.tree.is_empty() {
        &model.tree
    } else {
        &model.drawn.tree
    }
}

/// A label and an axis drop-down, the button published as `region` and each
/// open item as `region.<index>`. Returns the axis picked this frame.
fn axis_choice(
    ui: &mut Ui,
    (label, tip): (&str, &str),
    region: &str,
    current: Axis,
    options: impl Iterator<Item = Axis>,
) -> Option<Axis> {
    let names = t::axis_names();
    let mut picked = None;
    ui.label(label).on_hover_text(tip);
    let combo = egui::ComboBox::from_id_salt(region)
        .selected_text(names[current.index()])
        .show_ui(ui, |ui| {
            for axis in options {
                let item = ui.selectable_label(axis == current, names[axis.index()]);
                crate::diag::ui_rect_visible(
                    // ui-text-exempt: trace region name, never displayed
                    &format!("{region}.{}", axis.index()),
                    item.rect,
                    ui.clip_rect(),
                );
                if item.clicked() {
                    picked = Some(axis);
                }
            }
        });
    crate::diag::ui_rect_visible(region, combo.response.rect, ui.clip_rect());
    combo.response.on_hover_text(tip);
    picked
}
