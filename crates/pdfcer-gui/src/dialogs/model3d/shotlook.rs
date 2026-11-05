//! The saved picture's own look: lighting, shadows, occlusion, render mode
//! and every other choice the Look tab makes, set on the Picture tab apart
//! from the screen's. It starts as a copy of the screen's choices when its
//! switch is turned on; off, the picture looks as the screen does.
//!
//! The Look tab's own controls draw it: the set is swapped into the viewer's
//! fields for the controls and for the picture's options, and swapped back
//! within the same call, so nothing outside these two sees it.

use egui::Ui;
use pdfcer_core::threed::ThreeDLighting;

use super::ModelView;
use crate::text::model3dshot as s;

/// The switch giving the picture its own look.
pub const REGION_SHOT_OWN_LOOK: &str = "model3d.shot.own_look"; // ui-text-exempt: trace region name, never displayed

/// Every field [`ModelView::look`] reads that the Look tab sets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct LookSet {
    pub(super) view_lighting: Option<ThreeDLighting>,
    pub(super) lights_turn: bool,
    pub(super) shadows: bool,
    pub(super) occlusion: bool,
    pub(super) light_sense: pdfcer_3d::LightSense,
    pub(super) sun: bool,
    pub(super) light: super::sunlight::Light,
    pub(super) extras: super::looks::Extras,
}

#[cfg(test)]
impl LookSet {
    /// A set with each field at a fixed value, for tests.
    pub(super) fn default_for_tests() -> Self {
        Self {
            view_lighting: None,
            lights_turn: false,
            shadows: true,
            occlusion: false,
            light_sense: pdfcer_3d::LightSense::default(),
            sun: false,
            light: super::sunlight::Light::default(),
            extras: super::looks::Extras::default(),
        }
    }
}

impl ModelView {
    /// The screen's choices, as a set.
    pub(super) fn look_set(&self) -> LookSet {
        LookSet {
            view_lighting: self.view_lighting,
            lights_turn: self.lights_turn,
            shadows: self.shadows,
            occlusion: self.occlusion,
            light_sense: self.light_sense,
            sun: self.sun,
            light: self.light,
            extras: self.extras,
        }
    }

    /// Exchange the screen's choices with `set`'s.
    pub(super) fn swap_look(&mut self, set: &mut LookSet) {
        std::mem::swap(&mut self.view_lighting, &mut set.view_lighting);
        std::mem::swap(&mut self.lights_turn, &mut set.lights_turn);
        std::mem::swap(&mut self.shadows, &mut set.shadows);
        std::mem::swap(&mut self.occlusion, &mut set.occlusion);
        std::mem::swap(&mut self.light_sense, &mut set.light_sense);
        std::mem::swap(&mut self.sun, &mut set.sun);
        std::mem::swap(&mut self.light, &mut set.light);
        std::mem::swap(&mut self.extras, &mut set.extras);
    }

    /// `f` run with the picture's own look in place of the screen's, when it
    /// has one, else as it is.
    pub(super) fn in_shot_look<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let Some(mut set) = self.shot.own.take() else {
            return f(self);
        };
        self.swap_look(&mut set);
        let out = f(self);
        self.swap_look(&mut set);
        self.shot.own = Some(set);
        out
    }

    /// The own-look switch and, while it is on, the Look tab's controls
    /// setting the picture's look.
    pub(super) fn shot_look_control(&mut self, ui: &mut Ui) {
        let mut own = self.shot.own.is_some();
        let switch = ui
            .checkbox(&mut own, s::own_look())
            .on_hover_text(s::own_look_tooltip());
        crate::diag::ui_rect_visible(REGION_SHOT_OWN_LOOK, switch.rect, ui.clip_rect());
        if switch.changed() {
            self.shot.own = own.then(|| self.look_set());
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| format!("model-view-picture-look own={}", u8::from(own)));
        }
        if self.shot.own.is_none() {
            return;
        }
        ui.group(|ui| {
            self.in_shot_look(|view| {
                view.lighting_control(ui);
                view.effects_control(ui);
                view.looks_control(ui);
            });
        });
    }
}
