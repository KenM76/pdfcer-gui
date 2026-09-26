//! # `dialogs::settings::images` — two silences about resampling
//!
//! Both settings here are graded **tier (d)** by `pdfcer-core` — reasoned
//! inference, a guess — and both now say so. One of them did not in the old
//! shell, which is the specific instance of the window failing its own stated
//! contract that this port fixes.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/settings/images.md`.

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

/// How a large image is reduced to fit.
pub fn minify(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::minify_title(),
        t::minify_silence(),
        t::minify_radius(),
    );
    widgets::option(
        ui,
        &mut draft.working.image_minify,
        MinifyFilter::PointSample,
        t::minify_point_label(),
        Some(t::minify_point_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.image_minify,
        MinifyFilter::Smooth,
        t::minify_smooth_label(),
        Some(t::minify_smooth_note()),
    );
}
