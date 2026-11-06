//! Showing and hiding the navigation controls over the picture — the
//! gimbal, the drone sticks and their buttons, and the full-screen row —
//! by a switch in the options menu or a single tap on the picture. While
//! they are hidden a see-through camera button at the picture's bottom
//! centre saves the view, drawn by the engine without it or the menu.
//!
//! A tap toggles only once the double-tap window has passed without a
//! second tap, so a double tap does its own job (centre on, or go to, the
//! surface tapped) and leaves the controls as they were.

use egui::{Pos2, Rect, Ui, Vec2};
use pdfcer_gui_base::icons::{self, Icon};

use super::ModelView;
use crate::text::panels::modelnav as t;

/// The options menu's *Show navigation controls* switch.
pub const REGION_CONTROLS: &str = "model3d.controls"; // ui-text-exempt: trace region name, never displayed
/// The camera button drawn while the controls are hidden.
pub const REGION_SHOT: &str = "model3d.shot"; // ui-text-exempt: trace region name, never displayed

/// A double tap's most time between taps, seconds, and distance, points.
pub(super) const DOUBLE_TAP: (f64, f32) = (0.4, 32.0);
/// The camera button's side and its gap from the picture's bottom, points.
const SHOT: (f32, f32) = (56.0, 16.0);
/// How much of the camera button's fill and glyph shows through.
const SHOT_ALPHA: (f32, f32) = (0.35, 0.75);

impl ModelView {
    /// A tap on the picture away from every control, at `now` and `at`.
    /// Whether it is the second of a double tap; a first one is held for
    /// [`ModelView::settle_tap`].
    /// A tap that closes the options menu is the menu's, not the picture's.
    pub(super) fn tap_picture(&mut self, now: f64, at: Pos2) -> bool {
        if self.nav.options_open {
            self.nav.pending_tap = None;
            return false;
        }
        if self.measure.on {
            self.nav.pending_tap = None;
            self.measure.take_tap(now, at);
            return false;
        }
        let double = self.nav.pending_tap.take().is_some_and(|(then, was)| {
            now - then <= DOUBLE_TAP.0 && (at - was).length() <= DOUBLE_TAP.1
        });
        if !double {
            self.nav.pending_tap = Some((now, at));
        }
        double
    }

    /// Toggle the controls once a held tap's window has passed alone.
    pub(super) fn settle_tap(&mut self, ctx: &egui::Context) {
        let Some((then, _)) = self.nav.pending_tap else {
            return;
        };
        let left = DOUBLE_TAP.0 - (ctx.input(|i| i.time) - then);
        if left > 0.0 {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(left));
            return;
        }
        self.nav.pending_tap = None;
        self.set_controls(ctx, !self.nav.prefs.controls);
    }

    /// Show or hide the controls, for the session.
    pub(super) fn set_controls(&mut self, ctx: &egui::Context, shown: bool) {
        self.nav.prefs.controls = shown;
        if !shown {
            self.nav.drone = None;
        }
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| format!("model-nav-controls shown={shown}"));
        self.switched(ctx);
    }

    /// *Show navigation controls*, in the options menu.
    pub(super) fn controls_switch(&mut self, ui: &mut Ui) {
        let mut shown = self.nav.prefs.controls;
        let switch = ui
            .checkbox(&mut shown, t::controls())
            .on_hover_text(t::controls_tooltip());
        crate::diag::ui_rect_visible(REGION_CONTROLS, switch.rect, ui.clip_rect());
        if switch.changed() {
            self.set_controls(ui.ctx(), shown);
        }
    }

    /// The camera button over the picture `rect`'s bottom centre while the
    /// controls are hidden; a press saves a picture of the view.
    pub(super) fn shot_button(&mut self, ui: &mut Ui, rect: Rect) {
        if self.nav.prefs.controls {
            return;
        }
        let at = Rect::from_center_size(
            Pos2::new(rect.center().x, rect.bottom() - SHOT.1 - SHOT.0 / 2.0),
            Vec2::splat(SHOT.0),
        );
        let response = ui
            .interact(at, ui.id().with(REGION_SHOT), egui::Sense::click())
            .on_hover_text(t::shot_tooltip());
        crate::diag::ui_rect_visible(REGION_SHOT, at, ui.clip_rect());
        self.nav.hud.push(at.expand(4.0));
        let painter = ui.painter().with_clip_rect(rect);
        let fill = ui.visuals().window_fill.gamma_multiply(SHOT_ALPHA.0);
        painter.circle_filled(at.center(), SHOT.0 / 2.0, fill);
        icons::image(ui, Icon::Snapshot)
            .tint(egui::Color32::WHITE.gamma_multiply(SHOT_ALPHA.1)) // NOT A THEME COLOUR: pass-through tint, only its alpha applies
            .paint_at(ui, at.shrink(SHOT.0 * 0.22));
        if response.clicked() {
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| "model-view-shot".to_owned());
            self.save_scene();
        }
    }
}
