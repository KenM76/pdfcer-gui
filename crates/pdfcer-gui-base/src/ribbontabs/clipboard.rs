//! The **Clipboard** tab — *what am I taking out, or putting in?*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/ribbontabs/clipboard.md`.

use super::large;
use crate::text::ribbon;
use egui_shell::manifest::{Item, Tab};

/// The condition a verb that changes the document is shown under: Review and
/// Edit, where markup may be authored. Copy changes nothing, so it has none.
pub const CHANGES_DOCUMENT: &str = "mode.author_markup"; // ui-text-exempt: a condition name, never displayed

/// A large command shown only where the document may change.
fn changing(id: &str) -> Item {
    large(id).shown_when(CHANGES_DOCUMENT)
}

/// The Clipboard tab.
pub fn tab() -> Tab {
    Tab::new("clipboard", ribbon::tab_clipboard())
        .with_icon("paste")
        .with_question(ribbon::question_clipboard())
        .with_groups([
            super::group(
                "clipboard",
                ribbon::group_clipboard_clipboard(),
                [
                    changing("edit.cut"),
                    large("edit.copy"),
                    changing("edit.paste"),
                    changing("edit.paste_duplicate"),
                    changing("edit.duplicate"),
                ],
            ),
            super::group(
                "other_programs",
                ribbon::group_clipboard_other_programs(),
                [large("edit.copy_as_vector")],
            ),
        ])
}
