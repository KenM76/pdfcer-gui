//! # `text::bates` — the words for **Bates numbering**
//!
//! Pages ▸ Stamp ▸ Bates numbering…: its window, its live check of the
//! numbering, and the receipt its commit raises. Design:
//! `docs/modules/pdfcer-gui/dialogs/bates.md`.

use pdfcer_core::bates::{BatesError, BatesPosition};

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Bates numbering"
}

/// The standing rule, first line of the window.
#[must_use]
pub const fn intro() -> &'static str {
    "Stamps a running number on each page as page content, so it prints and \
     saves like the rest of the page. Pages are numbered in page order."
}

/// The whole-document scope choice.
#[must_use]
pub fn scope_all(of: usize) -> String {
    format!("All {of} page(s)")
}

/// The rail-pick scope choice.
#[must_use]
pub fn scope_picked(picked: usize) -> String {
    format!("The {picked} page(s) picked in the page rail")
}

/// Label of the prefix box.
#[must_use]
pub const fn prefix() -> &'static str {
    "Prefix"
}

/// Label of the suffix box.
#[must_use]
pub const fn suffix() -> &'static str {
    "Suffix"
}

/// Label of the digit-count box.
#[must_use]
pub const fn digits() -> &'static str {
    "Digits"
}

/// Tooltip of [`digits`].
#[must_use]
pub const fn digits_tooltip() -> &'static str {
    "The number is padded with leading zeros to this many digits (1 to 15)."
}

/// Label of the start-number box.
#[must_use]
pub const fn start() -> &'static str {
    "Start at"
}

/// Tooltip of [`start`].
#[must_use]
pub const fn start_tooltip() -> &'static str {
    "The number on the first stamped page. To continue a batch, enter the \
     next number the previous document reported."
}

/// Heading above the six position choices.
#[must_use]
pub const fn position_heading() -> &'static str {
    "Position on the page"
}

/// The name of one position.
#[must_use]
pub const fn position(p: BatesPosition) -> &'static str {
    match p {
        BatesPosition::TopLeft => "Top left",
        BatesPosition::TopCenter => "Top centre",
        BatesPosition::TopRight => "Top right",
        BatesPosition::BottomLeft => "Bottom left",
        BatesPosition::BottomCenter => "Bottom centre",
        BatesPosition::BottomRight => "Bottom right",
        _ => "Other",
    }
}

/// Label of the margin box.
#[must_use]
pub const fn margin() -> &'static str {
    "Distance from the edges (mm)"
}

/// Label of the font-size box.
#[must_use]
pub const fn font_size() -> &'static str {
    "Text size (pt)"
}

/// The live preview of the first and last labels.
#[must_use]
pub fn preview(first: &str, last: &str) -> String {
    if first == last {
        format!("Label: {first}")
    } else {
        format!("Labels: {first} to {last}")
    }
}

/// Why the current numbering cannot be stamped, in the operator's terms.
#[must_use]
pub fn problem(e: &BatesError) -> String {
    match e {
        BatesError::Digits(_) => "Use 1 to 15 digits.".to_owned(),
        BatesError::Overflow { number, digits } => format!(
            "{number} does not fit in {digits} digit(s). Add a digit or lower the start number."
        ),
        BatesError::Unencodable(c) => format!(
            "\u{201c}{c}\u{201d} cannot be printed in the label font. Use another character."
        ),
        BatesError::Geometry { .. } => {
            "The distance from the edges must be 0 or more and the text size above 0.".to_owned()
        }
        BatesError::NoPages => "No pages are selected.".to_owned(),
        other => other.to_string(),
    }
}

/// Note under the preview: stamping cannot be told apart from the page later.
#[must_use]
pub const fn permanence() -> &'static str {
    "Undo removes the numbers while this document is open. Once saved, they \
     are part of the pages; a page stamped twice carries two labels."
}

/// The commit button.
#[must_use]
pub const fn stamp() -> &'static str {
    "Stamp"
}

/// The cancel button.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

/// The receipt after a successful stamp.
#[must_use]
pub fn receipt(pages: usize, first: &str, last: &str, next: u64) -> String {
    format!(
        "Bates numbered {pages} page(s), {first} to {last}. The next document in \
         the batch starts at {next}."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_position_has_its_own_name() {
        let names: std::collections::BTreeSet<_> =
            BatesPosition::ALL.iter().map(|p| position(*p)).collect();
        assert_eq!(names.len(), BatesPosition::ALL.len());
        assert!(!names.contains("Other"));
    }

    #[test]
    fn overflow_names_the_number_and_the_remedy() {
        let s = problem(&BatesError::Overflow {
            number: 1000,
            digits: 3,
        });
        assert!(s.contains("1000") && s.contains("3 digit"), "{s}");
    }
}
