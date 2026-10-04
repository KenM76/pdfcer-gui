//! Words for the selected annotation's show, print and lock switches.

/// The section heading.
#[must_use]
pub const fn heading() -> &'static str {
    "Visibility and lock"
}

/// The on-screen switch.
#[must_use]
pub const fn on_screen() -> &'static str {
    "Show on screen"
}

/// What turning the on-screen switch off does.
#[must_use]
pub const fn on_screen_hover() -> &'static str {
    "Off: the mark is not drawn on screen and can no longer be clicked. It stays \
     in the Comments list, where Show on screen brings it back."
}

/// The print switch.
#[must_use]
pub const fn prints() -> &'static str {
    "Print"
}

/// What the print switch decides.
#[must_use]
pub const fn prints_hover() -> &'static str {
    "Whether the mark appears when the document is printed."
}

/// The lock switch.
#[must_use]
pub const fn locked() -> &'static str {
    "Locked"
}

/// What locking does.
#[must_use]
pub const fn locked_hover() -> &'static str {
    "A locked mark cannot be moved, resized, restyled or deleted until it is \
     unlocked here. Other programs honour the same lock."
}

/// The Comments-list control that brings a hidden mark back on screen.
#[must_use]
pub const fn show_again() -> &'static str {
    "Show on screen"
}
