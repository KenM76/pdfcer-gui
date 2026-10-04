//! Copy for `panels::properties::fieldextras`: a text field's scrolling,
//! spell-check and file-select flags, and every field's export name (`/TM`).

/// `/Ff` bit 24 (DoNotScroll), written as the positive the operator expects.
#[must_use]
pub const fn flag_scroll() -> &'static str {
    "Scroll long text"
}

/// See [`flag_scroll`].
#[must_use]
pub const fn flag_scroll_hover() -> &'static str {
    "When off, text longer than the box stops at its edge instead of scrolling, so nothing \
     can be typed that would not print. Recorded in the file for the reader that fills the \
     form in."
}

/// `/Ff` bit 21 (FileSelect).
#[must_use]
pub const fn flag_file_select() -> &'static str {
    "Sends a file"
}

/// See [`flag_file_select`].
#[must_use]
pub const fn flag_file_select_hover() -> &'static str {
    "Turns this field into a file chooser: the reader that fills the form in offers a Browse \
     button, and submitting the form sends the chosen file's contents."
}

/// Shown under the checkbox while the flag is set: what the field now is.
#[must_use]
pub const fn file_select_note() -> &'static str {
    "Submitting this form sends the contents of the file named in this field, not just its \
     name."
}

/// `/TM`, the mapping name.
#[must_use]
pub const fn label_export_name() -> &'static str {
    "Export name"
}

/// The empty box's hint: what an export uses when there is no `/TM`.
#[must_use]
pub const fn label_export_name_hint() -> &'static str {
    "the field's own name"
}

/// See [`label_export_name`].
#[must_use]
pub const fn label_export_name_hover() -> &'static str {
    "The name this field's value is saved under when the form's data is exported or \
     submitted, in place of the field name. Nothing on the page changes. Leave it empty to \
     use the field name."
}

/// `/Ff` bit 3 (NoExport), written as the positive.
#[must_use]
pub const fn flag_sent() -> &'static str {
    "Sent with the form"
}

/// See [`flag_sent`].
#[must_use]
pub const fn flag_sent_hover() -> &'static str {
    "When off, this field's value is left out when the form's data is submitted or exported. It still shows and prints, and can still be filled in."
}
