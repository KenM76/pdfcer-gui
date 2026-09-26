//! # `text::assoc` — every word O173 puts on screen
//!
//! `OPERATOR_REQUESTS.md` **O173**: *"we should have an easy way to make
//! pdfce-gui our default opener for pdfs. Ask once with a don't show me again (old-name-exempt: HIS words, quoted verbatim from O173 — correcting an operator's own sentence would stop this being a quotation)
//! check box option. Then it should be in the top of our settings as a button
//! to execute the changeover."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/assoc.md`.

/// The Settings group's heading, and the ask-once dialog's title.
#[must_use]
pub fn title() -> String {
    "Open PDFs with pdfcer".to_owned()
}

/// What the offer means, on both surfaces.
#[must_use]
pub fn body() -> String {
    "pdfcer can add itself to the list of programs Windows offers for PDF \
     files. Windows will then ask you to confirm the change in its own \
     settings — only you can make it, not pdfcer."
        .to_owned()
}

/// The button that does it, on both surfaces.
#[must_use]
pub fn action() -> String {
    "Make pdfcer the default…".to_owned()
}

/// The hover on that button.
#[must_use]
pub fn action_hover() -> String {
    "Adds pdfcer to Windows' list of PDF programs, then opens the Windows page \
     where you confirm it. Nothing outside your own user account is changed."
        .to_owned()
}

/// The ask-once dialog's dismissal.
#[must_use]
pub fn later() -> String {
    "Not now".to_owned()
}

/// The checkbox O173 asks for by name.
#[must_use]
pub fn dont_ask() -> String {
    "Don't ask me again".to_owned()
}

/// The hover on that checkbox, which is where the promise that nothing is lost
/// gets made.
#[must_use]
pub fn dont_ask_hover() -> String {
    "pdfcer will not offer this on startup again. The button stays at the top \
     of Settings."
        .to_owned()
}

// ---------------------------------------------------------------------------
// The state line — what Windows actually says, off-canvas, after the fact
// ---------------------------------------------------------------------------

/// **Windows opens PDFs with pdfcer.** The one string permitted to say so, and
/// it is only ever produced from a live reading of what Windows recorded.
#[must_use]
pub fn state_default() -> String {
    "Windows currently opens PDF files with pdfcer.".to_owned()
}

/// **Windows opens PDFs with something else, and here is its name.**
#[must_use]
pub fn state_other(owner: &str) -> String {
    format!("Windows currently opens PDF files with another program ({owner}).")
}

/// **Windows has no per-user choice recorded**, which is the ordinary state on
/// a machine where nobody has ever picked — not a fault, and not something to
/// dress up as one.
#[must_use]
pub fn state_unset() -> String {
    "Windows has no PDF program chosen for your account yet.".to_owned()
}

/// **pdfcer is registered as a candidate**, which is the half this program can
/// do and has done.
#[must_use]
pub fn state_registered() -> String {
    "pdfcer is in Windows' list of PDF programs.".to_owned()
}

/// **pdfcer is not in the list yet** — the state every portable build starts
/// in, because no installer ever ran.
#[must_use]
pub fn state_unregistered() -> String {
    "pdfcer is not in Windows' list of PDF programs yet.".to_owned()
}

/// **The list points at a different copy of pdfcer.**
#[must_use]
pub fn state_registered_elsewhere() -> String {
    "Windows' list points at a different copy of pdfcer. Use the button to \
     point it at this one."
        .to_owned()
}

/// What the group says immediately after the button was pressed and the
/// Windows page was opened.
#[must_use]
pub fn handed_over() -> String {
    "pdfcer was added to Windows' list, and the Windows settings page is open. \
     Choose pdfcer there to finish."
        .to_owned()
}

// ---------------------------------------------------------------------------
// Strings Windows itself displays
// ---------------------------------------------------------------------------

/// The ProgID's own display name — what the *Open with* menu shows.
#[must_use]
pub fn progid_description() -> String {
    "PDF document (pdfcer)".to_owned()
}

/// pdfcer's name in Windows' *Default apps* list.
#[must_use]
pub fn application_name() -> String {
    "pdfcer".to_owned()
}

/// The line under that name.
#[must_use]
pub fn application_description() -> String {
    "Read, mark up and edit PDF drawings".to_owned()
}

// ---------------------------------------------------------------------------
// Refusals
// ---------------------------------------------------------------------------

/// pdfcer cannot find its own executable, so there is no path to register.
#[must_use]
pub fn no_exe_path() -> String {
    "pdfcer could not determine where its own program file is, so it cannot \
     register itself with Windows."
        .to_owned()
}

/// A registry write was refused and said nothing about why.
#[must_use]
pub fn refused(key: &str) -> String {
    format!("Windows refused to record pdfcer as a PDF program ({key}).")
}

/// The Windows settings page would not open.
#[must_use]
pub fn settings_page_refused() -> String {
    // A plain `>` rather than a typographic separator. `text::glyphs`
    // measures the shipped font stack and it can draw NEITHER `▸` nor
    // `→` - both were tried here, both would have reached the operator as
    // a substitution box in the middle of the one sentence that exists to
    // tell them where to go by hand. `>` is also what Windows' own
    // breadcrumb uses, so the sentence reads as the path it names.
    "The Windows settings page would not open. You can reach it from Windows \
     Settings > Apps > Default apps, then search for pdfcer."
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The offer never claims the change was made**, on any surface, because
    /// only Windows can make it. See the module header.
    #[test]
    fn nothing_here_promises_the_default_was_changed() {
        for s in [title(), body(), action(), action_hover(), handed_over()] {
            let lower = s.to_lowercase();
            assert!(
                !lower.contains("is now the default") && !lower.contains("has been set"),
                "a surface claimed a change only Windows can make: {s}"
            );
        }
    }

    /// **The body warns that Windows will ask**, which is the fact that stops
    /// the operating system's own dialog reading as a failure.
    #[test]
    fn the_body_warns_that_windows_will_ask() {
        assert!(body().contains("Windows will then ask you to confirm"));
    }

    /// **The state line quotes what Windows said**, so the operator can act on
    /// it rather than on pdfcer's memory of its own button press.
    #[test]
    fn the_other_owner_line_names_the_owner() {
        assert!(state_other("AppXd4nrz").contains("AppXd4nrz"));
    }

    /// **A refusal that cannot open the page still says how to get there.**
    #[test]
    fn the_settings_refusal_carries_the_manual_route() {
        let s = settings_page_refused();
        assert!(s.contains("Default apps"), "no way forward offered: {s}");
    }
}
