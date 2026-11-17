//! # `text::commands::grouping` — the labels of Group and Ungroup, on the
//! Edit tab (page objects) and the Markup tab (markups)
//!
//! The labels are the same on both tabs; the tooltips say which kind of thing.

use super::CommandText;

/// `edit.group`
#[must_use]
pub const fn edit_group() -> CommandText {
    CommandText::new(
        "Group",
        "Gather the selected objects into one object that selects, moves and copies as a whole.",
    )
}

/// `edit.ungroup`
#[must_use]
pub const fn edit_ungroup() -> CommandText {
    CommandText::new(
        "Ungroup",
        "Take the selected group or placed drawing apart into its separate objects.",
    )
}

/// `markup.group`
#[must_use]
pub const fn markup_group() -> CommandText {
    CommandText::new(
        "Group",
        "Join the selected markups into one group that selects and moves as a whole.",
    )
}

/// `markup.ungroup`
#[must_use]
pub const fn markup_ungroup() -> CommandText {
    CommandText::new(
        "Ungroup",
        "Release the selected markup group into its separate markups.",
    )
}
