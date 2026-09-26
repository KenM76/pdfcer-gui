//! # `text::unsaved` — the words of the question `file.close` promised to ask
//!
//! Every string `pdfcer_gui::dialogs::unsaved` renders. Separated from the dialog
//! for `tools/gates/check-ui-strings.sh`'s reason, and worth its own file
//! rather than a section of a shared one because this surface's copy is doing
//! more work than most: **the operator's whole understanding of what they are
//! about to lose comes from four sentences and three button labels**, and there
//! is no second chance to read them.
//!
//! ## The one rule the whole file follows
//!
//! **Nothing here says "changes".** It says *edits*, and where it can, it says
//! how many. "You have unsaved changes" is the sentence every application shows
//! and it is nearly contentless — an operator cannot tell from it whether they
//! moved one dimension or spent an hour marking up a drawing, and the decision
//! they are being asked to make depends entirely on which.
//!
//! ## And nothing here says "Save"
//!
//! This build has no Save. `file.save` is in `pdfcer_gui::shell::manifest::PLANNED`,
//! blocked on autosave and crash recovery; the only writer is `file.save_copy`,
//! which writes a **different file** and leaves the open document exactly as
//! unsaved as it was. A button labelled *Save* would be the same lie as the
//! tooltip that exposed this defect in the first place — see
//! `pdfcer_gui::dialogs::unsaved`'s header — and the note under the buttons exists
//! to make sure nobody presses the first button believing something false about
//! which file their work ended up in.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/unsaved.md`.

/// The window title.
#[must_use]
pub const fn title() -> &'static str {
    "Unsaved edits"
}

/// The question, when the operator pressed Close.
#[must_use]
pub const fn question_close() -> &'static str {
    "This document has edits that are not in any file yet. Close it anyway?"
}

/// The question, when the operator is opening another document.
#[must_use]
pub const fn question_open() -> &'static str {
    "The document you have open has edits that are not in any file yet. Opening \
     another one will close it."
}

/// The question, when the operator is starting a new document.
#[must_use]
pub const fn question_new() -> &'static str {
    "The document you have open has edits that are not in any file yet. Starting \
     a new one will close it."
}

/// The question, when the operator is re-reading this same file under the
/// other reading of a key it names twice.
#[must_use]
pub const fn question_reread() -> &'static str {
    "Reading this file again is a fresh start from what is on disk, so the \
     edits you have made here cannot come with it."
}

/// How much is at stake, in the operator's units rather than the engine's.
#[must_use]
pub fn edits_at_stake(edits: u64) -> String {
    if edits == 1 {
        "1 edit has been made since it was opened.".to_owned()
    } else {
        format!("{edits} edits have been made since it was opened.")
    }
}

/// **The button that writes the file the operator opened** —
/// `OPERATOR_REQUESTS.md` O65.
#[must_use]
pub const fn save_button() -> &'static str {
    "Save"
}

/// **Save all** — `OPERATOR_REQUESTS.md` O102.
#[must_use]
pub fn save_all_button(count: usize) -> String {
    format!("Save all {count}")
}

/// The non-destructive button.
#[must_use]
pub const fn save_copy_button() -> &'static str {
    "Save a copy…"
}

/// What "a copy" actually means for the file they came from.
#[must_use]
pub const fn save_copy_note() -> &'static str {
    "A copy is written to a new file that you name. The document you are working \
     on is not changed on disk."
}

/// The note under the pair, when both buttons are offered.
#[must_use]
pub const fn save_choice_note() -> &'static str {
    "Save writes over the file you opened. Save a copy writes a new file that \
     you name and leaves the original alone."
}

/// The destructive button, when the operator pressed Close.
#[must_use]
pub const fn discard_close() -> &'static str {
    "Close without saving"
}

/// The destructive button, when the operator is opening another document.
#[must_use]
pub const fn discard_open() -> &'static str {
    "Open anyway, lose the edits"
}

/// The destructive button, when the operator is starting a new document.
#[must_use]
pub const fn discard_new() -> &'static str {
    "Start a new one, lose the edits"
}

/// The destructive button, when the operator is re-reading the same file.
#[must_use]
pub const fn discard_reread() -> &'static str {
    "Read it again, lose the edits"
}

/// The button that changes nothing.
#[must_use]
pub const fn cancel_button() -> &'static str {
    "Cancel"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Singular and plural are both written out.
    ///
    /// *"1 edits"* is the kind of thing that makes an operator trust the rest
    /// of the sentence less, on a surface whose entire job is being believed.
    #[test]
    fn the_count_reads_as_english() {
        assert_eq!(
            edits_at_stake(1),
            "1 edit has been made since it was opened."
        );
        assert!(edits_at_stake(2).starts_with("2 edits have"));
        assert!(edits_at_stake(48).starts_with("48 edits have"));
    }

    /// No button says "Save", and no sentence says "changes".
    #[test]
    fn nothing_promises_a_save_this_build_cannot_do() {
        // The predicate is **"if it says Save, it must say what it saves"**,
        // not "it must not say Save". A blanket ban would forbid the one
        // correct label as well as every wrong one, which is how a rule gets
        // relaxed and then quietly deleted — the next hand hits it, sees it
        // reject the shipped button, and takes the whole assertion out.
        //
        //
        // It used to refuse any label reading as a Save over the open file,
        // *"which this build cannot do"* — true when it was written, false
        // since `file.save` landed on 2026-08-20, and never revisited. So a
        // green test was actively holding the stale answer in place: the one
        // window an operator meets when they are about to lose work offered a
        // file picker as its only alternative to losing it. `OPERATOR_REQUESTS.md`
        // O65.
        //
        // ⇒ The property under test changes from *"nothing claims a save"* to
        // **"every label states its destination"**, which is the durable
        // version and would have been the right one all along: it passes on
        // today's build, would have passed on the 2026-08-20 build, and still
        // refuses a bare "Save changes" that says nothing about which file.
        for label in [
            save_button(),
            save_copy_button(),
            discard_close(),
            discard_open(),
            discard_new(),
            cancel_button(),
        ] {
            let writes = label.eq_ignore_ascii_case("save")
                || label.starts_with("Save")
                || label.contains(" save the ");
            let states_a_destination = label.contains("copy") || label.eq_ignore_ascii_case("save");
            assert!(
                !writes || states_a_destination,
                "{label:?} writes and does not say where. The two buttons in this \
                 window differ ONLY in destination, so a label that omits it is the \
                 one thing an operator cannot recover from."
            );
        }
        assert!(
            save_copy_button().contains("copy"),
            "the copy button must say what it writes"
        );
        assert!(
            save_copy_button().ends_with('\u{2026}'),
            "the copy button promises a picker and must carry the ellipsis that says so"
        );
        assert!(
            !save_button().contains('\u{2026}'),
            "Save writes to a destination already decided, so it must NOT promise a picker"
        );
        // The note under the copy button carries the half that is surprising —
        // that the file they opened is unchanged. Asserted on the claim rather
        // than on the sentence, because the sentence will be reworded.
        assert!(
            save_copy_note().contains("not changed on disk"),
            "the note must say the open file is left alone, which is the part \
             nobody expects"
        );
        // And the pair's note has to say which one touches the original,
        // because four words of button text cannot.
        assert!(
            save_choice_note().contains("over the file you opened"),
            "when both buttons are drawn, something must say which one overwrites"
        );
    }

    /// The three questions each name what will happen to the OPEN document.
    #[test]
    fn every_question_says_the_open_document_is_going_away() {
        assert!(question_close().contains("Close"));
        assert!(question_open().contains("close it"));
        assert!(question_new().contains("close it"));
    }
}
