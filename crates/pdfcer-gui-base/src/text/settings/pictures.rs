//! # `text::settings::pictures` — the words of the Display page's picture setting
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/settings/pictures.md`.

/// What it is.
#[must_use]
pub const fn pictures_title() -> &'static str {
    "Enlarged pictures"
}

/// What happens if you never touch it.
#[must_use]
pub const fn pictures_silence() -> &'static str {
    "A picture shown bigger than its own pixels — a scanned page zoomed in — is smoothed, as Acrobat shows it."
}

/// What changing it affects.
#[must_use]
pub const fn pictures_radius() -> &'static str {
    "Affects only what you see. Does not change the file, and does not change what prints or exports."
}

/// The checkbox.
#[must_use]
pub const fn pictures_label() -> &'static str {
    "Smooth enlarged pictures"
}

/// The note under the checkbox.
#[must_use]
pub const fn pictures_note() -> &'static str {
    "Turn this off to see each pixel of a scan as a square, for checking its real resolution."
}
