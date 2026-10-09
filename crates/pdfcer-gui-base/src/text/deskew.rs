//! # `text::deskew` — every word File ▸ Straighten scans says
//!
//! Consumed by `pdfcer_gui::dialogs::deskew`.

use super::panels::byte_size;

/// The window's title.
#[must_use]
pub fn title() -> String {
    "Straighten scans".to_owned()
}

/// What the window does, above its choices.
#[must_use]
pub fn intro() -> &'static str {
    "Measures how far each scanned page is tilted and turns the scan back level. Only the scanned picture turns; anything else on the page stays where it is. One Ctrl+Z undoes the whole run."
}

/// The choice that protects pages already carrying text.
#[must_use]
pub fn skip_text() -> &'static str {
    "Skip pages that already have text"
}

/// Why that choice is on by default.
#[must_use]
pub fn skip_text_tooltip() -> &'static str {
    "Text on a page, typed or recognised by OCR, does not turn with the picture. Straightening the picture under it would leave the two out of line, so such pages are left alone unless you clear this."
}

/// Narrow the run to the pictures selected on the current page.
#[must_use]
pub fn selected_only(count: usize) -> String {
    format!("Only the selected picture(s) on this page ({count})")
}

/// The button that starts the run.
#[must_use]
pub fn run_button() -> String {
    "Straighten".to_owned()
}

/// Why Straighten is greyed.
#[must_use]
pub fn names_nothing() -> &'static str {
    "The pages you chose name no page of this document."
}

/// The button that ends the run after the page in hand.
#[must_use]
pub fn stop_button() -> String {
    "Stop".to_owned()
}

/// Closes the window.
#[must_use]
pub fn close_button() -> String {
    "Close".to_owned()
}

/// Closes the window before a run.
#[must_use]
pub fn cancel_button() -> String {
    "Cancel".to_owned()
}

/// While the run is going.
#[must_use]
pub fn progress(at: usize, of: usize) -> String {
    format!("Straightening page {at} of {of}…")
}

/// A page whose scan was turned. `page` is 1-based.
#[must_use]
pub fn straightened(page: usize, degrees: f64, confidence: f64) -> String {
    format!(
        "Page {page}: corrected a tilt of {:.2}° (measured with {:.0}% confidence).",
        degrees.abs(),
        confidence * 100.0
    )
}

/// A page that was measured level enough to leave.
#[must_use]
pub fn already_level(page: usize, degrees: f64) -> String {
    format!(
        "Page {page}: already level (off by {:.2}°, too little to correct).",
        degrees.abs()
    )
}

/// A page whose measurement was not trusted.
#[must_use]
pub fn unsure(page: usize, degrees: f64, confidence: f64) -> String {
    format!(
        "Page {page}: left alone. It measured {:.2}°, but with only {:.0}% confidence.",
        degrees.abs(),
        confidence * 100.0
    )
}

/// A page whose picture holds too little ink to measure.
#[must_use]
pub fn unmeasurable(page: usize) -> String {
    format!("Page {page}: left alone. Its picture holds too little to measure a tilt from.")
}

/// A page with no picture to straighten.
#[must_use]
pub fn no_picture(page: usize) -> String {
    format!("Page {page}: no scanned picture.")
}

/// A page skipped because it already has text.
#[must_use]
pub fn has_text(page: usize) -> String {
    format!("Page {page}: skipped, it already has text.")
}

/// A page the engine declined; `reason` is its sentence, verbatim.
#[must_use]
pub fn refused(page: usize, reason: &str) -> String {
    format!("Page {page}: not straightened. {reason}")
}

/// A straightened page that carried text which did not turn with it.
#[must_use]
pub fn text_not_turned(page: usize) -> String {
    format!("Page {page} has text that did not turn with its picture; it may no longer line up.")
}

/// A page measured while another part of the program held the document.
#[must_use]
pub fn busy(page: usize) -> String {
    format!("Page {page}: not straightened. The document was busy; run it again for this page.")
}

/// The run's corrections could not be folded into one undo step.
#[must_use]
pub fn unfolded() -> &'static str {
    "Each straightened page is its own undo step this time, so undoing the run takes one Ctrl+Z per page."
}

/// Said while the run waits for its document to be back on screen.
#[must_use]
pub fn paused() -> &'static str {
    "Paused while another document is on screen. Switch back to this one's tab to carry on."
}

/// A turn that reached a different document than its run's.
#[must_use]
pub fn elsewhere(page: usize) -> String {
    format!("Page {page}: not straightened. Another document was on screen.")
}

/// The run's headline.
#[must_use]
pub fn done(corrected: usize, of: usize) -> String {
    format!("Straightened {corrected} of {of} page(s).")
}

/// The run was stopped before its last page.
#[must_use]
pub fn stopped(done: usize, of: usize) -> String {
    format!("Stopped after {done} of {of} page(s). What was straightened is kept.")
}

/// The straightened pictures are stored losslessly and so take more room.
#[must_use]
pub fn growth(old: usize, new: usize) -> String {
    format!(
        "The straightened pictures are stored without loss, so they take more room: {} became {}.",
        byte_size(old),
        byte_size(new)
    )
}
