//! # `text::settings::snapshot` — the words on the snapshot settings
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/prefs/snapshot.md`.

use crate::prefs::snapshot::SnapshotForm;

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

/// The copy-form setting's name.
#[must_use]
pub const fn form_title() -> &'static str {
    "Snapshot copies as"
}

/// What the copy-form setting changes.
#[must_use]
pub const fn form_silence() -> &'static str {
    "What the Snapshot tool puts on the clipboard when it copies its box."
}

/// One choice's label.
#[must_use]
pub const fn form_label(form: SnapshotForm) -> &'static str {
    match form {
        SnapshotForm::Outlines => "Drawing, with text as outlines",
        SnapshotForm::KeepText => "Drawing, with text kept as text",
        SnapshotForm::Picture => "Picture only",
    }
}

/// The line under one choice.
#[must_use]
pub const fn form_note(form: SnapshotForm) -> &'static str {
    match form {
        SnapshotForm::Outlines => {
            "Scales cleanly and looks the same everywhere; the words cannot be edited."
        }
        SnapshotForm::KeepText => {
            "The words can be edited where they are pasted; a program without the font shows \
             them in a substitute. Text that cannot be kept stays as outlines."
        }
        SnapshotForm::Picture => {
            "A picture at the resolution above, and nothing else. It cannot be scaled \
             without blurring."
        }
    }
}
