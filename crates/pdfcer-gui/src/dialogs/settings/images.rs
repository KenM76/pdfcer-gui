//! # `dialogs::settings::images` — two silences about resampling
//!
//! Both settings here are graded **tier (d)** by `pdfcer-core` — reasoned
//! inference, a guess — and both now say so. One of them did not in the old
//! shell, which is the specific instance of the window failing its own stated
//! contract that this port fixes.

use egui::Ui;
use pdfcer_core::settings::{MaskResample, MinifyFilter};

use super::{Draft, widgets};
use crate::text::settings as t;

/// How a transparency mask of a different size to its image is filled in.
///
/// # The silence is unusually well established
///
/// ISO 32000-1 fixes the **geometry** and says nothing about the **filter**:
/// Table 145 has both images mapped to the unit square *"regardless of whether
/// the samples coincide individually"*, and §8.9.6.3 says the same for explicit
/// masks. The negatives were searched over the whole 756-page source —
/// `resample*` zero hits, `nearest neigh*` zero hits, `bilinear` three hits and
/// none image-related — and §8.9.5.3's note grants a reader *"any specific
/// implementation of interpolation that it wishes"*.
///
/// So the silence is real and sourced. The **answer** is still a guess, and the
/// note says so: `Nearest` is chosen because it is the only filter that cannot
/// invent an alpha value absent from the mask, which is good reasoning and is
/// nevertheless reasoning.
///
/// # One addition to the middle option's note
///
/// `BoxAverage` is not merely "smoother". It is the *right* answer when the
/// mask is at **higher** resolution than the base image, where reading one
/// sample per texel discards fifteen sixteenths of what the producer supplied.
/// That case decides the setting for anyone with a high-resolution mask and the
/// source's note did not mention it.
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
///
/// # ★ The guess disclosure the old window omitted
///
/// `pdfcer-core` grades this default tier (d) as explicitly as it grades the
/// mask filter above, and the old note read as a confident recommendation with
/// no admission. Obligation 1 was unmet for this setting for the whole of its
/// shipped life — which is worth stating plainly, because the window's header
/// asserted the obligation was met for all of them.
///
/// # The observation that would change the default, named
///
/// The note does more than confess. `pdfcer-render`'s own source asserts *"most
/// production viewers smooth on minification regardless of `/Interpolate`"* —
/// and that assertion is **unverified**, which is precisely the shape of claim
/// the standing rule about claim-bearing copy targets. Moving a default onto it
/// would be churn dressed as research.
///
/// So the note says what has not been checked. A viewer-behaviour observation
/// filed to the empirical PDF notes would raise this from tier (d) to tier (c)
/// and, if confirmed, flip the default — and an operator or a future session
/// reading the window can now see that this is a piece of work somebody can do,
/// rather than a settled answer.
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
