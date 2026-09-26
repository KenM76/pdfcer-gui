//! **The print window's three ways out** — `OPERATOR_REQUESTS.md` **O185**.
//!
//!
//! # What the three words have to do between them
//!
//! | string | promise |
//! |---|---|
//! | [`commit`] (in [`super`]) | print, and keep these settings |
//! | [`keep_and_close`] | keep these settings, print nothing |
//! | [`cancel`] | put the settings back to what they were |
//!
//! Two of those three are new sentences rather than new behaviour: printing
//! has kept the settings since O166. Nothing had to *say so* until there was a
//! button beside it that only keeps, at which point an operator choosing
//! between them needs each to disclose the half the other does not.
//!
//! # Why the hovers exist at all, given R9 and this project's dislike of chrome
//!
//! Because the difference between these three buttons is entirely in what
//! happens to state the operator cannot see. A label can say *Cancel*; it
//! cannot say *and the copy count goes back to one*. Rule 4 is **fuzzy, never
//! sneaky**, and its surviving half is the one that binds here: an inference or
//! an effect the operator cannot observe still owes a disclosure, off-canvas.
//! A tooltip on the control that causes it is as close to the act as
//! off-canvas gets.
//!
//! Re-exported wholesale by [`super`], so every caller keeps writing
//! `t::cancel()` and this split is invisible at the call sites.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/print/footer.md`.

/// Leave without printing, **and put the settings back**.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

/// What [`cancel`] does to the settings, in the operator's own terms.
#[must_use]
pub const fn cancel_hover() -> &'static str {
    "Puts the print settings back to what they were when this window opened. Escape and the window's close button do the same."
}

/// Keep the settings for next time, without printing.
#[must_use]
pub const fn keep_and_close() -> &'static str {
    "Keep and close"
}

/// What [`keep_and_close`] does, in the operator's own terms.
#[must_use]
pub const fn keep_and_close_hover() -> &'static str {
    "Keeps these settings for next time without printing."
}

/// What pressing Print does to the settings, beyond printing.
#[must_use]
pub const fn commit_hover() -> &'static str {
    "Prints, and keeps these settings for next time."
}

/// Send the job. The plain label, when nothing will be clipped.
#[must_use]
pub const fn commit() -> &'static str {
    "Print"
}

/// Send the job, **with the clip count in the button's own label**.
#[must_use]
pub fn commit_with_clipping(clipped: usize) -> String {
    if clipped == 1 {
        "Print — 1 sheet will be clipped".to_owned()
    } else {
        format!("Print — {clipped} sheets will be clipped")
    }
}

/// Send the job, **naming a count that has actually been measured** —
/// operator request O113, 2026-09-04.
#[must_use]
pub fn commit_losing_content(losing: usize) -> String {
    if losing == 1 {
        "Print — 1 sheet will lose content".to_owned()
    } else {
        format!("Print — {losing} sheets will lose content")
    }
}

/// Send the job, **naming a ceiling** — operator request O113, 2026-09-04.
#[must_use]
pub fn commit_may_lose_content(at_most: usize) -> String {
    if at_most == 1 {
        "Print — 1 sheet may lose content".to_owned()
    } else {
        format!("Print — up to {at_most} sheets may lose content")
    }
}

/// Confirmation that the job went out.
#[must_use]
pub fn sent(pages: usize) -> String {
    if pages == 1 {
        "Sent 1 page to the printer.".to_owned()
    } else {
        format!("Sent {pages} pages to the printer.")
    }
}

/// The job did not go out, and why.
#[must_use]
pub fn failed(detail: &str) -> String {
    format!("Nothing was sent to the printer. {detail}")
}

/// **The driver would not report its settings, so the job carried only what
/// pdfcer sets itself.**
#[must_use]
pub const fn settings_synthesised() -> &'static str {
    "This printer would not report its current settings, so the job was sent with only the settings shown here — media type, quality and finishing fell back to the driver's own. Open Properties… before printing again to send them."
}

/// Why the Custom percentage field is greyed —
/// `OPERATOR_REQUESTS.md` O77's sweep.
#[must_use]
pub fn scale_custom_disabled() -> &'static str {
    "Choose Custom on the radio beside this field to set your own percentage."
}
