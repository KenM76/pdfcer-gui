//! # `dialogs::tests` — the dialog owner's own assertions
//!
//!
//! > the tests were the seam and the code was not.
//!
//! [`super`] is one subject — **who owns which window, and when is it
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
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/tests.md`.

#![cfg(test)]

use super::*;

/// **A window that closed BECAUSE it was answered is not retired
/// until the answer has been taken out of it.**
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
#[test]
fn no_document_means_no_dialog() {
    let mut dialogs = DialogsState::default();
    // The remembered settings are irrelevant to what this asserts — the guard
    // fires before they are read — so the shipped defaults are the honest
    // argument here. What matters is that no document means the spooler is
    // never enumerated, whatever the operator last printed with.
    dialogs.open_print(&Status::Empty, &crate::app::prefs::PrintPrefs::default());
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

/// **Apply redactions cannot be opened without a document, and a second
/// invocation does not rebuild it.**
#[test]
fn the_apply_dialog_is_guarded_on_both_counts() {
    let mut dialogs = DialogsState::default();
    dialogs.open_redact(&Status::Empty, crate::app::prefs::RedactionReach::default());
    assert!(
        dialogs.redact.is_none(),
        "a document with nothing open has nothing to redact, and building \
         the dialog would run a full rewrite in order to refuse"
    );

    let status = Status::Open(Box::new(crate::app::state::open_fixture(
        crate::app::state::FOUR_PAGES,
    )));
    dialogs.open_redact(&status, crate::app::prefs::RedactionReach::default());
    let first = std::ptr::from_ref(dialogs.redact.as_ref().expect("open"));
    dialogs.open_redact(&status, crate::app::prefs::RedactionReach::default());
    let second = std::ptr::from_ref(dialogs.redact.as_ref().expect("still open"));
    assert_eq!(
        first, second,
        "the second press replaced the dialog, re-running the removal and \
         discarding both acknowledgements"
    );
}

/// The render report cannot be opened without a document either, and the
/// guard is the one that matters most for it.
#[test]
fn no_document_means_no_diagnostics_dialog() {
    let mut dialogs = DialogsState::default();
    dialogs.open_diagnostics(&Status::Empty);
    assert!(dialogs.diagnostics.is_none());
}

/// Pressing Render diagnostics twice does not rebuild the report.
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
#[test]
fn no_document_means_no_recognition_dialog() {
    let mut dialogs = DialogsState::default();
    dialogs.open_ocr(&Status::Empty, Vec::new(), None);
    assert!(dialogs.ocr.is_none());
}

/// About opens with no document, and survives the document closing.
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
#[test]
fn opening_about_twice_leaves_the_first_one_alone() {
    let mut dialogs = DialogsState::default();
    dialogs.open_about();
    let first = std::ptr::from_ref(dialogs.about.as_ref().expect("open"));
    dialogs.open_about();
    let second = std::ptr::from_ref(dialogs.about.as_ref().expect("still open"));
    assert_eq!(first, second, "the second press replaced the dialog");
}
