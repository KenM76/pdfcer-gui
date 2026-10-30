//! # `dialogs::settings::comments` — who signs the comments you write
//!
//! One control, and a module for it because the *placement* of that control is
//! the part with an argument.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/settingspages/comments.md`.

use egui::Ui;

use super::widgets;
use crate::text::settings as t;

/// How a FreeText annotation's `/C` is read when the shell draws or stores
/// its appearance: `Settings::free_text_c`.
pub fn free_text_c(ui: &mut Ui, draft: &mut super::Draft) {
    use pdfcer_core::annot_author::FreeTextC;
    widgets::header(
        ui,
        t::free_text_c_title(),
        t::free_text_c_silence(),
        t::free_text_c_radius(),
    );
    let current = &mut draft.working.free_text_c;
    widgets::option(
        ui,
        current,
        FreeTextC::Fill,
        t::free_text_c_fill_label(),
        None,
    );
    widgets::option(
        ui,
        current,
        FreeTextC::Border,
        t::free_text_c_border_label(),
        None,
    );
}

/// The name written into `/T` on every comment this shell authors.
pub fn author_name(ui: &mut Ui, prefs: &mut crate::prefs::Prefs) {
    widgets::header(
        ui,
        t::author_name_title(),
        t::author_name_silence(),
        t::author_name_radius(),
    );
    widgets::text_value(
        ui,
        // ui-text-exempt: an egui control id, never displayed.
        "settings-author-name",
        &mut prefs.author_name,
        t::author_name_label(),
        Some(t::author_name_note()),
        Clone::clone,
        |typed| Some(typed.to_owned()),
    );
}
