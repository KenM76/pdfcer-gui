//! # `text::print::saved` — the words of the Print window's Preset row
//!
//! The words for `pdfcer_gui::dialogs::print::saved`.

use crate::prefs::printpresets::{MAX_NAME, NameRefusal};

/// The row's label.
#[must_use]
pub const fn label() -> &'static str {
    "Preset:"
}

/// The dropdown's text when no preset is chosen.
#[must_use]
pub const fn none_chosen() -> &'static str {
    "(choose a saved preset)"
}

/// The dropdown's text when there are no presets.
#[must_use]
pub const fn none_saved() -> &'static str {
    "(none saved)"
}

/// The dropdown's hover.
#[must_use]
pub const fn combo_hover() -> &'static str {
    "Put a saved set of Print settings into this window. The page range is never part of a \
     preset. Settings > Print presets lists the folders searched."
}

/// The suffix marking settings changed since a preset was chosen.
#[must_use]
pub const fn modified() -> &'static str {
    " (changed)"
}

/// The Save as button.
#[must_use]
pub const fn save_as() -> &'static str {
    "Save as…"
}

/// The Save as button's hover.
#[must_use]
pub const fn save_as_hover() -> &'static str {
    "Save every setting in this window, except the page range, as a preset you can choose \
     from the list."
}

/// Save as, greyed because there is nowhere to save.
#[must_use]
pub const fn save_nowhere() -> &'static str {
    "pdfcer has no settings folder here, so a preset has nowhere to be saved."
}

/// The name box's hint.
#[must_use]
pub const fn name_hint() -> &'static str {
    "Preset name"
}

/// The button that saves under the typed name.
#[must_use]
pub const fn save() -> &'static str {
    "Save"
}

/// The same button when the name is already a saved preset.
#[must_use]
pub const fn replace() -> &'static str {
    "Replace"
}

/// The Replace button's hover.
#[must_use]
pub const fn replace_hover() -> &'static str {
    "A preset of this name is already saved; this replaces it."
}

/// Cancel naming.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

/// The Delete button.
#[must_use]
pub const fn delete() -> &'static str {
    "Delete"
}

/// The Delete button's hover.
#[must_use]
pub const fn delete_hover() -> &'static str {
    "Delete the chosen preset's file."
}

/// Delete, greyed with nothing chosen.
#[must_use]
pub const fn delete_nothing() -> &'static str {
    "Choose a preset first."
}

/// The second press that deletes.
#[must_use]
pub fn delete_confirm(name: &str) -> String {
    format!("Delete \"{name}\"")
}

/// A name refused, and why.
#[must_use]
pub fn refused(why: NameRefusal) -> String {
    match why {
        NameRefusal::Empty => "Type a name for the preset.".to_owned(),
        NameRefusal::TooLong => format!("A preset name is at most {MAX_NAME} characters."),
        NameRefusal::Forbidden(c) => format!("A preset name cannot hold the character {c}."),
        NameRefusal::BadEnd => "A preset name cannot end with a dot.".to_owned(),
    }
}

/// A preset saved.
#[must_use]
pub fn saved(name: &str) -> String {
    format!("Saved the preset \"{name}\".")
}

/// A preset put into the window.
#[must_use]
pub fn applied(name: &str, unread: usize) -> String {
    match unread {
        0 => format!("Using the preset \"{name}\"."),
        n => format!(
            "Using the preset \"{name}\"; {n} of its lines were not understood and kept the defaults."
        ),
    }
}

/// A preset's printer is not on this machine.
#[must_use]
pub fn printer_missing(printer: &str) -> String {
    format!("Its printer \"{printer}\" is not on this computer, so the printer is unchanged.")
}

/// A preset deleted.
#[must_use]
pub fn deleted(name: &str) -> String {
    format!("Deleted the preset \"{name}\".")
}

/// A file operation failed.
#[must_use]
pub fn failed(what: &str, detail: &str) -> String {
    format!("Could not {what} the preset: {detail}")
}

/// The verb of [`failed`] for saving.
#[must_use]
pub const fn verb_save() -> &'static str {
    "save"
}

/// The verb of [`failed`] for reading.
#[must_use]
pub const fn verb_read() -> &'static str {
    "read"
}

/// The verb of [`failed`] for deleting.
#[must_use]
pub const fn verb_delete() -> &'static str {
    "delete"
}
