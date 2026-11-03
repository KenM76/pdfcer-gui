//! The viewer's navigation state and switches: orbiting or walking, ghost
//! mode, level walking, the gimbal and the walking speed, and the model's
//! surfaces a walker and a double-tap meet. `navorbit` and `navwalk` read
//! the input for each mode; `immersive` lays the viewer over the screen.
//!
//! The switches live for the session in egui's memory, so the next model
//! opened walks the same way.

use egui::{Pos2, Rect, Response, Ui};
use pdfcer_3d::{AssembledModel, Camera, Hit, PickIndex, Ray, RenderError};
use pdfcer_gui_base::icons::{self, Icon};

use super::drone::Drone;
use super::gimbal::Gimbal;
use super::orbit::radius;
use super::parts::Visibility;
use super::stick::Sticks;
use super::walk::{Rules, Walk};
use super::{ModelView, PITCH_LIMIT};
use crate::text::panels::modelnav as t;

/// The Walk / Orbit switch.
pub const REGION_WALK: &str = "model3d.walk"; // ui-text-exempt: trace region name, never displayed
/// Ghost mode.
pub const REGION_GHOST: &str = "model3d.ghost"; // ui-text-exempt: trace region name, never displayed
/// Stair climbing, where it is a switch in the row rather than under the
/// camera stick.
pub const REGION_STAIRS: &str = "model3d.stairs"; // ui-text-exempt: trace region name, never displayed
/// The drone sticks' buttons, `n` in [`super::drone::BUTTONS`] order.
pub const REGION_DRONE_BUTTON: &str = "model3d.drone.button."; // ui-text-exempt: trace region name, never displayed
/// The drone sticks, `n` in [`super::drone::Drone::centres`] order.
pub const REGION_DRONE_STICK: &str = "model3d.drone.stick."; // ui-text-exempt: trace region name, never displayed
/// Level walking.
pub const REGION_LEVEL: &str = "model3d.level"; // ui-text-exempt: trace region name, never displayed
/// The gimbal switch.
pub const REGION_GIMBAL: &str = "model3d.gimbal"; // ui-text-exempt: trace region name, never displayed
/// Gimbal tip `n` (+x, +y, +z, -x, -y, -z), then `6` for its centre.
pub const REGION_GIMBAL_TIP: &str = "model3d.gimbal.tip."; // ui-text-exempt: trace region name, never displayed
/// The full-screen switch where the viewer is drawn inside the application.
pub const REGION_IMMERSIVE: &str = "model3d.immersive"; // ui-text-exempt: trace region name, never displayed

/// The egui memory key of the switches.
const PREFS_KEY: &str = "model3d-nav-prefs"; // ui-text-exempt: an egui id, never displayed
/// A walking step's collision time above which the slowness is said, in
/// milliseconds.
pub(super) const SLOW_MS: f64 = 40.0;
/// A double tap on the gimbal's centre: its most time between taps, seconds.
const CENTRE_DOUBLE: f64 = 0.4;

/// The navigation switches.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Prefs {
    pub ghost: bool,
    pub level: bool,
    pub gimbal: bool,
    /// The navigation controls are drawn over the picture.
    pub controls: bool,
    pub speed: f64,
    /// Climb obstacles under a third of the eye height.
    pub stairs: bool,
    /// The eye height the operator set, millimetres; `None` for a share of
    /// the model's size.
    pub eye_mm: Option<f64>,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            ghost: false,
            level: true,
            gimbal: true,
            controls: true,
            speed: 1.0,
            stairs: false,
            eye_mm: None,
        }
    }
}

impl Prefs {
    pub fn rules(self) -> Rules {
        Rules {
            solid: !self.ghost,
            level: self.level,
            speed: self.speed,
            stairs: self.stairs,
        }
    }
}

/// The model's surface a walker meets: the engine's pick index, built once
/// per model, and the shown-mesh mask, read again when visibility changes.
#[derive(Default)]
struct Solids {
    /// The model the index was built from: its mesh list's address and length.
    model: Option<(usize, usize)>,
    index: Option<PickIndex>,
    version: Option<u32>,
    shown: Option<Vec<bool>>,
}

impl Solids {
    fn hit(&mut self, model: &AssembledModel, parts: &Visibility, ray: &Ray) -> Option<Hit> {
        let key = (model.meshes.as_ptr() as usize, model.meshes.len());
        if self.model != Some(key) {
            self.index = Some(PickIndex::new(&model.meshes));
            (self.model, self.version) = (Some(key), None);
        }
        if self.version != Some(parts.version) {
            self.shown = parts.shown_meshes(model);
            self.version = Some(parts.version);
        }
        let shown = self.shown.as_deref();
        self.index.as_ref()?.pick_where(ray, f64::INFINITY, |mesh| {
            shown.is_none_or(|mask| mask.get(mesh).copied().unwrap_or(true))
        })
    }
}

/// Navigation state kept between frames.
#[derive(Default)]
pub(super) struct Nav {
    /// Walking, or `None` orbiting.
    pub walk: Option<Walk>,
    pub prefs: Prefs,
    pub sticks: Sticks,
    /// Filling the screen where the viewer is drawn inside the application.
    pub immersive: bool,
    /// A tap on the picture not yet known single or double: when and where.
    pub pending_tap: Option<(f64, Pos2)>,
    /// The last tap on the gimbal's centre, for a double tap: when, and the
    /// view before it, which the double tap puts back to picture.
    centre_tap: Option<(std::time::Instant, super::orbit::Orbit, Option<Walk>)>,
    solids: Solids,
    /// The camera moved this frame, so the picture renders at the moving
    /// size.
    pub moving: bool,
    /// Areas over the picture whose own buttons take a finger.
    pub hud: Vec<Rect>,
    /// The options menu is open: the viewer holds it, not egui's one
    /// remembered popup, so a drop-down inside it keeps it open.
    pub options_open: bool,
    /// The drone sticks and their tilts, to draw, while they are offered.
    pub drone: Option<(Drone, [egui::Vec2; 3])>,
    /// This walk's frames and its slowest collision, milliseconds.
    pub walked: (u32, f64),
    /// The slowest collision of the last walk, milliseconds, said when slow.
    pub slow: Option<f64>,
    /// The session's switches have been taken up.
    loaded: bool,
    /// Start walking on the next frame: the viewer opens walking.
    pub start_walking: bool,
}

impl Nav {
    /// Default switches with the remembered eye height, millimetres. `walk`
    /// starts walking on the first frame, with ghost on.
    pub fn with_eye(eye_mm: Option<u32>, walk: bool) -> Self {
        let mut nav = Self::default();
        nav.prefs.eye_mm = eye_mm.map(f64::from);
        nav.prefs.ghost = walk;
        nav.start_walking = walk;
        nav
    }

    /// Take up the session's switches, once.
    pub fn load(&mut self, ctx: &egui::Context) {
        if std::mem::replace(&mut self.loaded, true) {
            return;
        }
        if let Some(prefs) = ctx.data(|d| d.get_temp(egui::Id::new(PREFS_KEY))) {
            self.prefs = prefs;
        }
    }

    fn store(&self, ctx: &egui::Context) {
        ctx.data_mut(|d| d.insert_temp(egui::Id::new(PREFS_KEY), self.prefs));
    }
}

/// A switch showing `icon`, with `label` beside it except on a phone.
pub(super) fn toggle(ui: &mut Ui, icon: Icon, label: &str, tip: &str, on: bool) -> Response {
    let image = if on {
        icons::selected_image(ui, icon)
    } else {
        icons::image(ui, icon)
    };
    let button = if egui_shell::ribbon::phone::is_phone(ui.ctx()) {
        egui::Button::image(image)
    } else {
        egui::Button::image_and_text(image, label)
    };
    ui.add(button.selected(on))
        .on_hover_text(label)
        .on_hover_text(tip)
}

impl ModelView {
    /// The camera the picture is drawn from, in an image of `aspect`.
    pub(super) fn view_camera(&self, aspect: f64) -> Result<Camera, RenderError> {
        match &self.nav.walk {
            Some(walk) => Ok(walk.camera(aspect)),
            None => self
                .orbit
                .camera(&self.bounds, self.reach, aspect, self.aimed.as_ref()),
        }
    }

    /// The picture's aspect, width / height, as last drawn.
    pub(super) fn aspect(&self) -> f64 {
        let [w, h] = self.rendered.map_or([4, 3], |r| r.size);
        f64::from(w.max(1)) / f64::from(h.max(1))
    }

    /// Walk, Ghost, Level, the gimbal, the speed, and the full-screen switch
    /// where the viewer has no OS window to fill the screen with.
    pub(super) fn nav_controls(&mut self, ui: &mut Ui) {
        let walking = self.nav.walk.is_some();
        let (icon, label, tip) = if walking {
            (Icon::Orbit, t::orbit(), t::orbit_tooltip())
        } else {
            (Icon::Walk, t::walk(), t::walk_tooltip())
        };
        let walk = toggle(ui, icon, label, tip, false);
        crate::diag::ui_rect_visible(REGION_WALK, walk.rect, ui.clip_rect());
        if walk.clicked() {
            self.set_walking(!walking);
        }
        if walking {
            self.walk_switches(ui, false);
        }
        let gimbal = toggle(
            ui,
            Icon::Gimbal,
            t::gimbal(),
            t::gimbal_tooltip(),
            self.nav.prefs.gimbal,
        );
        crate::diag::ui_rect_visible(REGION_GIMBAL, gimbal.rect, ui.clip_rect());
        if gimbal.clicked() {
            self.nav.prefs.gimbal = !self.nav.prefs.gimbal;
            self.switched(ui.ctx());
        }
        if !self.native && !self.nav.immersive {
            let full = toggle(
                ui,
                Icon::Fullscreen,
                t::immersive(),
                t::immersive_tooltip(),
                false,
            );
            crate::diag::ui_rect_visible(REGION_IMMERSIVE, full.rect, ui.clip_rect());
            if full.clicked() {
                self.set_immersive(true);
            }
        }
    }

    /// Ghost, Level and (in a window) the speed; `hud` draws them as the
    /// full-screen row's thumb-sized glyphs.
    pub(super) fn walk_switches(&mut self, ui: &mut Ui, hud: bool) {
        let prefs = self.nav.prefs;
        let switch = |ui: &mut Ui, icon, label, tip, on| {
            if hud {
                super::immersive::glyph(ui, icon, label, on).on_hover_text(tip)
            } else {
                toggle(ui, icon, label, tip, on)
            }
        };
        let ghost = switch(ui, Icon::Ghost, t::ghost(), t::ghost_tooltip(), prefs.ghost);
        crate::diag::ui_rect_visible(REGION_GHOST, ghost.rect, ui.clip_rect());
        if ghost.clicked() {
            self.nav.prefs.ghost = !prefs.ghost;
            self.switched(ui.ctx());
        }
        let level = switch(
            ui,
            Icon::LevelView,
            t::level(),
            t::level_tooltip(),
            prefs.level,
        );
        crate::diag::ui_rect_visible(REGION_LEVEL, level.rect, ui.clip_rect());
        if level.clicked() {
            self.nav.prefs.level = !prefs.level;
            if let (true, Some(walk)) = (self.nav.prefs.level, &mut self.nav.walk) {
                walk.level();
            }
            self.switched(ui.ctx());
        }
        if self.nav.drone.is_none() {
            let stairs = switch(
                ui,
                Icon::Stairs,
                t::stairs(),
                t::stairs_tooltip(),
                prefs.stairs,
            );
            crate::diag::ui_rect_visible(REGION_STAIRS, stairs.rect, ui.clip_rect());
            if stairs.clicked() {
                self.toggle_stairs(ui.ctx());
            }
        }
        if !hud {
            let speed = ui
                .add(
                    egui::Slider::new(&mut self.nav.prefs.speed, 0.25..=4.0)
                        .logarithmic(true)
                        .custom_formatter(|v, _| t::speed_value(v))
                        .text(t::speed()),
                )
                .on_hover_text(t::speed_tooltip());
            if speed.drag_stopped() || speed.lost_focus() {
                self.switched(ui.ctx());
            }
        }
    }

    /// Switch stair climbing.
    pub(super) fn toggle_stairs(&mut self, ctx: &egui::Context) {
        self.nav.prefs.stairs = !self.nav.prefs.stairs;
        self.switched(ctx);
    }

    /// Keep the switches and trace them.
    pub(super) fn switched(&mut self, ctx: &egui::Context) {
        self.nav.store(ctx);
        let p = self.nav.prefs;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-nav-prefs ghost={} level={} gimbal={} controls={} speed={:.2} stairs={} eye_mm={}",
                p.ghost,
                p.level,
                p.gimbal,
                p.controls,
                p.speed,
                p.stairs,
                p.eye_mm
                    .map_or_else(|| "auto".to_owned(), |mm| format!("{mm:.1}"))
            )
        });
    }

    /// Start walking from the orbit's eye, or go back to orbiting the way
    /// the walker looked. The walker's up is the operator's *Up*, never the
    /// file camera's: a file view's up is often tilted, and a walk on it
    /// would climb or sink while level.
    pub(super) fn set_walking(&mut self, on: bool) {
        if on {
            let Ok(camera) =
                self.orbit
                    .camera(&self.bounds, self.reach, self.aspect(), self.aimed.as_ref())
            else {
                return;
            };
            self.nav.walk = Some(Walk::from_camera(
                &camera,
                super::orbit::Axes::upright(self.upright),
                radius(&self.bounds),
            ));
            let height = self.eye_height();
            if let Some(walk) = &mut self.nav.walk {
                walk.height = height;
            }
        } else if let Some(walk) = self.nav.walk.take() {
            self.orbit.look_along(walk.direction());
            self.orbit.pitch = self.orbit.pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
        }
        self.nav.drone = None;
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| format!("model-walk on={on}"));
    }

    /// Back to orbiting, for a view chosen by name; the chooser then sets
    /// the orbit.
    pub(super) fn orbiting(&mut self) {
        if self.nav.walk.take().is_some() {
            self.nav.drone = None;
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| "model-walk on=false".to_owned());
        }
    }

    /// The hint under the controls: `orbiting` while orbiting, how to walk
    /// while walking.
    pub(super) fn nav_hint<'a>(
        &self,
        ctx: &egui::Context,
        orbiting: &'a str,
    ) -> std::borrow::Cow<'a, str> {
        let touch = self.nav.sticks.touch() || egui_shell::ribbon::phone::is_phone(ctx);
        match (self.nav.walk.is_some(), touch) {
            (true, true) => t::walk_hint_touch().into(),
            (true, false) => t::walk_hint().into(),
            (false, true) => orbiting.into(),
            (false, false) => t::orbit_hint_with_keys(orbiting).into(),
        }
    }

    /// Fill the screen, or stop.
    pub(super) fn set_immersive(&mut self, on: bool) {
        self.nav.immersive = on;
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| format!("model-view-immersive on={on}"));
    }

    /// The model's shown surface along `ray`, nearest first.
    pub(super) fn hit(&mut self, ray: &Ray) -> Option<Hit> {
        self.nav.solids.hit(&self.model.drawn, &self.parts, ray)
    }

    /// The model's shown surface under `at` on the picture `rect`.
    pub(super) fn hit_at(&mut self, rect: Rect, at: Pos2) -> Option<Hit> {
        let (w, h) = (rect.width().max(1.0), rect.height().max(1.0));
        let camera = self.view_camera(f64::from(w / h)).ok()?;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let ray = camera
            .ray_through(
                f64::from(at.x - rect.left()),
                f64::from(at.y - rect.top()),
                w.round() as u32,
                h.round() as u32,
            )
            .ok()?;
        self.hit(&ray)
    }

    /// A tap on gimbal zone `zone` ([`Gimbal::zones`] order). A second tap
    /// on its centre within [`CENTRE_DOUBLE`] puts back the view the first
    /// one fitted or levelled and saves a picture of it, drawn without the
    /// controls (O357).
    pub(super) fn gimbal_tap(&mut self, gimbal: &Gimbal, zone: usize) {
        let axis = gimbal.tips.get(zone).map(|tip| tip.axis);
        if axis.is_none() {
            let now = std::time::Instant::now();
            match self.nav.centre_tap.take() {
                Some((then, orbit, walk))
                    if now.duration_since(then).as_secs_f64() <= CENTRE_DOUBLE =>
                {
                    (self.orbit, self.nav.walk) = (orbit, walk);
                    // ui-text-exempt: diagnostic trace, never displayed
                    crate::diag::trace(|| "model-gimbal-snapshot".to_owned());
                    self.save_scene();
                    return;
                }
                _ => self.nav.centre_tap = Some((now, self.orbit, self.nav.walk)),
            }
        }
        match (&mut self.nav.walk, axis) {
            (Some(walk), Some(axis)) => walk.face(axis),
            (Some(walk), None) => walk.level(),
            (None, Some(axis)) => self.orbit.look_along(axis.map(|c| -c)),
            (None, None) => self.home(),
        }
        let walking = self.nav.walk.is_some();
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| format!("model-gimbal zone={zone} walking={walking}"));
    }

    /// The gimbal over `rect` for the current camera, when it and the
    /// controls are shown.
    pub(super) fn gimbal(&self, rect: Rect) -> Option<Gimbal> {
        if !(self.nav.prefs.gimbal && self.nav.prefs.controls) {
            return None;
        }
        let camera = self
            .view_camera(f64::from(rect.width().max(1.0) / rect.height().max(1.0)))
            .ok()?;
        Gimbal::layout(rect, &camera)
    }

    /// The gimbal and the drone sticks over the picture `rect`, and their
    /// hot areas for a driven check.
    pub(super) fn paint_overlay(&self, ui: &Ui, rect: Rect) {
        let painter = ui.painter().with_clip_rect(rect);
        let visuals = ui.visuals();
        if let Some(gimbal) = self.gimbal(rect) {
            gimbal.paint(&painter, visuals);
            for (i, zone) in gimbal.zones().into_iter().enumerate() {
                crate::diag::ui_rect_visible(
                    // ui-text-exempt: trace region name, never displayed
                    &format!("{REGION_GIMBAL_TIP}{i}"),
                    zone,
                    rect,
                );
            }
        }
        if let Some((drone, tilts)) = &self.nav.drone {
            drone.paint(ui, &painter, *tilts, self.nav.prefs.stairs);
            for (i, button) in drone.buttons.into_iter().enumerate() {
                // ui-text-exempt: trace region name, never displayed
                crate::diag::ui_rect_visible(&format!("{REGION_DRONE_BUTTON}{i}"), button, rect);
            }
            for (i, (centre, r)) in drone.centres.into_iter().zip(drone.radii).enumerate() {
                let base = Rect::from_center_size(centre, egui::Vec2::splat(r * 2.0));
                // ui-text-exempt: trace region name, never displayed
                crate::diag::ui_rect_visible(&format!("{REGION_DRONE_STICK}{i}"), base, rect);
            }
        }
    }

    /// Said under the picture when walls and floors make walking slow.
    pub(super) fn nav_note(&self, ui: &mut Ui) {
        if let (Some(ms), Some(_), false) = (self.nav.slow, &self.nav.walk, self.nav.prefs.ghost) {
            ui.small(t::slow_collision(ms, self.model.triangles));
        }
    }
}
