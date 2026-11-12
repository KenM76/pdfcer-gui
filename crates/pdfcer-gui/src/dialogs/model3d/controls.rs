//! What is drawn over the picture, and the switches for it: the arrow in
//! the top-left corner that slides the options glyph and the button row out
//! of sight and back (never by itself; shown by default), and the
//! navigation controls — the gimbal and the drone sticks with their buttons
//! — switched from the row's sticks glyph or the options menu. Design:
//! `DESIGNS.md` § O413.
//!
//! A tap on the picture is held for the double-tap window, so a double tap
//! does its own job: centre on, or go to, the surface tapped.

use egui::{Pos2, Rect, Ui, UiBuilder, Vec2};
use pdfcer_gui_base::icons::Icon;

use super::ModelView;
use crate::text::panels::modelnav as t;

/// The options menu's *Show navigation controls* switch.
pub const REGION_CONTROLS: &str = "model3d.controls"; // ui-text-exempt: trace region name, never displayed
/// The row's camera glyph.
pub const REGION_SHOT: &str = "model3d.shot"; // ui-text-exempt: trace region name, never displayed
/// The row's sticks glyph.
pub const REGION_STICKS: &str = "model3d.sticks"; // ui-text-exempt: trace region name, never displayed
/// The corner arrow that hides and shows the options glyph and the row.
pub const REGION_BAR: &str = "model3d.bar"; // ui-text-exempt: trace region name, never displayed

/// A double tap's most time between taps, seconds, and distance, points.
pub(super) const DOUBLE_TAP: (f64, f32) = (0.4, 32.0);
/// The glyphs' gap from the picture's corner and from each other, points.
pub(super) const INSET: f32 = 8.0;

impl ModelView {
    /// A tap on the picture away from every control, at `now` and `at`.
    /// Whether it is the second of a double tap; a first one is held for
    /// the next.
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

    /// The arrow in `rect`'s top-left corner: it hides the options glyph
    /// and the button row, or brings them back.
    pub(super) fn bar_button(&mut self, ui: &mut Ui, rect: Rect) {
        let side = self.glyph_side(ui.ctx());
        let at = Rect::from_min_size(rect.min + Vec2::splat(INSET), Vec2::splat(side));
        let shown = self.nav.prefs.bar;
        let (icon, tip) = if shown {
            (Icon::ChevronLeft, t::bar_hide())
        } else {
            (Icon::ChevronRight, t::bar_show())
        };
        let mut child = ui.new_child(UiBuilder::new().max_rect(at));
        let arrow = super::immersive::glyph(&mut child, icon, tip, false, side);
        crate::diag::ui_rect_visible(REGION_BAR, arrow.rect, ui.clip_rect());
        self.nav.hud.push(at.expand(4.0));
        if arrow.clicked() {
            self.nav.prefs.bar = !shown;
            if shown {
                self.nav.options_open = false;
            }
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| format!("model-nav-bar shown={}", !shown));
            self.switched(ui.ctx());
        }
    }

    /// The row's camera and sticks glyphs, `side` points square.
    pub(super) fn bar_extras(&mut self, ui: &mut Ui, side: f32) {
        let shot = super::immersive::glyph(ui, Icon::Snapshot, t::shot_tooltip(), false, side);
        crate::diag::ui_rect_visible(REGION_SHOT, shot.rect, ui.clip_rect());
        if shot.clicked() {
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| "model-view-shot".to_owned());
            self.save_scene();
        }
        let shown = self.nav.prefs.controls;
        let sticks = super::immersive::glyph(ui, Icon::Sticks, t::controls_glyph(), shown, side);
        crate::diag::ui_rect_visible(REGION_STICKS, sticks.rect, ui.clip_rect());
        if sticks.clicked() {
            self.set_controls(ui.ctx(), !shown);
        }
    }
}
