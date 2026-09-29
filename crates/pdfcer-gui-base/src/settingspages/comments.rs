//! # `dialogs::settings::comments` — who signs the comments you write
//!
//! One control, and a module for it because the *placement* of that control is
//! the part with an argument.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/settingspages/comments.md`.

use egui::Ui;

use super::widgets;
use crate::text::settings as t;

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
