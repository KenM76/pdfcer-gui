//! # `text::enlarge` — every word File ▸ Enlarge scans says
//!
//! Consumed by `pdfcer_gui::dialogs::deskew`, which runs both scan jobs; the
//! words the two share (Stop, Close, the page scope) are `text::deskew`'s, and
//! the factor and kernel choices are `text::ocrupscale`'s.

use super::panels::byte_size;

/// The window's title.
#[must_use]
pub fn title() -> String {
    "Enlarge scans".to_owned()
}

/// What the window does, above its choices.
#[must_use]
pub fn intro() -> &'static str {
    "Gives each scanned picture more pixels, smoothing between the ones it has. It adds no detail; it helps where small print is too coarse to recognise or blocky at high zoom. The page looks the same size; the file grows. One Ctrl+Z undoes the whole run."
}

/// The button that starts the run.
#[must_use]
pub fn run_button() -> String {
    "Enlarge".to_owned()
}

/// While the run is going.
#[must_use]
pub fn progress(at: usize, of: usize) -> String {
    format!("Enlarging page {at} of {of}…")
}

/// A page whose picture was enlarged. `page` is 1-based; sizes in pixels.
#[must_use]
pub fn enlarged(page: usize, from: (u32, u32), to: (u32, u32), bytes: (usize, usize)) -> String {
    format!(
        "Page {page}: {} × {} became {} × {} pixels ({} became {}).",
        from.0,
        from.1,
        to.0,
        to.1,
        byte_size(bytes.0),
        byte_size(bytes.1)
    )
}

/// A page the engine declined; `reason` is its sentence, verbatim.
#[must_use]
pub fn refused(page: usize, reason: &str) -> String {
    format!("Page {page}: not enlarged. {reason}")
}

/// A page reached while another part of the program held the document.
#[must_use]
pub fn busy(page: usize) -> String {
    format!("Page {page}: not enlarged. The document was busy; run it again for this page.")
}

/// A turn that reached a different document than its run's.
#[must_use]
pub fn elsewhere(page: usize) -> String {
    format!("Page {page}: not enlarged. Another document was on screen.")
}

/// The run's headline.
#[must_use]
pub fn done(enlarged: usize, of: usize) -> String {
    format!("Enlarged {enlarged} of {of} page(s).")
}

/// The run was stopped before its last page.
#[must_use]
pub fn stopped(done: usize, of: usize) -> String {
    format!("Stopped after {done} of {of} page(s). What was enlarged is kept.")
}

/// What the run cost in file size.
#[must_use]
pub fn growth(old: usize, new: usize) -> String {
    format!(
        "The enlarged pictures take {} where they took {}.",
        byte_size(new),
        byte_size(old)
    )
}

/// The run's enlargements could not be folded into one undo step.
#[must_use]
pub fn unfolded() -> &'static str {
    "Each enlarged page is its own undo step this time, so undoing the run takes one Ctrl+Z per page."
}
