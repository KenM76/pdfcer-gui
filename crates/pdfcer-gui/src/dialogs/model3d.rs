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
use pdfcer_gui_base::icons::Icon;

use crate::app::actions::Action;
use crate::app::actions::attachments::AttachmentAction;
use crate::app::actions::models::Assembled;
use crate::text::panels::models as t;
use pdfcer_core::threed::{ThreeDArtwork, ThreeDLighting, ThreeDSavedView};

mod behind;
mod bindings;
mod card;
mod controls;
mod drone;
mod effects;
mod eye;
mod fileview;
mod fit;
mod gimbal;
mod gpu;
mod gpu_bounce;
mod gpu_lens;
mod gpu_paint;
mod gpu_uniforms;
mod immersive;
mod lensmenu;
mod lights;
mod looks;
mod measure;
mod measure_paint;
mod measure_tab;
mod memory;
mod nav;
mod navkeys;
mod navmouse;
mod navorbit;
mod navwalk;
mod options;
mod parts;
mod partstree;
mod phone;
mod picture;
mod rows;
mod saved;
pub(crate) mod settingspage;
mod shot;
mod shotlook;
mod sphere;
mod stick;
mod sundisc;
mod sunlight;
mod window;

use pdfcer_3d_nav::{Lens, Orbit, PITCH_LIMIT, Upright, Walk};

/// The picture's published region, for `ui-verify`.
pub const REGION_IMAGE: &str = "model3d.image"; // ui-text-exempt: trace region name, never displayed
/// One named view's button; the suffix is its index in `t::view_names`.
pub const REGION_VIEW_PREFIX: &str = "model3d.view."; // ui-text-exempt: trace region name, never displayed
/// The *File's view* button.
pub const REGION_FILE_VIEW: &str = "model3d.view.file"; // ui-text-exempt: trace region name, never displayed
/// The up-axis drop-down; its items add `.<index in Axis::ALL>`
/// (`pdfcer_3d_nav::Axis`).
pub const REGION_UP: &str = "model3d.up"; // ui-text-exempt: trace region name, never displayed
/// The Front-look drop-down; its items add `.<index in Axis::ALL>`.
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
    /// A file the operator picks: PNG, or JPEG for a 360° picture.
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
    /// The walker's camera, when walking.
    walk: Option<Walk>,
    /// Walking seen from behind, with the camera's arm when measured.
    behind: Option<Option<f64>>,
    size: [u32; 2],
    background: egui::Color32,
    /// The highlighted part's colour, the theme's selection fill.
    accent: egui::Color32,
    /// [`parts::Visibility::version`].
    parts: u32,
    look: effects::Look,
    lens: Lens,
    /// What the card may draw.
    card: card::CardParts,
}

/// The 3D viewer's state.
pub(crate) struct ModelView {
    title: String,
    /// The listing row it was opened on, which a picture for the page names.
    artwork: ThreeDArtwork,
    model: Assembled,
    bounds: Bounds,
    /// `pdfcer_3d_nav::reach` of the model: the fitted sphere's radius.
    reach: f64,
    orbit: Orbit,
    /// The axes the named views are measured in.
    upright: Upright,
    /// The lens every picture is drawn through.
    lens: Lens,
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
    /// The live picture on the graphics card, when it can draw it.
    gpu: gpu::Gpu,
    rendered: Option<Rendered>,
    failed: Option<String>,
    close_requested: bool,
    /// The minimised state last traced.
    minimized: Option<bool>,
    /// This frame's Escape was used up — it left full screen or gave back
    /// the mouse — so it must not close the window.
    escape_left_full_screen: bool,
    /// The host drew a real OS window, the only kind that can fill the screen.
    native: bool,
    /// A picture of this view, waiting for the action queue.
    picture: Option<(PictureFor, Drawn)>,
    /// The Picture tab's settings and the picture it is drawing.
    shot: shot::Shot,
    /// Why the mesh fit last picked was not applied.
    fit_refused: Option<String>,
    /// Views to write into the file and the one to open on, waiting for
    /// the action queue.
    views: Option<(Vec<ThreeDSavedView>, usize, Vec<String>)>,
    /// The measure tool.
    measure: measure::Measure,
    /// Which parts are shown and which is highlighted.
    parts: parts::Visibility,
    /// The lighting every saved view asks for and the live picture is lit
    /// by; `None` writes none and lights it as the view asks.
    view_lighting: Option<ThreeDLighting>,
    /// The lights turn with the view (`LightFrame::Camera`), else stay put.
    lights_turn: bool,
    /// Lights cast shadows.
    shadows: bool,
    /// Ambient occlusion darkens creases and corners.
    occlusion: bool,
    /// Which way the scheme's light directions point, for shadows.
    light_sense: pdfcer_3d::LightSense,
    /// Shadows are cast by a sun fixed above the model on the Up axis,
    /// else by the lighting.
    sun: bool,
    /// A brightness per lighting choice, and the aimed sun.
    light: sunlight::Light,
    /// The render mode, casters, lamp and highlight choices.
    extras: looks::Extras,
    /// Orbiting or walking, the switches, and the fingers on the picture.
    nav: nav::Nav,
    /// The eye height was set or reset, for the preferences file.
    /// *Save as default* was pressed this frame.
    save_defaults: bool,
    /// The kept settings as last written.
    remembered: memory::Settings,
}

/// The opening's two trace lines: what was drawn and what its parts are.
fn trace_opened(page_index: usize, model: &Assembled, saved_orbit: bool) {
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
            u8::from(saved_orbit),
            fit::trace_fields(model)
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
}

impl ModelView {
    /// Open on `model`, decoded from `artwork`, at the file's opening view
    /// when it carries a camera, else at the isometric view; `walk` opens it
    /// walking from that view's eye, with ghost on.
    #[must_use]
    pub(crate) fn open(
        artwork: ThreeDArtwork,
        model: Assembled,
        eye_mm: Option<u32>,
        walk: bool,
    ) -> Self {
        let page_index = artwork.page_index;
        let bounds = framed(&model.drawn);
        let reach = pdfcer_3d_nav::reach(&model.drawn.meshes, &bounds);
        let opening = model.opening.clone();
        let saved = opening.as_ref().and_then(fileview::saved_orbit);
        saved::trace_listed(&model.saved);
        let saved_list = model.saved.clone();
        trace_opened(page_index, &model, saved.is_some());
        let parts = parts::Visibility::stored(shown_tree(&model));
        let look = shotlook::LookSet::factory();
        Self {
            title: t::view_title(page_index),
            artwork,
            model,
            bounds,
            reach,
            orbit: saved.unwrap_or_else(|| fileview::named(0, true, Upright::default())),
            upright: Upright::default(),
            lens: Lens::default(),
            named: 0,
            aimed: opening.clone(),
            saved: saved_list,
            saved_chosen: None,
            opening,
            texture: None,
            gpu: gpu::Gpu::default(),
            rendered: None,
            failed: None,
            close_requested: false,
            minimized: None,
            escape_left_full_screen: false,
            native: false,
            picture: None,
            shot: shot::Shot::default(),
            fit_refused: None,
            views: None,
            measure: measure::Measure::default(),
            parts,
            view_lighting: look.view_lighting,
            lights_turn: look.lights_turn,
            shadows: look.shadows,
            occlusion: look.occlusion,
            light_sense: look.light_sense,
            sun: look.sun,
            light: look.light,
            extras: look.extras,
            nav: nav::Nav::with_eye(eye_mm, walk),
            save_defaults: false,
            remembered: memory::Settings::new(),
        }
    }

    /// Draw it, queueing a picture for the page when one was asked for.
    /// Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        self.nav.load(ctx);
        if std::mem::take(&mut self.nav.start_walking) {
            self.set_walking(true);
            self.switched(ctx);
        }
        let closed = if self.nav.immersive && ctx.embed_viewports() {
            self.show_immersive(ctx);
            false
        } else {
            let (frame, ()) = crate::dialogs::host::Host::new(
                VIEWPORT_KEY,
                &self.title,
                self.nav
                    .window_open
                    .map_or(egui::vec2(720.0, 600.0), egui::Vec2::from),
                egui::vec2(360.0, 320.0),
            )
            .maximizable()
            .minimizable()
            .show(ctx, |ui| self.body(ui));
            self.native = frame.class == egui::ViewportClass::Immediate;
            frame.closed
        };
        match self.finished_shot(ctx) {
            Some(Ok(drawn)) => self.picture = Some((PictureFor::File, drawn)),
            Some(Err(said)) => self.failed = Some(t::poster_not_drawn(&said)),
            None => {}
        }
        if let Some((purpose, drawn)) = self.picture.take() {
            self.queue(purpose, drawn, actions);
        }
        if std::mem::take(&mut self.save_defaults) {
            self.save_defaults(actions);
        }
        if let Some((views, default, notes)) = self.views.take() {
            actions.push(Action::Attachment(AttachmentAction::SetModelViews {
                artwork: self.artwork.clone(),
                views,
                default,
                notes,
            }));
        }
        if let Some((view_index, measurements, notes)) = self.measure.writing.take() {
            actions.push(Action::Attachment(AttachmentAction::SetViewMeasurements {
                artwork: self.artwork.clone(),
                view_index,
                measurements,
                notes,
            }));
        }
        let closed = closed && !std::mem::take(&mut self.escape_left_full_screen);
        let button = std::mem::take(&mut self.close_requested);
        self.keep_settings(ctx, closed || button, actions);
        if closed || button {
            let how = if button { "button" } else { "window" };
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| format!("model-view-closed how={how}"));
        }
        !closed && !button
    }

    fn body(&mut self, ui: &mut Ui) {
        self.mouse_escape(ui);
        self.window_keys(ui);
        if self.filling_screen(ui.ctx()) {
            self.immersive_body(ui, ui.max_rect());
            return;
        }
        if egui_shell::ribbon::phone::is_phone(ui.ctx()) {
            self.phone_body(ui);
            return;
        }
        ui.small(self.nav_hint(ui.ctx(), t::view_hint()));
        self.file_view_note(ui);
        let started = std::time::Instant::now();
        egui::Panel::left(egui::Id::new("model3d-parts")) // ui-text-exempt: an egui id, never displayed
            .resizable(true)
            .default_size(180.0)
            .show(ui, |ui| self.parts_list(ui));
        let parts = started.elapsed();

        // The notes and buttons take their own height first and the picture
        // exactly what is left: a guessed footer the content overran made the
        // window grow every frame, redrawing the model at each new size.
        egui::Panel::bottom(egui::Id::new("model3d-footer")) // ui-text-exempt: an egui id, never displayed
            .resizable(false)
            .show(ui, |ui| {
                self.notes(ui);
                ui.horizontal_wrapped(|ui| self.buttons(ui));
            });
        let footer = started.elapsed() - parts;
        let area = ui.available_size().max(egui::vec2(64.0, 64.0));
        self.picture(ui, area);
        let picture = started.elapsed() - parts - footer;
        crate::diag::trace(|| {
            let ms = |d: std::time::Duration| d.as_secs_f64() * 1000.0;
            format!(
                // ui-text-exempt: diagnostic trace, never displayed
                "model-view-frame parts={:.1} footer={:.1} picture={:.1}",
                ms(parts),
                ms(footer),
                ms(picture)
            )
        });
    }

    /// The parts list, with show and hide.
    fn parts_list(&mut self, ui: &mut Ui) {
        let steerable = !self.model.drawn.tree.is_empty();
        parts::show(
            ui,
            shown_tree(&self.model),
            self.model.tree_error.as_deref(),
            &mut self.parts,
            steerable,
        );
    }

    /// The model drawn into `area`, steered by the pointer or fingers on it.
    fn picture(&mut self, ui: &mut Ui, area: egui::Vec2) {
        let (rect, response) = ui.allocate_exact_size(area, Sense::click_and_drag());
        crate::diag::ui_rect_visible(REGION_IMAGE, rect, ui.clip_rect());
        if self.nav.walk.is_some() {
            self.walk_input(ui, &response);
        } else {
            self.orbit_input(ui, &response);
        }
        self.measure_input(ui.ctx(), &response, rect);
        // Rebuilt below and by the full-screen row, for the next frame's
        // fingers.
        self.nav.hud.clear();

        let background = ui.visuals().extreme_bg_color;
        let gpu = self.gpu.live();
        if !gpu && self.texture.is_none() && self.failed.is_none() {
            // The GPU stopped before the CPU drew anything.
            self.rendered = None;
        }
        // On the CPU, half the pixels while the camera moves, so it keeps
        // up; the full picture is drawn the frame it stops.
        let card = gpu && self.gpu.declined().is_none();
        let halve = self.nav.moving && !card;
        let scale = ui.ctx().pixels_per_point() * if halve { 0.5 } else { 1.0 };
        let size = picture::render_size([rect.width(), rect.height()], scale, MAX_SIDE);
        let ppp = ui.ctx().pixels_per_point();
        self.shot.base = [rect.width(), rect.height()].map(|p| (p * ppp).round().max(1.0) as u32);
        self.shot.screen = background.to_array();
        let wanted = Rendered {
            orbit: self.orbit,
            walk: self.nav.walk,
            behind: self
                .nav
                .prefs
                .behind
                .then(|| self.nav.arm.map(|(_, arm)| arm)),
            size,
            background,
            accent: egui_shell::theme::Theme::canvas_selection_ink(ui.ctx()),
            parts: self.parts.version,
            look: self.look(),
            lens: self.lens,
            card: self.gpu.card,
        };
        if self.gpu.bounce_arrived(ui.ctx()) {
            self.rendered = None;
        }
        if self.rendered != Some(wanted) {
            self.draw_picture(ui.ctx(), wanted, gpu);
        }
        match (&self.texture, &self.failed) {
            (_, Some(said)) => {
                ui.put(rect, egui::Label::new(said.as_str()));
            }
            _ if gpu && self.gpu.declined().is_none() => self.gpu.paint(ui.painter(), rect),
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
        self.paint_measures(ui, rect);
        self.paint_overlay(ui, rect);
        self.hud_row(ui, rect);
        self.measure_card(ui.ctx(), rect);
        self.bar_button(ui, rect);
        self.options_button(ui, rect);
    }

    /// What the picture leaves out or inferred, said under it.
    fn notes(&mut self, ui: &mut Ui) {
        self.measure_readout(ui);
        self.nav_note(ui);
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
        self.looks_notes(ui);
    }

    /// Draw `wanted` on the graphics card when `gpu` is live and it takes
    /// the frame, else on the processor.
    fn draw_picture(&mut self, ctx: &egui::Context, wanted: Rendered, gpu: bool) {
        if gpu {
            self.rendered = Some(wanted);
            match self.gpu_frame(wanted) {
                Ok(true) => {
                    self.failed = None;
                    return;
                }
                Ok(false) => self.failed = None,
                Err(e) => {
                    self.failed = Some(t::view_render_failed(&e.to_string()));
                    return;
                }
            }
        }
        self.render(ctx, wanted);
    }

    /// *Close*, and what the view can be written to.
    fn buttons(&mut self, ui: &mut Ui) {
        let close = phone::glyph_button(ui, Icon::Close, t::view_close(), t::view_close());
        crate::diag::ui_rect_visible(REGION_CLOSE, close.rect, ui.clip_rect());
        if close.clicked() {
            self.close_requested = true;
        }
        if crate::panels::attachments::models::has_own_poster(&self.artwork) {
            self.picture_control(ui, PictureFor::Page);
            self.views_control(ui);
        }
        self.picture_control(ui, PictureFor::File);
    }

    /// *Use this view on the page* or *Save picture…*: draw this view and
    /// queue it for `purpose`.
    fn picture_control(&mut self, ui: &mut Ui, purpose: PictureFor) {
        let (icon, label, tip, region) = match purpose {
            PictureFor::Page => (
                Icon::ReplaceImage,
                t::view_use_on_page(),
                t::view_use_on_page_tooltip(),
                REGION_USE_ON_PAGE,
            ),
            PictureFor::File => (
                Icon::ExportImage,
                t::view_save_picture(),
                t::view_save_picture_tooltip(),
                REGION_SAVE_PICTURE,
            ),
        };
        if matches!(purpose, PictureFor::File) && self.shot_drawing() {
            ui.horizontal(|ui| {
                ui.add(egui::Spinner::new().color(ui.visuals().text_color()));
                ui.label(crate::text::model3dshot::drawing());
            });
            return;
        }
        let button = phone::glyph_button(ui, icon, label, tip);
        crate::diag::ui_rect_visible(region, button.rect, ui.clip_rect());
        if button.clicked() {
            self.queue_picture(purpose);
        }
    }

    /// Draw this view, without the controls, and queue it for `purpose`.
    fn queue_picture(&mut self, purpose: PictureFor) {
        let drawn = match purpose {
            PictureFor::Page => self
                .poster()
                .map(|drawn| self.picture = Some((purpose, drawn))),
            PictureFor::File => self.start_shot(),
        };
        if let Err(said) = drawn {
            self.failed = Some(t::poster_not_drawn(&said));
        }
    }

    /// *Save picture…* of this view, from a double tap on the gimbal.
    pub(super) fn save_scene(&mut self) {
        self.queue_picture(PictureFor::File);
    }

    /// Queue `drawn` for `purpose`: the page's picture takes the samples as
    /// they are, a file takes them encoded.
    fn queue(&mut self, purpose: PictureFor, drawn: Drawn, actions: &mut Vec<Action>) {
        let artwork = self.artwork.clone();
        let action = match purpose {
            PictureFor::Page => AttachmentAction::SetModelPoster {
                artwork,
                width: drawn.width,
                height: drawn.height,
                rgba: drawn.rgba,
            },
            PictureFor::File => match self.shot.encoded(&drawn) {
                Ok((bytes, sphere)) => AttachmentAction::SaveModelPicture {
                    artwork,
                    bytes,
                    sphere,
                },
                Err(said) => {
                    self.failed = Some(t::poster_not_drawn(&said));
                    return;
                }
            },
        };
        actions.push(Action::Attachment(action));
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

/// The box the viewer frames: the drawn meshes only, since a fully
/// transparent mesh is not drawn; a model with none frames the origin.
fn framed(model: &pdfcer_3d::AssembledModel) -> Bounds {
    Bounds::of_coloured(&model.meshes, &model.colours).unwrap_or(Bounds {
        min: [0.0; 3],
        max: [0.0; 3],
    })
}
