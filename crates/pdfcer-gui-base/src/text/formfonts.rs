//! The sentences for Edit ▸ Forms ▸ Repair fonts, which moves every inline
//! font in the form's default resources into an object of its own so Acrobat
//! draws the fields' values.

/// The status line after a repair that moved `fonts` fonts.
#[must_use]
pub fn repaired(fonts: usize) -> String {
    format!(
        "Repaired {fonts} form font(s): filled fields now show their values in Acrobat. The \
         fields look the same here. Press Ctrl+Z to undo."
    )
}

/// The status line when the form's fonts need nothing done.
#[must_use]
pub const fn nothing_to_repair() -> &'static str {
    "This form's fonts need no repair: the document is unchanged."
}
