//! # `dialogs::settings::display` — how pdfcer draws, as distinct from what it draws
//!
//! The eighth group, and the only one whose settings are **not** about the PDF
//! standard. Every other group in this window exists because a clause declines
//! to have an opinion; these two exist because a machine has a speed.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/settings/display.md`.

use egui::Ui;

use super::widgets;
use crate::app::prefs::{
    MAX_SETTLE_MS, MIN_SETTLE_MS, OpeningFit, PageCache, PasteChords, Prefs, RenderQuality,
    WheelPaging,
};
use crate::text::settings as t;

/// How sharply a page is rasterised.
pub fn render_quality(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::quality_title(),
        t::quality_silence(),
        t::quality_radius(),
    );
    for option in RenderQuality::ALL {
        widgets::option(
            ui,
            &mut prefs.render_quality,
            *option,
            t::quality_label(*option),
            Some(t::quality_note(*option)),
        );
    }
}

/// How much memory the page cache may hold.
pub fn page_cache(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::page_cache_title(),
        t::page_cache_silence(),
        t::page_cache_radius(),
    );
    for option in PageCache::ALL {
        widgets::option(
            ui,
            &mut prefs.page_cache,
            *option,
            // Owned, unlike every other label in this window, because the
            // megabyte figure is COMPUTED from the budget rather than written
            // beside it. `widgets::option` takes `&str`, and a `String` that
            // lives to the end of the call is the smallest thing that works —
            // the alternative is four `const` labels carrying four hand-copied
            // numbers, which is the drift this derivation exists to prevent.
            &t::page_cache_label(*option),
            Some(t::page_cache_note(*option)),
        );
    }
}

/// How long a zoom must stop changing before the page is redrawn sharply.
pub fn zoom_settle(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::settle_title(),
        t::settle_silence(),
        t::settle_radius(),
    );
    ui.add(
        egui::Slider::new(&mut prefs.zoom_settle_ms, MIN_SETTLE_MS..=MAX_SETTLE_MS)
            .suffix(t::settle_suffix())
            .text(t::settle_slider_label()),
    );
    ui.label(egui::RichText::new(t::settle_note()).small().weak());
}

/// How the first page of a newly opened document is sized to the window.
pub fn opening_fit(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::opening_fit_title(),
        t::opening_fit_silence(),
        t::opening_fit_radius(),
    );
    for option in OpeningFit::ALL {
        widgets::option(
            ui,
            &mut prefs.opening_fit,
            *option,
            t::opening_fit_label(*option),
            Some(t::opening_fit_note(*option)),
        );
    }
}

/// What a plain mouse wheel does under a one-page-at-a-time display mode.
pub fn paste_chords(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::paste_chords_title(),
        t::paste_chords_silence(),
        t::paste_chords_radius(),
    );
    for option in PasteChords::ALL {
        widgets::option(
            ui,
            &mut prefs.paste_chords,
            *option,
            t::paste_chords_label(*option),
            Some(t::paste_chords_note(*option)),
        );
    }
}

/// drift from the other.
pub fn wheel_paging(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::wheel_paging_title(),
        t::wheel_paging_silence(),
        t::wheel_paging_radius(),
    );
    for option in WheelPaging::ALL {
        widgets::option(
            ui,
            &mut prefs.wheel_paging,
            *option,
            t::wheel_paging_label(*option),
            Some(t::wheel_paging_note(*option)),
        );
    }
}

/// Which of the three View ▸ Display overlays are already on when a document
/// opens.
pub fn field_shade(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::field_shade_title(),
        t::field_shade_silence(),
        t::field_shade_radius(),
    );
    widgets::toggle(
        ui,
        &mut prefs.shade_form_fields,
        t::field_shade_label(),
        Some(t::field_shade_note()),
    );
}

/// The OCR colour's Reset button, declared only while it is drawn.
pub const OCR_RESET_REGION: &str = "settings.display.ocr_colour.reset"; // ui-text-exempt: trace region name, never displayed

/// **What colour the recognised text is drawn in over a scan** —
/// `OPERATOR_REQUESTS.md` O229.
pub fn ocr_colour(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::ocr_colour_title(),
        t::ocr_colour_silence(),
        t::ocr_colour_radius(),
    );
    ui.horizontal(|ui| {
        // The swatch edits the draft's bytes in place. `egui`'s own picker,
        // and not `canvas::markup::swatch`'s preset grid: that one exists
        // because a markup colour is chosen against a convention other people
        // will read, and this one is chosen against whatever is on THIS scan,
        // where a named palette would be an obstacle.
        ui.color_edit_button_srgb(&mut prefs.ocr_layer_colour);
        ui.label(t::ocr_colour_label());
        if prefs.ocr_layer_colour != crate::canvas::ocrlayer::DEFAULT_COLOUR {
            let reset = ui.button(t::ocr_colour_reset());
            crate::diag::ui_rect_visible(OCR_RESET_REGION, reset.rect, ui.clip_rect());
            if reset.clicked() {
                prefs.ocr_layer_colour = crate::canvas::ocrlayer::DEFAULT_COLOUR;
            }
        }
    });
    ui.label(egui::RichText::new(t::ocr_colour_note()).small().weak());
}

/// **The two auto-hide settings** — 2026-09-05.
pub fn auto_hide(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::auto_hide_title(),
        t::auto_hide_silence(),
        t::auto_hide_radius(),
    );
    widgets::toggle(
        ui,
        &mut prefs.ribbon_auto_hide,
        t::auto_hide_ribbon_label(),
        Some(t::auto_hide_ribbon_note()),
    );
    widgets::toggle(
        ui,
        &mut prefs.rail_auto_hide,
        t::auto_hide_rail_label(),
        Some(t::auto_hide_rail_note()),
    );
}

pub fn page_chrome(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::chrome_title(),
        t::chrome_silence(),
        t::chrome_radius(),
    );
    widgets::toggle(
        ui,
        &mut prefs.chrome.rulers,
        t::chrome_rulers_label(),
        Some(t::chrome_rulers_note()),
    );
    widgets::toggle(
        ui,
        &mut prefs.chrome.grid,
        t::chrome_grid_label(),
        Some(t::chrome_grid_note()),
    );
    widgets::toggle(
        ui,
        &mut prefs.chrome.guides,
        t::chrome_guides_label(),
        Some(t::chrome_guides_note()),
    );
    widgets::disclosure(ui, t::chrome_guides_bound());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shipped quality is the identity multiplier.
    #[test]
    fn the_shipped_quality_changes_no_raster() {
        assert!((RenderQuality::default().multiplier() - 1.0).abs() < f32::EPSILON);
    }

    /// The three qualities are ordered less-to-more and are distinct.
    #[test]
    fn the_qualities_ascend() {
        let m: Vec<f32> = RenderQuality::ALL.iter().map(|q| q.multiplier()).collect();
        assert_eq!(m.len(), 3);
        assert!(m[0] < m[1] && m[1] < m[2], "{m:?}");
    }

    /// The shipped settle is reachable on its own slider.
    #[test]
    fn the_shipped_settle_is_reachable_on_the_slider() {
        let ms = Prefs::default().zoom_settle_ms;
        assert!(
            (MIN_SETTLE_MS..=MAX_SETTLE_MS).contains(&ms),
            "the shipped settle {ms} is outside the slider's range"
        );
    }
}
