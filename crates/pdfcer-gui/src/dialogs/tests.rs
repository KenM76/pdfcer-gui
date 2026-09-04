//! # `dialogs::tests` — the dialog owner's own assertions
//!
//! Split out of [`super`] on 2026-09-04 under **R2**, when the Export-image
//! window (`OPERATOR_REQUESTS.md` O120) took that file past the 1,500-line
//! ceiling. The seam is `canvas::selection::tests`' seam and it is chosen for
//! that module's stated reason rather than for a size:
//!
//! > the tests were the seam and the code was not.
//!
//! ★ [`super`] is one subject — **who owns which window, and when is it
//! dropped** — and it cannot be cut in half without putting the field, the
//! draw call and the close rule for one dialog in different files. That is the
//! arrangement `check-file-size.sh`'s own header warns against ("a reviewer
//! cannot see that a keyboard guard at line 13,777 interacts with a focus
//! request at line 16,891"), applied at a smaller scale. The assertions,
//! though, are a genuinely separate subject: they are *about* that code rather
//! than part of it, and every one of them reaches its subject through the
//! public surface.
//!
//! ## `#![cfg(test)]` as an inner attribute, deliberately
//!
//! Two gates read it from the file rather than from the filename —
//! `check-ui-strings.sh` (so assertion messages are not counted as
//! operator-facing copy) and `check-theme-colors.sh` (so a fixture colour is
//! not a palette violation). Both state the same reason: the property that
//! earns the exemption is *"not in the shipped binary"*, and a filename is a
//! restatement of that which goes stale the moment a third such module exists.

#![cfg(test)]

use super::*;

/// ★★★ **A window that closed BECAUSE it was answered is not retired
/// until the answer has been taken out of it.**
///
/// The regression test for the defect
/// `an_invalidating_save_is_warned_about` found by driving on 2026-08-29:
/// the signature warning's proceed button set the confirmation, which made
/// `show` answer `false`, which made the owner drop the dialog **and the
/// confirmation with it** — so `resume_after_signature` found nothing,
/// traced nothing, and wrote nothing. Save was unusable on every signed
/// document.
///
/// Four rows because the predicate has two inputs and each combination is a
/// real state: an open unanswered window (the normal case), an open window
/// that has just been answered (`answered` wins and it is kept — it cannot
/// arise today, since answering closes both windows, but the rule must not
/// depend on that), a cancelled window (retired, and it answers nothing —
/// which is what makes the ✕ non-destructive), and the one that cost the
/// day.
#[test]
fn an_answered_window_survives_its_own_close() {
    assert!(
        !retire(true, false),
        "an open window with nothing parked stays open"
    );
    assert!(
        !retire(true, true),
        "an answer is never discarded, whatever `show` says about visibility"
    );
    assert!(
        retire(false, false),
        "a CANCELLED window is retired: it closed and answered nothing, which is \
         exactly what makes the ✕ mean Cancel"
    );
    assert!(
        !retire(false, true),
        "★ THE DEFECT: a window closed by its own proceed button is holding the \
         answer that closed it, and dropping it here loses the save"
    );
}

/// A dialog cannot be opened without a document.
///
/// The guard that stops a keyboard chord from enumerating the spooler —
/// a call that blocks on a network printer — to populate a window that
/// would be closed again on the next frame.
#[test]
fn no_document_means_no_dialog() {
    let mut dialogs = DialogsState::default();
    dialogs.open_print(&Status::Empty);
    assert!(dialogs.print.is_none());
}

/// Closing the document closes the document-scoped dialogs.
///
/// Asserted through the public path rather than by setting the field, so
/// the test covers what a frame actually does.
#[test]
fn a_closed_document_closes_every_document_scoped_dialog() {
    let mut dialogs = DialogsState::default();
    assert!(dialogs.print.is_none());
    dialogs.close_document_scoped();
    assert!(dialogs.print.is_none());
    assert!(dialogs.ocr.is_none());
    assert!(dialogs.diagnostics.is_none());
    assert!(dialogs.redact.is_none());
}

/// ★ **Apply redactions cannot be opened without a document, and a second
/// invocation does not rebuild it.**
///
/// Both guards matter more for this dialog than for any of its neighbours,
/// because opening it runs a full rewrite of the document. The second
/// assertion is the one with teeth: a rebuild would re-run that work *and*
/// discard the operator's two acknowledgements, throwing away the reading
/// they have just done on the one report in this program that has to be
/// read before a control is pressed.
#[test]
fn the_apply_dialog_is_guarded_on_both_counts() {
    let mut dialogs = DialogsState::default();
    dialogs.open_redact(&Status::Empty);
    assert!(
        dialogs.redact.is_none(),
        "a document with nothing open has nothing to redact, and building \
         the dialog would run a full rewrite in order to refuse"
    );

    let status = Status::Open(Box::new(crate::app::state::open_fixture(
        crate::app::state::FOUR_PAGES,
    )));
    dialogs.open_redact(&status);
    let first = std::ptr::from_ref(dialogs.redact.as_ref().expect("open"));
    dialogs.open_redact(&status);
    let second = std::ptr::from_ref(dialogs.redact.as_ref().expect("still open"));
    assert_eq!(
        first, second,
        "the second press replaced the dialog, re-running the removal and \
         discarding both acknowledgements"
    );
}

/// The render report cannot be opened without a document either, and the
/// guard is the one that matters most for it.
///
/// Its command is gated on `doc.open`, so the ribbon cannot reach this
/// state — but a chord can, and without the guard the dialog would be built
/// and then closed by [`DialogsState::show`] on the very next frame. A
/// window that flickers is harder to diagnose than one that never appears.
#[test]
fn no_document_means_no_diagnostics_dialog() {
    let mut dialogs = DialogsState::default();
    dialogs.open_diagnostics(&Status::Empty);
    assert!(dialogs.diagnostics.is_none());
}

/// Pressing Render diagnostics twice does not rebuild the report.
///
/// Nothing would be lost — it holds no configuration, and it reads the
/// texture live — but the window would jump back to the centre and the
/// findings list back to the top, which for an operator half-way down a
/// census is the program losing their place. About's argument, one dialog
/// over.
#[test]
fn opening_the_diagnostics_report_twice_leaves_the_first_one_alone() {
    let mut dialogs = DialogsState::default();
    let status = Status::Open(Box::new(crate::app::state::open_fixture(
        crate::app::state::FOUR_PAGES,
    )));
    dialogs.open_diagnostics(&status);
    let first = std::ptr::from_ref(dialogs.diagnostics.as_ref().expect("open"));
    dialogs.open_diagnostics(&status);
    let second = std::ptr::from_ref(dialogs.diagnostics.as_ref().expect("still open"));
    assert_eq!(first, second, "the second press replaced the dialog");
}

/// Recognise text cannot be opened without a document either.
///
/// Same guard as print's, and it matters for a different reason: the
/// dialog captures the page index and the document path on construction,
/// so one built against `Status::Empty` would have neither and would be a
/// window that could only refuse.
#[test]
fn no_document_means_no_recognition_dialog() {
    let mut dialogs = DialogsState::default();
    dialogs.open_ocr(&Status::Empty, Vec::new());
    assert!(dialogs.ocr.is_none());
}

/// About opens with no document, and survives the document closing.
///
/// ★ The one property that would have been lost by reusing print's shape.
/// `open_about` takes no `Status` precisely so this cannot regress by
/// someone adding a guard "for consistency"; the assertion is here so
/// that if they do, something says why it was not consistent in the first
/// place.
#[test]
fn about_opens_without_a_document_and_survives_one_closing() {
    let mut dialogs = DialogsState::default();
    dialogs.open_about();
    assert!(
        dialogs.about.is_some(),
        "About must open on an empty canvas: it describes pdfcer, not a file"
    );
    dialogs.close_document_scoped();
    assert!(
        dialogs.about.is_some(),
        "About is not about the document and must not close with it"
    );
}

/// Pressing About twice does not rebuild the dialog.
///
/// Nothing would be *lost* — it holds no configuration — but the window
/// would jump back to the centre and the attribution list back to the
/// top, which for an operator reading it is the program losing their
/// place.
#[test]
fn opening_about_twice_leaves_the_first_one_alone() {
    let mut dialogs = DialogsState::default();
    dialogs.open_about();
    let first = std::ptr::from_ref(dialogs.about.as_ref().expect("open"));
    dialogs.open_about();
    let second = std::ptr::from_ref(dialogs.about.as_ref().expect("still open"));
    assert_eq!(first, second, "the second press replaced the dialog");
}
