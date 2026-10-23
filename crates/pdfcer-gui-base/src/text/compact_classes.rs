//! # `text::compact_classes` — the compact window's per-kind picture settings
//!
//! Every word of the split inside the Pictures fold of Save a compacted copy.

/// The split switch.
#[must_use]
pub const fn split() -> &'static str {
    "Separate settings for colour, grey and black-and-white pictures"
}

/// Hover on the split switch.
#[must_use]
pub const fn split_tooltip() -> &'static str {
    "Gives each kind of picture its own resolution and storage, as Acrobat's PDF Optimizer \
     does. Black-and-white scans usually need a higher resolution than photographs to stay \
     legible. Untick a kind to leave those pictures as they are."
}

/// The colour group's switch.
#[must_use]
pub const fn colour_heading() -> &'static str {
    "Colour pictures"
}

/// The grey group's switch.
#[must_use]
pub const fn grey_heading() -> &'static str {
    "Greyscale pictures"
}

/// The black-and-white group's switch.
#[must_use]
pub const fn mono_heading() -> &'static str {
    "Black-and-white pictures"
}
