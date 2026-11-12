//! # `text::settings::printpresets` — the words on the Print presets page
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/prefs/printpresets.md`.

/// The page's heading.
#[must_use]
pub const fn group() -> &'static str {
    "Print presets"
}

/// The folder list's title.
#[must_use]
pub const fn title() -> &'static str {
    "Folders to find print presets in"
}

/// What the list does.
#[must_use]
pub const fn silence() -> &'static str {
    "The Print window's Preset list shows every preset in these folders, in this order. \
     New presets are saved in the first one. When two folders hold a preset of the same name, \
     the one listed first is used."
}

/// Where the change applies.
#[must_use]
pub const fn radius() -> &'static str {
    "Applies the next time the Print window opens, and is kept in your preferences file."
}

/// Beside the default folder, which cannot be removed.
#[must_use]
pub const fn default_note() -> &'static str {
    "default: new presets are saved here"
}

/// Hover on the default folder.
#[must_use]
pub const fn default_hover() -> &'static str {
    "The print-presets folder in pdfcer's settings folder. It is always searched first and \
     cannot be removed."
}

/// In place of the default folder when the program has no settings folder.
#[must_use]
pub const fn no_default() -> &'static str {
    "pdfcer has no settings folder here, so presets cannot be saved; extra folders are still \
     searched."
}

/// The Open button beside the default folder.
#[must_use]
pub const fn open_folder() -> &'static str {
    "Open"
}

/// The Open button's hover.
#[must_use]
pub const fn open_hover() -> &'static str {
    "Show this folder in File Explorer, to copy presets in or out."
}

/// The Add button's hover.
#[must_use]
pub const fn add_hover() -> &'static str {
    "Every .pdfcerprint file in the folder is listed in the Print window's Preset list. A \
     shared network folder lets a whole office use the same presets."
}

/// The folder picker's title.
#[must_use]
pub const fn dialog_title() -> &'static str {
    "Choose a folder of print presets"
}

/// How many presets the folders hold.
#[must_use]
pub fn found(count: usize) -> String {
    match count {
        0 => "No presets saved yet.".to_owned(),
        1 => "1 preset in these folders.".to_owned(),
        n => format!("{n} presets in these folders."),
    }
}
