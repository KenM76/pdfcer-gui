//! # `text::print::colour` — the words for the Print dialog's colour choice

/// Above the three choices.
#[must_use]
pub const fn heading() -> &'static str {
    "Colour"
}

/// As the document is coloured.
#[must_use]
pub const fn colour() -> &'static str {
    "In colour"
}

/// Composite grey.
#[must_use]
pub const fn grey() -> &'static str {
    "In grey (greyscale)"
}

/// Every mark solid black.
#[must_use]
pub const fn black() -> &'static str {
    "Black only"
}

/// Under the choices while black only is chosen.
#[must_use]
pub const fn black_note() -> &'static str {
    "Everything that is not white prints solid black, including pale fills and photographs."
}
