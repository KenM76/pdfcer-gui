//! # `text::settings::richstyle` — rich-text field styling in Settings ▸ Forms

/// Title for the rich-text style precedence setting.
pub const fn rich_style_title() -> &'static str {
    "The font, size or colour pdfcer uses when it fills in a formatted text field"
}

/// What the standard leaves open here.
pub const fn rich_style_silence() -> &'static str {
    "A formatted text field can name a font, size and colour in two places: its default style and its default appearance. Where a piece of text sets none of its own and the two places disagree, the standard does not say which one wins."
}

/// The radius line.
pub const fn rich_style_radius() -> &'static str {
    "Changes how pdfcer draws formatted text fields it fills in or redraws, and what is saved for them. Text that sets its own font, size or colour keeps it either way. Bold, italic, underline and alignment always come from the default style."
}

/// Label for the default.
pub const fn rich_style_ds_label() -> &'static str {
    "The field's default style first (pdfcer's default)"
}

/// Note for the default.
pub const fn rich_style_ds_note() -> &'static str {
    "The default appearance only fills in what the default style leaves unset."
}

/// Label for the alternative.
pub const fn rich_style_da_label() -> &'static str {
    "The field's default appearance first"
}

/// Note for the alternative.
pub const fn rich_style_da_note() -> &'static str {
    "The default style only fills in what the default appearance leaves unset. A size of 0 there, meaning fit the text to the box, counts as unset."
}
