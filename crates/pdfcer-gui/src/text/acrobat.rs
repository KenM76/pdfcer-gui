//! # `text::acrobat` — every word O122 puts on screen
//!
//! `OPERATOR_REQUESTS.md` **O122**: the *Open in Acrobat* control beside the
//! Read / Review / Edit selector, the three things it can say before it acts,
//! and the Settings field that says where Acrobat is.
//!
//! ## Why one module for four surfaces
//!
//! The ribbon command, three dialogs and a Settings group would normally be
//! four places in this catalog. They are one here because they are **one
//! conversation**: the button promises something, the dialog says what that
//! costs, and the Settings field is where the promise comes from. A wording
//! change to any one of them is nearly always a wording change to another, and
//! catalog entries that must move together are entries that should be read
//! together.
//!
//! ## ★★★ The sentence this whole module is written around
//!
//! > **The file will be closed.**
//!
//! That is the operator's own instruction — O122 point 6, *"it will note the
//! file will be closed when opened in acrobat"* — and it is the fact every
//! string here has to carry without burying. Acrobat takes its own lock on the
//! file it opens; two editors on one PDF is how an afternoon's work
//! disappears; so pdfcer gives the file up rather than keeping it open
//! alongside. The operator is told **before** it happens, because closing
//! somebody's document is not a thing to do quietly.
//!
//! ## ★★ The button that is not offered, and the reason it is not
//!
//! There is no *"open without saving"*. [`crate::dialogs::unsaved`] offers
//! exactly that shape — *Save · Don't save · Cancel* — and is right to, because
//! there the document is merely being closed. Here it is being closed **and
//! handed to a program that will happily save over it**, so the discarded
//! edits would not simply be lost: they would be lost, and then overwritten by
//! a version of the file that never had them, in a program the operator
//! believes is showing them their work.
//!
//! So the wording never implies a third answer exists. [`save_first_body`]
//! says what will be saved; it does not ask whether to save.
//!
//! ## Vocabulary
//!
//! - **Acrobat**, never *"Adobe Acrobat"* in a button and never *"the external
//!   viewer"*. It is a proper name the operator uses, and a generic phrase in
//!   its place reads as a program that is not sure what it found.
//! - **The document**, not *"the PDF"* — the same choice the rest of this
//!   catalog makes.
//! - The edition is named **only where it is useful**: on hover, where there
//!   is room to say *Acrobat Pro* or *Acrobat Reader* and it tells the
//!   operator which of their two installations is about to open. The button
//!   itself says *Acrobat*, because a label that changed between machines is a
//!   label nobody can be told to press.

use super::commands::CommandText;
use crate::acrobat::{Edition, Source, Viewer};

// ---------------------------------------------------------------------------
// The ribbon control
// ---------------------------------------------------------------------------

/// `file.open_in_acrobat` — the control beside the mode selector.
///
/// ★ **No ellipsis**, deliberately, and it is a close call. This crate's
/// convention is that an ellipsis means *"activating this opens a dialog
/// rather than acting"*, and this control always raises a dialog. But the
/// dialog is a **confirmation of the thing the label names**, not a request
/// for information the label left out — the difference between *Settings…*,
/// which cannot act until you tell it what to change, and a Close button that
/// asks whether you meant it. An ellipsis here would suggest there is more to
/// decide than *yes* or *no*.
///
/// ★★ The tooltip states the closing up front rather than at the end. It is
/// the surprising half, and a hover sentence is read from the left until it
/// stops being interesting.
#[must_use]
pub const fn file_open_in_acrobat() -> CommandText {
    CommandText::new(
        "Acrobat",
        "Close this document here and open it in Acrobat. Only one program can safely have a PDF \
         open at a time, so pdfcer gives the file up rather than holding it while Acrobat edits \
         it. You are asked first, and if you have unsaved changes you are offered the chance to \
         save them.",
    )
}

/// The tooltip when the control is there but not usable — no document open.
///
/// R9's second half: *greying is reserved for temporarily unavailable and is
/// **always explained on hover***. "No document" is exactly that, and this is
/// the explanation.
#[must_use]
pub fn no_document_tooltip() -> String {
    "There is no document open to send to Acrobat.".to_owned()
}

// ---------------------------------------------------------------------------
// The three dialogs
// ---------------------------------------------------------------------------

/// The window title, shared by all three.
///
/// ★ One title rather than three. The window is the same window answering the
/// same request; changing its name according to which sentence is inside it
/// would make a taskbar entry that renames itself while the operator is
/// looking away.
#[must_use]
pub fn title() -> String {
    "Open in Acrobat".to_owned()
}

/// The heading of the clean case — O122 point 6.
#[must_use]
pub fn confirm_close_heading() -> String {
    "This document will be closed.".to_owned()
}

/// The body of the clean case.
///
/// ★ It says **why**, in one clause, and the why is the part that makes the
/// behaviour reasonable rather than officious. An operator told only *"it will
/// be closed"* reads a program being awkward; told *"because two programs
/// editing one file is how work gets lost"*, they read a program looking after
/// their drawing.
#[must_use]
pub fn confirm_close_body(viewer: &Viewer) -> String {
    format!(
        "pdfcer will close it and hand the file to {}. Only one program can safely have a PDF \
         open at a time — two of them writing to one file is how a day's work disappears — so \
         pdfcer gives it up rather than holding on.",
        edition_name(viewer.edition)
    )
}

/// The heading of the unsaved case — O122 point 5, *"forms filled out for
/// example"*.
#[must_use]
pub fn save_first_heading() -> String {
    "Save your changes first?".to_owned()
}

/// The body of the unsaved case.
///
/// ★★★ It names the number of edits and it names the **file**, because those
/// are the two things that make the choice concrete. And it closes on the
/// consequence of *not* saving, phrased as a fact rather than as an option:
/// there is no button for that outcome and the sentence must not read as if
/// there were.
#[must_use]
pub fn save_first_body(edits: u64, file: &str, viewer: &Viewer) -> String {
    let changes = if edits == 1 {
        "1 change".to_owned()
    } else {
        format!("{edits} changes")
    };
    format!(
        "You have {changes} that are not yet in {file}. pdfcer will save them, close the document \
         and hand the file to {}. Without saving, Acrobat would open the file as it was before \
         you started — and anything you saved from there would overwrite your work.",
        edition_name(viewer.edition)
    )
}

/// The heading of the never-saved refusal.
///
/// ★ A statement, not a question. There is nothing to decide: this is the one
/// of the three that offers no way forward, and a heading shaped like a
/// question would promise one.
#[must_use]
pub fn no_file_heading() -> String {
    "This document has never been saved.".to_owned()
}

/// The body of the never-saved refusal.
///
/// ★★ It says what to do — *save it somewhere first* — because a refusal that
/// only refuses leaves the operator to guess, and the guess most people make
/// is that the button is broken. And it is careful **not** to say Acrobat is
/// missing: that is a different refusal with a different remedy, and confusing
/// the two sends somebody looking for an installer they already have.
#[must_use]
pub fn no_file_body() -> String {
    "Acrobat opens files on disk, and there is no file yet — this document exists only inside \
     pdfcer. Save it somewhere first, then it can be opened in Acrobat."
        .to_owned()
}

/// The button that saves and then hands over.
///
/// ★ Named for **what it does**, never *Yes* or *OK*, which is this crate's
/// standing rule for a button whose press has consequences: an operator who
/// reads only the buttons — which is most operators, most of the time — must
/// still get it right.
#[must_use]
pub fn save_and_open_button() -> String {
    "Save and open in Acrobat".to_owned()
}

/// The button that goes ahead with a clean document.
///
/// ★ Also named for what it does, and *not* "OK" — even though the operator's
/// own words were *"with and ok button to continue"*. What he asked for is a
/// confirm-and-proceed control, which this is; what "OK" would cost is the
/// sentence that says the document is closing being the only place that fact
/// appears, and a dismissed dialog is one nobody read.
#[must_use]
pub fn close_and_open_button() -> String {
    "Close and open in Acrobat".to_owned()
}

/// Cancel — on all three, and it is the ✕ as well.
#[must_use]
pub fn cancel_button() -> String {
    "Cancel".to_owned()
}

/// The only button on the never-saved refusal.
#[must_use]
pub fn dismiss_button() -> String {
    "Close".to_owned()
}

/// The status line after the document has been handed over.
#[must_use]
pub fn handed_over(viewer: &Viewer) -> String {
    format!(
        "Closed here and opened in {}.",
        edition_name(viewer.edition)
    )
}

/// ★★ The status line when the launch failed **after** the save succeeded.
///
/// A real state and the one worth wording most carefully: the operator's work
/// is safe on disk, and the only thing that did not happen is the handover.
/// Saying that plainly is the difference between a nuisance and a panic.
#[must_use]
pub fn launch_failed(detail: &str) -> String {
    format!(
        "Your document was saved, and is still open here. Acrobat would not start: {detail}. The \
         path pdfcer is using is in Settings, under Acrobat."
    )
}

/// The decline when the save the operator asked for did not happen.
///
/// ★ Nothing else follows a failed save. The document is not closed and
/// Acrobat is not started, because the whole point of the question was that
/// the edits must survive the handover.
#[must_use]
pub fn save_failed() -> String {
    "Your changes were not saved, so the document has not been closed or sent to Acrobat."
        .to_owned()
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

/// The Settings group heading.
#[must_use]
pub fn group_acrobat() -> String {
    "Acrobat".to_owned()
}

/// The section title.
#[must_use]
pub fn path_title() -> &'static str {
    "Where Acrobat is"
}

/// What this setting silences — the `widgets::header` convention.
#[must_use]
pub fn path_silence() -> &'static str {
    "Nothing. Leaving it blank lets pdfcer find Acrobat itself."
}

/// How far this setting reaches — the `widgets::header` convention.
#[must_use]
pub fn path_radius() -> &'static str {
    "The Acrobat button beside Read / Review / Edit, and nothing else. It never changes what \
     pdfcer writes into a document."
}

/// The field's label.
#[must_use]
pub fn path_label() -> String {
    "Acrobat program".to_owned()
}

/// ★★★ The note under the field, which is the escape hatch's instructions.
///
/// This is the string O122's decision hangs on: *"the path control lives in
/// Settings and is visible there whether or not discovery succeeded, so a
/// non-standard install is fixable without the button ever having appeared."*
/// A person whose Acrobat is somewhere unusual arrives at this field having
/// seen **no button at all**, so the note has to explain a control they have
/// never met.
#[must_use]
pub fn path_note() -> String {
    "Leave this blank and pdfcer asks Windows where Acrobat is, preferring Pro over Reader. Fill \
     it in to point at a particular installation — a portable copy, a second version, or one \
     Windows has not been told about. If nothing is found and nothing is set here, the Acrobat \
     button is simply not shown."
        .to_owned()
}

/// The Browse button.
#[must_use]
pub fn path_browse() -> String {
    "Browse…".to_owned()
}

/// The Browse button's hover.
#[must_use]
pub fn path_browse_hover() -> String {
    "Pick the Acrobat program file — usually Acrobat.exe for Pro, or AcroRd32.exe for Reader."
        .to_owned()
}

/// The file picker's own title.
#[must_use]
pub fn path_dialog_title() -> String {
    "Choose the Acrobat program".to_owned()
}

/// The picker's filter name, for the row that offers programs.
#[must_use]
pub fn path_filter_name() -> String {
    "Programs".to_owned()
}

/// ★★★ **What discovery actually resolved**, shown under the field.
///
/// The half of the escape hatch that makes it usable rather than merely
/// present. Without this line a person who typed a path with a letter missing
/// sees exactly what a person who typed it correctly sees — a filled-in field
/// and no button — and has no way to tell the two apart. With it, the mistake
/// is visible at the place it was made.
#[must_use]
pub fn resolved_note(viewer: Option<&Viewer>) -> String {
    match viewer {
        Some(v) => format!(
            "Now using {} at {}{}.",
            edition_name(v.edition),
            v.path.display(),
            match v.source {
                Source::Configured => ", the path set above",
                Source::AppPaths => ", found through Windows' own registration",
                Source::PdfHandler => ", found as the program registered to open PDFs",
            }
        ),
        None => "No Acrobat found. The Acrobat button is not being shown. If it is installed \
                 somewhere unusual, give its full path above."
            .to_owned(),
    }
}

/// The product name for an edition, as it appears in prose.
///
/// ★ Not `Debug`, and not a bare "Acrobat": which of the two is installed is a
/// thing the operator knows about their own machine, and naming it is how they
/// confirm that pdfcer found the one they meant. Somebody with both installed
/// who sees *Acrobat Reader* here has been told about a misconfiguration they
/// could not otherwise have detected.
#[must_use]
pub const fn edition_name(edition: Edition) -> &'static str {
    match edition {
        Edition::Pro => "Acrobat Pro",
        Edition::Reader => "Acrobat Reader",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn viewer(edition: Edition) -> Viewer {
        Viewer {
            path: PathBuf::from(r"C:\Program Files\Adobe\Acrobat DC\Acrobat\Acrobat.exe"),
            edition,
            source: Source::AppPaths,
        }
    }

    /// **★★★ Every sentence the operator can be shown before the document is
    /// handed over says that it will be closed.**
    ///
    /// O122 point 6 in checkable form. The two dialogs that lead to a handover
    /// must both carry the fact; the third must not, because nothing is being
    /// closed there and saying so would be false.
    #[test]
    fn both_confirmations_say_the_document_will_be_closed() {
        let pro = viewer(Edition::Pro);
        let clean = format!("{} {}", confirm_close_heading(), confirm_close_body(&pro));
        let dirty = format!(
            "{} {}",
            save_first_heading(),
            save_first_body(3, "sheet 1.pdf", &pro)
        );
        for (which, text) in [("clean", &clean), ("unsaved", &dirty)] {
            assert!(
                text.contains("close") || text.contains("closed"),
                "the {which} confirmation never says the document is closed: {text}"
            );
        }

        // ★★ AND THE CLEAN SHAPE SAYS IT IN THE **HEADING**, not only
        // somewhere in the paragraph.
        //
        // Added after a falsification found the gap: replacing the heading
        // with *"Ready when you are."* left the assertion above passing,
        // because it reads heading and body joined and the body still
        // mentioned closing. O122 point 6 is that the dialog **notes the file
        // will be closed**, and a note buried in the third clause of a
        // paragraph is a note nobody reads — the heading is the only line an
        // operator is guaranteed to see.
        assert!(
            confirm_close_heading().contains("closed"),
            "the clean confirmation's HEADING must carry the fact, not just its body: {}",
            confirm_close_heading()
        );
        assert!(
            !no_file_body().contains("closed"),
            "nothing is closed in the never-saved case, so nothing should say so"
        );
    }

    /// **★★★ No sentence and no button offers to open without saving.**
    ///
    /// The module header's argument, mechanised. A future edit that adds a
    /// *Don't save* button to this dialog — reasonably, by analogy with
    /// [`crate::dialogs::unsaved`] — has to delete this test first, which is
    /// where they will read why it is there.
    #[test]
    fn nothing_offers_to_open_without_saving() {
        let pro = viewer(Edition::Pro);
        let buttons = [
            save_and_open_button(),
            close_and_open_button(),
            cancel_button(),
            dismiss_button(),
        ];
        for button in &buttons {
            let lower = button.to_lowercase();
            assert!(
                !lower.contains("without saving") && !lower.contains("discard"),
                "a button offers to discard the operator's edits: {button}"
            );
        }
        assert!(
            save_and_open_button().to_lowercase().contains("save"),
            "the only way forward from unsaved edits must name saving"
        );
        // The body says what happens WITHOUT saving as a warning, and must
        // not read as an available choice.
        let body = save_first_body(2, "sheet 1.pdf", &pro);
        assert!(
            body.contains("overwrite"),
            "the reason there is no third button has to be in the sentence: {body}"
        );
    }

    /// **★★ The three refusals are three different sentences.**
    ///
    /// The one that matters is that *"no file on disk"* does not read as
    /// *"Acrobat is missing"*: they have different remedies, and an operator
    /// sent to look for an installer they already have will conclude the
    /// feature does not work.
    #[test]
    fn a_missing_file_does_not_read_as_a_missing_acrobat() {
        let missing_file = format!("{} {}", no_file_heading(), no_file_body());
        let missing_acrobat = resolved_note(None);
        assert_ne!(missing_file, missing_acrobat);
        assert!(
            !missing_file.contains("not found") && !missing_file.contains("No Acrobat"),
            "the never-saved refusal must not sound like a missing Acrobat: {missing_file}"
        );
        assert!(
            missing_acrobat.contains("No Acrobat"),
            "the missing-Acrobat note must say so plainly: {missing_acrobat}"
        );
        assert!(
            missing_file.contains("Save it"),
            "a refusal that does not say what to do next reads as a broken button"
        );
    }

    /// The resolved note names the edition, the path and where it came from —
    /// the three facts a person debugging a wrong setting needs.
    #[test]
    fn the_resolved_note_names_edition_path_and_source() {
        let note = resolved_note(Some(&viewer(Edition::Reader)));
        assert!(note.contains("Acrobat Reader"), "{note}");
        assert!(note.contains("Acrobat.exe"), "{note}");
        assert!(note.contains("registration"), "{note}");

        let configured = resolved_note(Some(&Viewer {
            source: Source::Configured,
            ..viewer(Edition::Pro)
        }));
        assert!(
            configured.contains("set above"),
            "a configured path must be identifiable as the operator's own: {configured}"
        );
    }

    /// One change is "1 change", not "1 changes".
    #[test]
    fn the_edit_count_is_worded_for_one_as_well_as_for_many() {
        let pro = viewer(Edition::Pro);
        assert!(save_first_body(1, "a.pdf", &pro).contains("1 change that"));
        assert!(save_first_body(7, "a.pdf", &pro).contains("7 changes that"));
    }
}
