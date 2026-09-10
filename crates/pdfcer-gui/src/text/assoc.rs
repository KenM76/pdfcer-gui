//! # `text::assoc` — every word O173 puts on screen
//!
//! `OPERATOR_REQUESTS.md` **O173**: *"we should have an easy way to make
//! pdfce-gui our default opener for pdfs. Ask once with a don't show me again (old-name-exempt: HIS words, quoted verbatim from O173 — correcting an operator's own sentence would stop this being a quotation)
//! check box option. Then it should be in the top of our settings as a button
//! to execute the changeover."*
//!
//! Three surfaces, one conversation, so one module: the **ask-once dialog**
//! that offers it, the **Settings group** that is where the offer lives
//! afterwards, and the **state line** that says what Windows actually thinks.
//! Plus the two registry strings Windows itself displays, which belong here for
//! the same reason every other displayed string does — they are read by the
//! operator, in the *Open with* menu and in Windows Settings, and a catalog
//! that held every string except the two with the widest audience would be a
//! catalog with a hole in it.
//!
//! ## ★★★ The sentence this module is written around
//!
//! > **Windows will ask you to confirm.**
//!
//! Every word here is shaped by a platform fact: **no program can make itself
//! the default PDF viewer on its own.** Windows 10 and 11 store the choice
//! behind a hash only Explorer can compute, and anything written there by an
//! application is detected and discarded. See [`crate::app::assoc`] for the
//! mechanism and the citation.
//!
//! So the button cannot promise what its name suggests, and the wording has to
//! be honest about a two-step act without turning into an essay about the
//! registry. The shape chosen: **say what pdfcer does, then say what is left
//! for the operator, in that order, in two short sentences.** The button label
//! ends in an ellipsis for the same reason every other label that opens
//! something does — it is a promise that another surface is coming.
//!
//! ⇒ What is deliberately NOT written here: any string claiming the default was
//! changed. [`is_default`](crate::app::assoc::is_default) is the only thing
//! allowed to make that claim, and it makes it by reading what Windows says
//! rather than by remembering what pdfcer did.
//!
//! ## Vocabulary
//!
//! - **Windows**, named outright. This is one of the few places where the
//!   operating system is a participant in the conversation rather than the
//!   floor it happens on, and *"the system"* would read as pdfcer being coy
//!   about which program it is talking about.
//! - **Open PDFs with pdfcer**, not *"file association"*, not *"default
//!   handler"*, not *"ProgID"*. The operator's question is *"does
//!   double-clicking a drawing open this?"* and the words should be that
//!   question's words.
//! - **Ask again**, not *"show this again"*, in the checkbox — the thing being
//!   suppressed is a question, not a notification.

/// The Settings group's heading, and the ask-once dialog's title.
///
/// ★ Deliberately identical. The dialog is the offer and the group is where the
/// offer lives afterwards; an operator who ticks *"don't ask again"* and then
/// changes their mind is looking for the words they dismissed, and finding a
/// differently-named group is how a feature is concluded not to exist.
#[must_use]
pub fn title() -> String {
    "Open PDFs with pdfcer".to_owned()
}

/// What the offer means, on both surfaces.
///
/// Two sentences: what pdfcer does, then what is left for the operator. The
/// second is not a caveat tucked underneath — it is half the act, and an
/// operator who is not expecting a Windows dialog will read that dialog as
/// something going wrong.
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
///
/// ★ *"Not now"* rather than *"Cancel"*: cancelling implies the offer is
/// withdrawn, and it is not — it is in Settings, permanently, which is the
/// whole point of the operator's *"then it should be in the top of our
/// settings"*.
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
///
/// ★ The ProgID is shown raw. It is not a friendly name and there is no
/// reliable way to turn one into a friendly name — `AppXd4nrz…` is what Windows
/// stores for Edge — but it is *stable and searchable*, and an operator who
/// wants to know what has the association can paste it somewhere. A prettier
/// string that guessed would eventually name the wrong program.
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
///
/// ⚠ The state worth calling out, because it is the one that looks like it
/// works and does not: a portable build gets unzipped somewhere new, and the
/// registration still names the old folder. Double-clicking then opens a build
/// the operator thought they had replaced — or nothing, if the old folder is
/// gone.
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
///
/// ★ Rare enough to be surprising and real enough to need a sentence:
/// `current_exe` is documented as able to fail. Saying so plainly beats a
/// button that does nothing.
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
///
/// ⇒ The instruction is the fallback: this names the route through the Settings
/// app, because an operator who cannot reach the page by link can still reach
/// it by hand, and a refusal with no way forward is just an apology.
#[must_use]
pub fn settings_page_refused() -> String {
    // A plain `>` rather than a typographic separator. `icons::glyphs`
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
