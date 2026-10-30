//! # `text::panels::savedviews` — the 3D viewer's *Saved views* menu
//!
//! Every string `pdfcer_gui::dialogs::model3d::saved` shows.

/// The menu of every view saved in the file.
#[must_use]
pub fn view_saved(count: usize) -> String {
    format!("Saved views ({count})")
}

/// The menu's tip.
#[must_use]
pub fn view_saved_tooltip() -> &'static str {
    "Look from one of the views saved in this file, as its author saved it."
}

/// One saved view: its name, or its place in the list when it has none;
/// the file's default is marked.
#[must_use]
pub fn view_saved_item(name: &str, index: usize, default: bool) -> String {
    let name = if name.trim().is_empty() {
        format!("View {}", index + 1)
    } else {
        name.to_owned()
    };
    if default {
        format!("{name} (opens on)")
    } else {
        name
    }
}

/// Why a saved view cannot be chosen.
#[must_use]
pub fn view_saved_no_camera() -> &'static str {
    "This view saves no camera position to look from."
}
