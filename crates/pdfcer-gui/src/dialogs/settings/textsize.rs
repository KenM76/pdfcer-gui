//! The Settings window's own text size: a slider at the far right of the
//! search row and Ctrl+wheel over the window both move
//! `Prefs::settings_text_scale`, which scales this window's text on top of
//! the program's `ui_scale` and nowhere else. See `prefs::settingstext`.

use egui::{Slider, Ui};
use pdfcer_gui_base::prefs::settingstext::{MAX, MIN, STEP, normalise};

use crate::text::settings as t;

/// The slider, published so a driven check can press it.
pub const REGION_TEXT_SIZE: &str = "settings.text_size"; // ui-text-exempt: trace region name, never displayed
/// The slider's label, published so a check can place it in the row.
pub const REGION_TEXT_SIZE_LABEL: &str = "settings.text_size.label"; // ui-text-exempt: trace region name, never displayed

/// The slider's rail, in points before scaling.
const WIDTH: f32 = 90.0;

/// Scale this window's text and the widget sizes that hold it by `scale`.
/// The window's `Ui` starts each frame from the program's style, so this
/// never compounds.
pub fn apply(ui: &mut Ui, scale: f32) {
    let style = ui.style_mut();
    for font in style.text_styles.values_mut() {
        font.size *= scale;
    }
    let spacing = &mut style.spacing;
    spacing.interact_size *= scale;
    spacing.icon_width *= scale;
    spacing.icon_width_inner *= scale;
    spacing.icon_spacing *= scale;
}

/// egui's zoom for one wheel notch, as `ln`: 40 points of line scroll at
/// `scroll_zoom_speed` 1/200. egui spreads a notch over several frames.
const NOTCH_LN: f32 = 0.2;

/// One [`STEP`] per Ctrl+wheel notch over the window: away from the
/// operator for larger, as every zoom reads it. The smoothed per-frame zoom
/// is summed and rounded to whole notches, the remainder carried.
pub fn wheel(ui: &Ui, scale: &mut f32) {
    // `max_rect`, not `ui_contains_pointer`: this runs before anything is
    // laid out, when the Ui's min rect is still empty.
    if !ui.rect_contains_pointer(ui.max_rect()) {
        return;
    }
    let zoom = ui.input(egui::InputState::zoom_delta);
    if zoom == 1.0 {
        return;
    }
    let id = egui::Id::new("settings-text-wheel");
    let mut carried: f32 = ui.data(|d| d.get_temp(id)).unwrap_or(0.0) + zoom.ln();
    while carried >= NOTCH_LN / 2.0 {
        *scale = normalise(*scale + STEP);
        carried -= NOTCH_LN;
    }
    while carried <= -NOTCH_LN / 2.0 {
        *scale = normalise(*scale - STEP);
        carried += NOTCH_LN;
    }
    ui.data_mut(|d| d.insert_temp(id, carried));
}

/// The label and slider, right to left from the row's end.
pub fn slider(ui: &mut Ui, scale: &mut f32) {
    ui.spacing_mut().slider_width = WIDTH;
    let response = ui
        .add(
            Slider::new(scale, MIN..=MAX)
                .step_by(f64::from(STEP))
                .show_value(false),
        )
        .on_hover_text(t::text_size_tooltip());
    crate::diag::ui_rect(REGION_TEXT_SIZE, response.rect);
    if response.changed() {
        *scale = normalise(*scale);
    }
    let label = ui
        .label(t::text_size_label())
        .on_hover_text(t::text_size_tooltip());
    crate::diag::ui_rect(REGION_TEXT_SIZE_LABEL, label.rect);
}
