//! Words for the text section's spacing and position rows.

/// The fold that holds the spacing and position rows.
#[must_use]
pub const fn text_spacing_heading() -> &'static str {
    "Spacing and position"
}

/// Character spacing.
#[must_use]
pub const fn text_tracking_label() -> &'static str {
    "Tracking"
}

/// Word spacing.
#[must_use]
pub const fn text_word_spacing_label() -> &'static str {
    "Word spacing"
}

/// Horizontal scaling.
#[must_use]
pub const fn text_width_label() -> &'static str {
    "Width"
}

/// Free baseline shift.
#[must_use]
pub const fn text_rise_label() -> &'static str {
    "Baseline shift"
}

/// The unit after a value in thousandths of an em.
#[must_use]
pub const fn per_mille_em_suffix() -> &'static str {
    " /1000 em"
}

/// The unit after a width percentage.
#[must_use]
pub const fn percent_suffix() -> &'static str {
    " %"
}

/// The button that applies one row's value.
#[must_use]
pub const fn text_spacing_set() -> &'static str {
    "Set"
}

/// Hover on Tracking.
#[must_use]
pub const fn text_tracking_hint() -> &'static str {
    "Extra space after every character, in thousandths of the text size. 0 is the font's own spacing; negative tightens."
}

/// Hover on Word spacing.
#[must_use]
pub const fn text_word_spacing_hint() -> &'static str {
    "Extra space at every space character, in thousandths of the text size. Fonts that store a character in more than one byte ignore it."
}

/// Hover on Width.
#[must_use]
pub const fn text_width_hint() -> &'static str {
    "Stretches or squeezes the letters sideways; 100 % is normal."
}

/// Hover on Baseline shift.
#[must_use]
pub const fn text_rise_hint() -> &'static str {
    "Moves the text up (positive) or down (negative) from its line, in thousandths of the text size, without resizing it."
}

/// The position row's label.
#[must_use]
pub const fn text_script_label() -> &'static str {
    "Position"
}

/// The three position buttons, in order: normal, superscript, subscript.
#[must_use]
pub const fn text_script_names() -> [&'static str; 3] {
    ["Normal", "Superscript", "Subscript"]
}

/// Hover on the position buttons.
#[must_use]
pub const fn text_script_hint() -> &'static str {
    "Superscript and subscript raise or lower the text and make it smaller; Normal puts it back on the line at its own size."
}
