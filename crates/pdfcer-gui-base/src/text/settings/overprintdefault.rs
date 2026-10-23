//! # `text::settings::overprintdefault` — the words of the Display page's overprint default
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/prefs/overprint.md`.

use crate::prefs::OverprintDefault;

/// What it is.
#[must_use]
pub const fn overprint_title() -> &'static str {
    "Use overprint preview"
}

/// What happens if you never touch it.
#[must_use]
pub const fn overprint_silence() -> &'static str {
    "Every document opens showing overprinting ink as it prints. Acrobat's own default is only for PDF/X files."
}

/// What changing it affects.
#[must_use]
pub const fn overprint_radius() -> &'static str {
    "Decides how a document opens; View > Display > Overprint preview still turns it on or off. Affects only what you see."
}

/// One choice's label.
#[must_use]
pub const fn overprint_label(choice: OverprintDefault) -> &'static str {
    match choice {
        OverprintDefault::PdfxOnly => "Only for PDF/X files",
        OverprintDefault::Always => "Always (pdfcer's default)",
        OverprintDefault::Never => "Never",
        OverprintDefault::Automatic => "Automatic",
    }
}

/// One choice's description.
#[must_use]
pub const fn overprint_note(choice: OverprintDefault) -> &'static str {
    match choice {
        OverprintDefault::PdfxOnly => {
            "On for a file that says it is PDF/X, a print-ready standard. Off for everything else."
        }
        OverprintDefault::Always => {
            "On for every document. A page with no overprint looks the same either way."
        }
        OverprintDefault::Never => {
            "Off for every document, so overprinting ink hides what is beneath it."
        }
        OverprintDefault::Automatic => {
            "On for a document that asks for overprint anywhere. Off for one that does not."
        }
    }
}
