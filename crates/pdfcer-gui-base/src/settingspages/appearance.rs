//! # `dialogs::settings::appearance` — the theme picker
//!
//! One setting, and the only one in the window that is not about the PDF
//! standard at all.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/settingspages/appearance.md`.

use egui::Ui;
use egui_shell::theme::Preset;

use super::{Draft, widgets};
use crate::text::settings as t;

/// The theme radio group.
pub fn theme(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(ui, t::theme_title(), t::theme_silence(), t::theme_radius());

    let current = Preset::from_key(&draft.working.theme);
    for preset in Preset::ALL {
        let selected = current == Some(*preset);
        let response = ui.radio(selected, t::theme_preset_label(*preset));
        // Published so a harness can CLICK it. That a theme picker exists is
        // not the property worth proving; that choosing Dark makes the window
        // dark is, and that needs a rectangle to aim at. See `DEFECTS.md` D10.
        crate::diag::ui_rect(
            &format!("{}{}", super::REGION_THEME_PREFIX, preset.key()),
            response.rect,
        );
        if response.clicked() {
            draft.working.theme = preset.key().to_owned();
        }
        let note = t::theme_preset_note(*preset);
        if !note.is_empty() {
            ui.label(egui::RichText::new(note).small().weak());
        }
    }

    // The token names a theme this build does not have.
    //
    // Said out loud, with the name quoted, because otherwise the operator sees
    // none of the three selected and no explanation — which reads as a
    // rendering fault. And the likeliest cause is benign and worth knowing: a
    // settings file from a NEWER pdfcer, whose token is being **preserved**
    // rather than overwritten. Telling them it is kept is what stops them
    // "fixing" it by picking one of the three, which would discard it.
    if current.is_none() {
        widgets::disclosure(ui, &t::theme_unknown(&draft.working.theme));
    }
}

/// **How big pdfcer's own controls are drawn.**
pub fn ui_scale(ui: &mut Ui, prefs: &mut crate::prefs::Prefs) {
    use crate::prefs::{MAX_UI_SCALE, MIN_UI_SCALE, UI_SCALE_STEP};

    widgets::header(
        ui,
        t::ui_scale_title(),
        t::ui_scale_silence(),
        t::ui_scale_radius(),
    );
    ui.add(
        egui::Slider::new(&mut prefs.ui_scale, MIN_UI_SCALE..=MAX_UI_SCALE)
            .step_by(f64::from(UI_SCALE_STEP))
            // The slider edits the multiplier and displays the percentage.
            // `custom_formatter` rather than storing a percentage, because the
            // file, the frame hook and `egui`'s own `zoom_factor` all speak
            // multipliers — converting at the one place a human reads it keeps
            // a single unit everywhere else.
            .custom_formatter(|value, _| t::ui_scale_percent(value))
            .text(t::ui_scale_slider_label()),
    );
    ui.label(egui::RichText::new(t::ui_scale_note()).small().weak());
}

/// The coloured-icons switch, `OPERATOR_REQUESTS.md` O232. Edits the draft,
/// which the frame previews live, as [`ui_scale`] does.
pub fn colour_icons(ui: &mut Ui, prefs: &mut crate::prefs::Prefs) {
    widgets::header(
        ui,
        t::colour_icons_title(),
        t::colour_icons_silence(),
        t::colour_icons_radius(),
    );
    ui.checkbox(&mut prefs.colour_icons, t::colour_icons_label());
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::settings::Settings;

    /// Every preset the shell offers has a name and a key that round-trips.
    #[test]
    fn every_offered_preset_round_trips_through_its_token() {
        for preset in Preset::ALL {
            let key = preset.key();
            assert_eq!(
                Preset::from_key(key),
                Some(*preset),
                "the picker offers {key:?} and the shell would not recognise it back"
            );
            assert!(!t::theme_preset_label(*preset).is_empty());
        }
    }

    /// The shipped default token is one the shell knows.
    #[test]
    fn the_shipped_default_token_is_a_theme_this_build_has() {
        let token = Settings::default().theme;
        assert_eq!(
            Preset::from_key(&token),
            Some(Preset::default()),
            "core's default theme token {token:?} is not this shell's default preset"
        );
    }

    /// An unrecognised token is preserved, not corrected.
    #[test]
    fn an_unknown_token_is_not_silently_replaced() {
        let mut settings = Settings::default();
        settings.theme = "midnight".to_owned();
        let draft = Draft::new(&settings, &crate::prefs::Prefs::default());
        assert_eq!(draft.working.theme, "midnight");
        assert!(Preset::from_key(&draft.working.theme).is_none());
    }
}
