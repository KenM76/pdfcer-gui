//! # `text::settings::keys` — the words on Settings ▸ Keyboard shortcuts
//!
//! No chord and no command name is written here: both come from the live
//! keymap and the command registry.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/settingspages/keys.md`.

/// The page's name in the list and its heading.
#[must_use]
pub const fn title() -> &'static str {
    "Keyboard shortcuts"
}

/// What the page changes.
#[must_use]
pub const fn silence() -> &'static str {
    "Change a command's key by pressing the new one, switch it off, or put \
     pdfcer's own key back."
}

/// Where the change applies.
#[must_use]
pub const fn radius() -> &'static str {
    "Applies to every window after Save, and is kept in your preferences file."
}

/// The filter box's hint.
#[must_use]
pub const fn filter_hint() -> &'static str {
    "Find a command"
}

/// The button that puts every key back.
#[must_use]
pub const fn reset_all() -> &'static str {
    "Reset all"
}

/// The heading of the commands that are on no ribbon tab.
#[must_use]
pub const fn group_other() -> &'static str {
    "Other commands"
}

/// The heading of the keys that work inside a text edit.
#[must_use]
pub const fn group_typing() -> &'static str {
    "While typing text"
}

/// Under the typing heading: the keys that cannot be changed.
#[must_use]
pub const fn typing_fixed() -> &'static str {
    "The commands above also work while a text edit is open, on whatever keys \
     you give them here. Keys that need no Ctrl or Alt are typed as text \
     there instead. These editing keys are fixed: the arrows, Home and End \
     move the caret, with Shift to select and Ctrl to move by word; Enter \
     starts a new line; Tab types spaces to the next stop; Backspace and \
     Delete remove text; Esc closes the edit."
}

/// The chord column for a command with no key.
#[must_use]
pub const fn no_key() -> &'static str {
    "—"
}

/// The button that waits for the new key.
#[must_use]
pub const fn change() -> &'static str {
    "Change"
}

/// The Change button while it waits.
#[must_use]
pub const fn press_a_key() -> &'static str {
    "Press a key… (Esc cancels)"
}

/// The button that switches a command's keys off.
#[must_use]
pub const fn off() -> &'static str {
    "Off"
}

/// The button that puts one command's own keys back.
#[must_use]
pub const fn reset() -> &'static str {
    "Reset"
}

/// The button that takes a key from the command holding it.
#[must_use]
pub const fn reassign() -> &'static str {
    "Reassign"
}

/// The button that drops a clash.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

/// A key another command already has.
#[must_use]
pub fn clash(chord: &str, holder: &str, wanted: &str) -> String {
    format!("{chord} is already used by {holder}. Reassign it to {wanted}?")
}

/// A key pdfcer keeps for page navigation and zoom.
#[must_use]
pub fn kept_for_viewing(chord: &str) -> String {
    format!("{chord} is kept for page navigation and zoom, and cannot be assigned.")
}

/// A key pdfcer keeps for the selection on the page.
#[must_use]
pub fn kept_for_canvas(chord: &str) -> String {
    format!(
        "{chord} is kept for moving, deleting and leaving the selection on the page, \
         and cannot be assigned."
    )
}

/// The page when the program has no keymap to edit.
#[must_use]
pub const fn unavailable() -> &'static str {
    "This build has no keyboard shortcuts to change."
}
