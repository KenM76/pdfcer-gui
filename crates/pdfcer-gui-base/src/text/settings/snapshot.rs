//! # `text::settings::snapshot` — the words on the snapshot resolution setting
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/prefs/snapshot.md`.

/// The setting's name.
#[must_use]
pub const fn title() -> &'static str {
    "Snapshot resolution"
}

/// What the setting changes.
#[must_use]
pub const fn silence() -> &'static str {
    "How sharp the picture is when the Snapshot tool copies its box. Higher is \
     sharper and larger."
}

/// Where the change applies.
#[must_use]
pub const fn radius() -> &'static str {
    "Applies to every snapshot copied after Save, and is kept in your \
     preferences file."
}

/// The box's label.
#[must_use]
pub const fn label() -> &'static str {
    "Dots per inch"
}

/// The range under the box.
#[must_use]
pub fn range_note(min: u32, max: u32, default: u32) -> String {
    format!("From {min} to {max}; pdfcer starts at {default}.")
}
