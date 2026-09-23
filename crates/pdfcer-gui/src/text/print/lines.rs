//! # `text::print::lines` — one fixed line width for the whole print
//!
//! The words for [`crate::dialogs::print::lines`].

/// The checkbox that replaces the document's line weights with one width.
#[must_use]
pub const fn lines_fixed() -> &'static str {
    "Fixed line width"
}

/// Hover on the fixed-line-width checkbox.
#[must_use]
pub const fn lines_fixed_tooltip() -> &'static str {
    "Ignore the document's line weights and print every line at one width. \
     Only this print changes; the document does not."
}

/// The checkbox that lets pdfcer choose the width.
#[must_use]
pub const fn lines_auto() -> &'static str {
    "Auto"
}

/// Hover on the Auto checkbox.
#[must_use]
pub const fn lines_auto_tooltip() -> &'static str {
    "The thinnest line that still prints cleanly and reads easily: 0.1 mm, \
     or one printer dot if the resolution is too coarse for that. Clear it to \
     type a width."
}

/// Hover on the typed width.
#[must_use]
pub const fn lines_width_tooltip() -> &'static str {
    "Every line's width on the paper, whatever the print's scale."
}

/// The small line under the controls while a fixed width is on.
#[must_use]
pub const fn lines_caption(auto: bool) -> &'static str {
    if auto {
        "Every line prints at 0.1 mm, or one printer dot where that is wider."
    } else {
        "Every line prints at the width above, whatever the scale."
    }
}
