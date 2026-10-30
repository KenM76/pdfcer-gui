//! # `text::print::presets` — the document's own print settings
//!
//! The words for `pdfcer_gui::dialogs::print::presets`.

use crate::printspooler::Duplex;

/// Above the list of settings taken from the document.
#[must_use]
pub const fn heading() -> &'static str {
    "From this document's print settings:"
}

/// The document asks for actual size.
#[must_use]
pub const fn scale_actual() -> &'static str {
    "actual size"
}

/// The document requires actual size.
#[must_use]
pub const fn scale_enforced() -> &'static str {
    "actual size, which the document requires"
}

/// Hover on a scale choice greyed by the document.
#[must_use]
pub const fn scale_locked_tooltip() -> &'static str {
    "This document requires printing at actual size, so the size cannot be changed."
}

/// The document names a page range.
#[must_use]
pub fn pages(spec: &str) -> String {
    format!("pages {spec}")
}

/// The document's page range names no page it has.
#[must_use]
pub const fn pages_outside(_page_count: usize) -> &'static str {
    "a page range naming no page in this file, not used"
}

/// The document asks for copies; `used` is fewer when it asked too many.
#[must_use]
pub fn copies(asked: u32, used: u16) -> String {
    if u32::from(used) < asked {
        format!("{used} copies (the document asks for {asked}; the most is {used})")
    } else if used == 1 {
        "1 copy".to_owned()
    } else {
        format!("{used} copies")
    }
}

/// The document's two-sided setting.
#[must_use]
pub const fn duplex(d: Duplex) -> &'static str {
    match d {
        Duplex::Simplex => "one-sided",
        Duplex::LongEdge => "two-sided, flipped on the long edge",
        Duplex::ShortEdge => "two-sided, flipped on the short edge",
    }
}

/// The document's tray setting.
#[must_use]
pub const fn tray(by_size: bool) -> &'static str {
    if by_size {
        "paper tray chosen by page size"
    } else {
        "paper tray not chosen by page size"
    }
}

/// `/PrintArea` or `/PrintClip` names a box other than the crop box.
#[must_use]
pub fn boundary(name: &str) -> String {
    format!("the document asks to print its {name}; pdfcer prints the crop box")
}

/// An entry the document has but that could not be read.
#[must_use]
pub fn ignored(why: &str) -> String {
    format!("not used, unreadable: {why}")
}

/// One line of the list under `heading`.
#[must_use]
pub fn bullet(line: &str) -> String {
    format!("• {line}")
}
