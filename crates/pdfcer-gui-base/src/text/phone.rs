//! # `phone` — the words of the phone layout (`MOBILE_DESIGN.md` §3)

/// The app bar's title while no document is open.
pub const NO_DOCUMENT: &str = "pdfcer";

/// The app bar's overflow button's name, which holds the File tab.
pub const MORE: &str = "More options";

/// The bottom bar's last cell, which opens the tool sheet.
pub const ALL_TOOLS: &str = "All tools";

/// The app bar's second line: where the operator is in the document.
#[must_use]
pub fn page_subtitle(page_1_based: usize, total: usize) -> String {
    format!("Page {page_1_based} of {total}")
}

/// Shown on the canvas when nothing is open, in place of the desktop's
/// keyboard and command-line advice.
pub const CANVAS_NO_DOCUMENT: &str = "No document open. Tap the folder at the top to open a PDF, or open one from Files, email or Drive.";
