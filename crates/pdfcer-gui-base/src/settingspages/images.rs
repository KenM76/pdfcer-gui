//! # `dialogs::settings::images` — two silences about resampling
//!
//! Both settings here are graded **tier (d)** by `pdfcer-core` — reasoned
//! inference, a guess — and both now say so. One of them did not in the old
//! shell, which is the specific instance of the window failing its own stated
//! contract that this port fixes.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/settingspages/images.md`.

use egui::Ui;
use pdfcer_core::settings::{MaskResample, MinifyFilter};

use super::{Draft, widgets};
use crate::text::settings as t;

/// How a transparency mask of a different size to its image is filled in.
pub fn mask_resample(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(ui, t::mask_title(), t::mask_silence(), t::mask_radius());
    widgets::option(
        ui,
        &mut draft.working.mask_resample,
        MaskResample::Nearest,
        t::mask_nearest_label(),
        Some(t::mask_nearest_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.mask_resample,
        MaskResample::BoxAverage,
        t::mask_box_label(),
        Some(t::mask_box_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.mask_resample,
        MaskResample::Bilinear,
        t::mask_bilinear_label(),
        Some(t::mask_bilinear_note()),
    );
}

/// The minify options' regions, `<this>.<file word>`: `point_sample`,
/// `smooth`, `area_average`.
pub const MINIFY_REGION: &str = "settings.images.minify"; // ui-text-exempt: trace region name, never displayed

/// How a large image is reduced to fit: every `MinifyFilter` the engine has.
pub fn minify(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::minify_title(),
        t::minify_silence(),
        t::minify_radius(),
    );
    let options = [
        (
            MinifyFilter::PointSample,
            "point_sample", // ui-text-exempt: trace region name, never displayed
            t::minify_point_label(),
            t::minify_point_note(),
        ),
        (
            MinifyFilter::Smooth,
            "smooth", // ui-text-exempt: trace region name, never displayed
            t::minify_smooth_label(),
            t::minify_smooth_note(),
        ),
        (
            MinifyFilter::AreaAverage,
            "area_average", // ui-text-exempt: trace region name, never displayed
            t::minify_area_label(),
            t::minify_area_note(),
        ),
    ];
    for (value, word, label, note) in options {
        let radio = widgets::option(
            ui,
            &mut draft.working.image_minify,
            value,
            label,
            Some(note),
        );
        let region = format!("{MINIFY_REGION}.{word}");
        crate::diag::ui_rect_visible(&region, radio.rect, ui.clip_rect());
    }
}
