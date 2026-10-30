//! # `text::printmarks` — the words for **Printer's marks**
//!
//! File ▸ Print ▸ Printer's marks…: its window, its live check and the
//! receipt, which discloses how far the sheets grew and which pages had no
//! trim box of their own. Consumed by `pdfcer_gui::dialogs::printmarks`.

use pdfcer_core::printer_marks::PrinterMarksError;

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Printer's marks"
}

/// The standing rule, first line of the window.
#[must_use]
pub const fn intro() -> &'static str {
    "Draws marks for the print shop outside the trimmed page. Each sheet grows \
     to make room for them; the page inside the trim is not moved or scaled."
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

/// The crop-mark toggle.
#[must_use]
pub const fn trim() -> &'static str {
    "Crop marks"
}

/// The bleed-mark toggle.
#[must_use]
pub const fn bleed() -> &'static str {
    "Bleed marks"
}

/// The registration-target toggle.
#[must_use]
pub const fn registration() -> &'static str {
    "Registration marks"
}

/// The colour-bar toggle.
#[must_use]
pub const fn color_bars() -> &'static str {
    "Colour bars"
}

/// The page-information toggle.
#[must_use]
pub const fn page_info() -> &'static str {
    "Page information"
}

/// Tooltip of [`page_info`].
#[must_use]
pub const fn page_info_tooltip() -> &'static str {
    "A line under the page: the label below, then the page number and count."
}

/// Label of the page-information label box.
#[must_use]
pub const fn label() -> &'static str {
    "Label"
}

/// Label of the offset box.
#[must_use]
pub const fn offset() -> &'static str {
    "Gap from the trim (pt)"
}

/// Label of the mark-length box.
#[must_use]
pub const fn length() -> &'static str {
    "Mark length (pt)"
}

/// Label of the line-width box.
#[must_use]
pub const fn line_width() -> &'static str {
    "Line width (pt)"
}

/// Why the marks cannot be drawn, in the operator's terms.
#[must_use]
pub fn problem(e: &PrinterMarksError) -> String {
    match e {
        PrinterMarksError::NoMarks => "Tick at least one kind of mark.".to_owned(),
        PrinterMarksError::Geometry { what, .. } => {
            format!("The {what} must be above 0 and at most 144 pt.")
        }
        PrinterMarksError::Unencodable(c) => format!(
            "\u{201c}{c}\u{201d} cannot be printed in the label font. Use another character."
        ),
        PrinterMarksError::NoPages => "No pages are selected.".to_owned(),
        other => other.to_string(),
    }
}

/// Note under the controls: the sheets change size.
#[must_use]
pub const fn permanence() -> &'static str {
    "Undo removes the marks and restores the sheet size while this document \
     is open. Once saved, the larger sheet and its marks are part of the pages."
}

/// The commit button.
#[must_use]
pub const fn add() -> &'static str {
    "Add marks"
}

/// The cancel button.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

/// The receipt: pages marked and how far the largest sheet grew.
#[must_use]
pub fn receipt(pages: usize, grown_mm: f64) -> String {
    format!(
        "Added printer's marks to {pages} page(s). The sheets grew by up to {grown_mm:.1} mm \
         on each side to hold them."
    )
}

/// Disclosure: pages with no trim box were trimmed at their visible edge.
#[must_use]
pub fn trim_assumed(pages: usize) -> String {
    format!(
        "{pages} page(s) did not say where they are trimmed, so the marks were placed \
         at the page's visible edge."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_size_out_of_range_names_which() {
        let s = problem(&PrinterMarksError::Geometry {
            what: "line width",
            value: 0.0,
        });
        assert!(s.contains("line width"), "{s}");
    }
}
