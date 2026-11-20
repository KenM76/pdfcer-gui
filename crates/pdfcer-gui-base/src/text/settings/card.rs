//! # `text::settings::card` — the words of the Display page's graphics-card setting
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/settings/card.md`.

/// What it is.
#[must_use]
pub const fn card_title() -> &'static str {
    "Graphics card"
}

/// What happens if you never touch it.
#[must_use]
pub const fn card_silence() -> &'static str {
    "Pages are drawn on the processor."
}

/// What changing it affects.
#[must_use]
pub const fn card_radius() -> &'static str {
    "Affects only how the pages on screen are drawn, not what they look like. Does not change the file, printing or exports."
}

/// The checkbox.
#[must_use]
pub const fn card_label() -> &'static str {
    "Draw pages on the graphics card"
}

/// The note under the checkbox.
#[must_use]
pub const fn card_note() -> &'static str {
    "The picture is the same either way. A page the graphics card cannot draw yet is drawn on the processor, and Render notes in the status bar say which way each page was drawn and why."
}
