//! # `text::print::running` — a header or footer on the printed copy only
//!
//! The words for `pdfcer_gui::dialogs::print::running`, on the Print window's
//! Comments tab. The edge and alignment names are `text::running`'s.

/// The section's heading.
#[must_use]
pub const fn heading() -> &'static str {
    "Header and footer"
}

/// The rule under the heading: the file is not changed.
#[must_use]
pub const fn intro() -> &'static str {
    "Printed on this copy only; the document is not changed. {page} becomes \
     each page's number and {pages} the page count."
}

/// Label of the header line.
#[must_use]
pub const fn header() -> &'static str {
    "Header"
}

/// Label of the footer line.
#[must_use]
pub const fn footer() -> &'static str {
    "Footer"
}

/// Hover on either text box.
#[must_use]
pub const fn line_tooltip() -> &'static str {
    "Leave empty for none. The preview shows it once you leave the box."
}

/// Why the printed copy could not carry the header or footer.
#[must_use]
pub fn refused(reason: &str) -> String {
    format!("Can't print the header or footer: {reason}")
}
