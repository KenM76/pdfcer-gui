//! Settings text for what the ribbon's and the dock's tabs show (O299).

use crate::tabface::TabLabel;

/// The setting's name.
#[must_use]
pub const fn tab_face_title() -> &'static str {
    "What tabs show"
}

/// What happens if you never touch it.
#[must_use]
pub const fn tab_face_silence() -> &'static str {
    "Every tab shows its picture and its name."
}

/// What it costs, and what it does not affect.
#[must_use]
pub const fn tab_face_radius() -> &'static str {
    "Applies at once. The ribbon's tabs and the side panels' tabs are set separately. A tab showing only its picture still names itself when you hover over it, and a tab with no picture always shows its name."
}

/// The heading over the ribbon's three options.
#[must_use]
pub const fn tab_face_ribbon() -> &'static str {
    "Ribbon tabs (Home, Edit, View…)"
}

/// The heading over the dock's three options.
#[must_use]
pub const fn tab_face_dock() -> &'static str {
    "Panel tabs (the side areas)"
}

/// One option's name.
#[must_use]
pub const fn tab_face_label(face: TabLabel) -> &'static str {
    match face {
        TabLabel::Both => "Picture and name",
        TabLabel::Glyph => "Picture only",
        TabLabel::Text => "Name only",
    }
}
