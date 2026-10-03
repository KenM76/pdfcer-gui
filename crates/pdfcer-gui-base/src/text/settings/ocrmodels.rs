//! # `text::settings::ocrmodels` — the words on the OCR models page
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/prefs/ocrmodels.md`.

/// The page's heading.
#[must_use]
pub const fn group() -> &'static str {
    "OCR models"
}

/// The folder list's title.
#[must_use]
pub const fn title() -> &'static str {
    "Extra folders to find OCR models in"
}

/// What the list does, including the folder it never needs to name.
#[must_use]
pub const fn silence() -> &'static str {
    "Recognise text always looks in the models folder beside pdfcer-gui.exe first, then in \
     these, in order. A folder may hold one model, or several in folders of their own."
}

/// Where the change applies.
#[must_use]
pub const fn radius() -> &'static str {
    "Applies the next time Recognise text opens, and is kept in your preferences file."
}

/// Shown in place of an empty list.
#[must_use]
pub const fn folders_none() -> &'static str {
    "No extra folders. Recognise text offers only the models that came with pdfcer."
}

/// The Add button's hover.
#[must_use]
pub const fn add_hover() -> &'static str {
    "Every model in the folder is offered in Recognise text's model list. When two folders \
     hold a model of the same name, the one listed first is used."
}

/// The folder picker's title bar.
#[must_use]
pub const fn dialog_title() -> &'static str {
    "Choose a folder holding OCR models"
}

/// The heading over the models the folders hold.
#[must_use]
pub const fn found() -> &'static str {
    "Models found:"
}

/// Shown when the folders hold no model at all.
#[must_use]
pub const fn found_none() -> &'static str {
    "No models found in these folders."
}
