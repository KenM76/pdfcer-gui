//! # `text::radiogroup` — the words for putting a radio button into a group
//!
//! Consumed by `app::actions::forms::radiogroup` (the edit's confirmation)
//! and `panels::properties::radiogroup` (the Properties row).

/// The Properties row's label.
pub const LABEL: &str = "Radio group";

/// The button that moves the selected radio button into the typed group.
pub const MOVE: &str = "Move to group";

/// The hover on [`MOVE`].
pub const MOVE_HOVER: &str = "Make this button a member of the named group. \
Buttons in one group are mutually exclusive: turning one on turns the others off. \
A name no field has yet starts a new group.";

/// The hover on the group name box.
pub const NAME_HOVER: &str = "The group this radio button belongs to. \
Pick an existing group from the list, or type a new name.";

/// Why [`MOVE`] is greyed: the name is already this button's group.
pub const SAME_GROUP: &str = "This button is already in that group.";

/// Why [`MOVE`] is greyed: the name is empty.
pub const EMPTY_NAME: &str = "Type the name of a group.";

/// Why [`MOVE`] is greyed: the name belongs to a field that is not a radio
/// button, which a radio button cannot join.
#[must_use]
pub fn not_a_group(name: &str) -> String {
    format!("{name} is a form field of another kind, not a radio group.")
}

/// Said after a pasted radio button joined the copied button's group.
#[must_use]
pub fn joined(group: &str, export: &str) -> String {
    format!("Added a button to radio group {group}. It turns on as \u{201c}{export}\u{201d}.")
}

/// Said after a radio button was moved into another group.
#[must_use]
pub fn moved(group: &str, export: &str) -> String {
    format!("Moved the button into radio group {group}. It turns on as \u{201c}{export}\u{201d}.")
}

/// Said when the move could not be folded into one undo entry.
pub const TWO_UNDOS: &str = "Undoing the move takes two presses rather than one.";
