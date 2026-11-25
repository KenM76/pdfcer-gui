//! Settings > Display: what a document tab remembers.

/// The setting's name.
#[must_use]
pub const fn tab_mode_title() -> &'static str {
    "Each tab's mode"
}

/// What happens if you never touch it.
#[must_use]
pub const fn tab_mode_silence() -> &'static str {
    "Read, Review and Edit apply to the whole window: switching tabs keeps the mode you are in."
}

/// What it costs, and what it does not affect.
#[must_use]
pub const fn tab_mode_radius() -> &'static str {
    "Applies at once. A newly opened document starts in the mode you are in. The mode is remembered while the document stays open."
}

/// The checkbox.
#[must_use]
pub const fn tab_mode_label() -> &'static str {
    "Remember the mode of each tab"
}
