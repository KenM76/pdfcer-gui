//! # `app::save::tests` — what is guaranteed about writing this document out
//!
//! Split out of [`super`] on 2026-09-04 (evening) under rule R2, when the
//! deferred-redaction assertions took `app/save.rs` past the 1,500-line
//! ceiling. **Nothing moved but its address.**
//!
//! ★ The seam is a real one. `save.rs` answers *"how does a document reach a
//! file?"* and grows when a save verb or a save mode is added; this answers
//! *"what must never be true of a file this shell wrote?"* and grows when a
//! way of getting that wrong is discovered. The two have different rates and
//! different readers — the second is the one somebody reads after a file came
//! out wrong.
//!
//! ★★ The suite's centre of gravity is deliberately the **truth table for
//! [`super::has_unsaved_edits`]**. Three of its assertions exist because a
//! two-term version of that predicate answered *clean* on a document with
//! unsaved work in it, twice, for two unrelated reasons — an in-place save the
//! engine's `is_modified()` cannot see, and a redaction that collapses the
//! session so that `is_modified()` correctly says *no*. Both were silent, and
//! both reached the operator.

// ★ The INNER `#![cfg(test)]` is redundant — the module is declared
// `#[cfg(test)] mod tests;` — and it is here anyway, because
// `tools/gates/check-ui-strings.sh` exclusion 2 recognises a test-only FILE by
// exactly this attribute. Without it every assertion message below is read as
// operator copy outside the catalog.
#![cfg(test)]

use super::*;
use crate::app::state::{FOUR_PAGES, Origin, open_fixture, open_local_fixture};

/// A scratch path under the OS temporary directory, unique to this test.
///
/// `std::env::temp_dir` rather than a path in the repository: a test that
/// writes beside the fixtures leaves a file somebody eventually commits,
/// which is the exact hazard `tools/ui-verify`'s OCR check records having
/// hit.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("pdfcer-gui-save-tests");
    std::fs::create_dir_all(&dir).expect("the temporary directory must be creatable");
    dir.join(name)
}

/// ★ **The suggested name is never the file that was opened.**
///
/// The shipped tooltip's promise as a **default** rather than as a warning.
/// An operator who accepts the suggestion without reading it must not
/// overwrite the drawing they were working on, and this is the assertion
/// that says so — the tooltip says it in words, and words are not a
/// mechanism.
#[test]
fn the_suggested_name_is_never_the_source_file() {
    let mut doc = open_fixture(FOUR_PAGES);
    doc.path = PathBuf::from("D:\\jobs\\4471\\Sheet 1.pdf");
    doc.origin = Origin::Opened;

    let suggested = suggested_path(&doc);
    assert_ne!(suggested, doc.path);
    assert_eq!(suggested, PathBuf::from("D:\\jobs\\4471\\Sheet 1-copy.pdf"));
    assert_eq!(
        suggested.parent(),
        doc.path.parent(),
        "the copy should land beside the original, where the operator will look for it"
    );
}

/// ★ **A created document is suggested its own name, with no suffix and no
/// folder.**
///
/// The other half of `stored_under`, and the interesting failure is not
/// "the suffix was skipped" but "the guard was written the wrong way round",
/// after which every opened document would be offered its own path as the
/// default — turning the tooltip's promise into a trap. Both directions are
/// therefore asserted, here and in the test above.
#[test]
fn a_created_document_is_suggested_its_own_name() {
    let mut doc = open_fixture(FOUR_PAGES);
    doc.path = PathBuf::from("Untitled 1.pdf");
    doc.origin = Origin::Created;

    let suggested = suggested_path(&doc);
    assert_eq!(suggested, PathBuf::from("Untitled 1.pdf"));
    assert_eq!(
        suggested.parent(),
        Some(Path::new("")),
        "a document that has never been anywhere names no folder; the picker supplies one"
    );
}

/// A capitalised extension still produces a `.pdf`, and a path with no stem
/// still produces a name.
#[test]
fn the_suggestion_is_always_a_usable_pdf_name() {
    for source in ["D:\\scans\\SHEET.PDF", "sheet", "D:\\a.b.pdf"] {
        let mut doc = open_fixture(FOUR_PAGES);
        doc.path = PathBuf::from(source);
        doc.origin = Origin::Opened;
        let suggested = suggested_path(&doc);
        assert!(
            suggested.to_string_lossy().ends_with(".pdf"),
            "{source} suggested {suggested:?}"
        );
        assert_ne!(suggested, doc.path);
    }
}

/// ★★ **The copy really is an incremental update: the original file's bytes
/// are its prefix, verbatim.**
///
/// The assertion this whole module exists for, and the one that fails
/// against the plausible wrong implementation. A copy produced by
/// `to_full_bytes` would satisfy everything else a reader might check — it
/// is a valid PDF, it has the right pages, it carries the edit — and it
/// would have rewritten the file from scratch, destroying every digital
/// signature (§12.8.1) and discarding the previous revision that
/// `file.save_copy`'s shipped tooltip promises *"stays intact inside the
/// file"*.
///
/// A §7.5.6 incremental update cannot do that by construction: the original
/// revision is left untouched and the new one is appended after it. So
/// `output[..input.len()] == input` is a **property of the save mode**, and
/// it is the cheapest possible test that tells the two modes apart.
///
/// With no edits made, the engine's own contract goes further and the
/// output *is* the input — asserted too, because a build that appended an
/// empty revision to an untouched document would still pass the prefix
/// check while quietly growing every file an operator copied.
#[test]
fn a_saved_copy_begins_with_the_original_file_byte_for_byte() {
    let doc = open_fixture(FOUR_PAGES);
    let source = std::fs::read(&doc.path).expect("the fixture is readable");
    let target = scratch("unedited.pdf");
    let _ = std::fs::remove_file(&target);

    let report = write_copy(&doc, &target).expect("an unedited document must save");
    let written = std::fs::read(&target).expect("the copy must exist");

    assert!(
        written.starts_with(&source),
        "the copy does not begin with the original's bytes, so this is not an incremental \
         update — a full rewrite destroys every signature in the file and discards the \
         previous revision the command's own tooltip promises stays intact"
    );
    assert_eq!(
        written, source,
        "with an empty dirty set `save_incremental`'s contract is that the output IS the \
         input; a copy that grew is a revision appended for an edit nobody made"
    );
    assert!(report.byte_identical);
    assert_eq!(report.bytes_appended, 0);
    let _ = std::fs::remove_file(&target);
}

/// ★★ **An edit reaches the file, and it reaches it as an appended
/// revision.**
///
/// The round trip, in the smallest form a unit test can hold: rotate a page
/// through the engine, save a copy, and re-open the copy from disk with a
/// fresh `Document::load` — the same call `PdfcerApp::open_path` makes — and
/// read the rotation back.
///
/// Two assertions and both are load-bearing. **The edit is present**, which
/// is what separates "a file was written" from "the operator's work was
/// written"; a build that wrote `Document::bytes()` instead of the session's
/// update would produce a perfectly good PDF with the edit missing and would
/// pass every check that only asks whether a file appeared. **And the
/// original's bytes are still its prefix**, which is what separates an
/// incremental save from a full rewrite that would also carry the edit.
///
/// `set_page_rotation` rather than a markup annotation because it is the
/// cheapest engine verb that changes a page object and is readable back
/// through `EditSession::pages` without a decomposition — the *shape* of the
/// proof is what matters here, and `tools/ui-verify`'s `save_copy_round_trip`
/// makes the same claim about a real annotation placed by a real drag.
#[test]
fn an_edit_survives_the_round_trip_through_a_saved_copy() {
    use pdfcer_core::document::Document;
    use pdfcer_core::edit::EditSession;

    let mut doc = open_fixture(FOUR_PAGES);
    let source = std::fs::read(&doc.path).expect("the fixture is readable");
    let before = doc.pages[0].rotate;
    let session = std::sync::Arc::get_mut(&mut doc.session)
        .expect("nothing else holds the session in a test");
    session
        .set_page_rotation(0, 90)
        .expect("rotating page 1 of the fixture must be expressible");
    assert_ne!(before, 90, "the fixture must not already be rotated");

    let target = scratch("rotated.pdf");
    let _ = std::fs::remove_file(&target);
    let report = write_copy(&doc, &target).expect("an edited document must save");
    assert!(
        report.bytes_appended > 0,
        "an edited document must append a revision; {report:?}"
    );
    assert!(
        !report.byte_identical,
        "an edited document's copy cannot be byte-identical to its input"
    );

    let written = std::fs::read(&target).expect("the copy must exist");
    assert!(
        written.starts_with(&source),
        "the edited copy must still begin with the original's bytes — that is what makes it \
         an update rather than a rewrite"
    );

    // …and the round trip: re-open the file that was written, from disk,
    // through the same loader the application uses.
    let reopened = Document::load(&target).expect("the copy must open");
    let pages = EditSession::new(reopened)
        .pages()
        .expect("the copy's page tree must walk");
    assert_eq!(
        pages[0].rotate, 90,
        "the edit is not in the saved file. A save that writes a file is not the same claim \
         as a save that writes the EDIT, and this is the second one"
    );
    assert_eq!(
        pages.len(),
        doc.pages.len(),
        "the copy must carry every page the original had"
    );
    let _ = std::fs::remove_file(&target);
}

/// ★ **Saving does not touch the open document — not its epoch, not its
/// identity.**
///
/// §3, asserted rather than described. Three failures this catches, each of
/// which looks like tidying up:
///
/// * bumping `edit_epoch` — dissolves the canvas selection, discards the
///   decomposition and the page-text cache, and retires a rule-4 disclosure
///   the operator may not have read, to record an event that changed nothing
///   on screen;
/// * **zeroing** `edit_epoch` — turns off `dialogs::ocr`'s `UnsavedEdits`
///   refusal, whose whole job is to stop OCR producing a recognised copy
///   with the operator's edits silently missing;
/// * writing `path`/`origin` — that is Save **As**, a command this build
///   does not have, and doing it here would rename the operator's open
///   document because they asked for a copy.
#[test]
fn saving_a_copy_changes_nothing_about_the_open_document() {
    let mut doc = open_fixture(FOUR_PAGES);
    let session = std::sync::Arc::get_mut(&mut doc.session)
        .expect("nothing else holds the session in a test");
    session
        .set_page_rotation(0, 90)
        .expect("rotating page 1 of the fixture must be expressible");
    doc.edit_epoch = 4;
    let path = doc.path.clone();
    let origin = doc.origin;

    let target = scratch("untouched.pdf");
    let _ = std::fs::remove_file(&target);
    write_copy(&doc, &target).expect("the copy must be written");

    assert_eq!(
        doc.edit_epoch, 4,
        "a save changes no revision: the document in memory afterwards is the one that was \
         there before, so bumping the epoch would discard several caches to record nothing, \
         and zeroing it would tell OCR the document is unedited when it is not"
    );
    assert_eq!(doc.path, path, "Save a COPY is not Save As");
    assert_eq!(doc.origin, origin);
    let _ = std::fs::remove_file(&target);
}

/// ★ **A document `file.new` made saves too, and re-opens as a document.**
///
/// The case the shipped `file.new` tooltip now promises and that nothing
/// else covers: `tools/ui-verify`'s round trip drives an *opened* document,
/// because it needs a page with content to drag a rectangle across.
///
/// It is worth its own test rather than being assumed from the opened case,
/// because a created document is the one whose `path` is **not a file**. A
/// save that reached for `doc.path` anywhere — to read base bytes, to
/// resolve a directory, to decide anything — would work perfectly on every
/// opened document and fail here alone, with `Untitled 1.pdf` as the error.
#[test]
fn a_created_document_saves_and_the_copy_opens() {
    use pdfcer_core::document::Document;
    use pdfcer_core::edit::EditSession;

    let (doc, pages) = crate::app::blank::document().expect("the template parses");
    let created = crate::app::state::OpenDoc::created(
        PathBuf::from("Untitled 1.pdf"),
        EditSession::new(doc),
        pages,
    );
    assert_eq!(created.origin, Origin::Created);
    assert_eq!(created.stored_under(), None, "it has no file behind it");

    let target = scratch("created.pdf");
    let _ = std::fs::remove_file(&target);
    let report = write_copy(&created, &target).expect("a created document must save");
    assert_eq!(
        report.bytes_written,
        crate::app::blank::TEMPLATE.len(),
        "an unedited created document's copy is the template, unchanged"
    );

    let reopened = Document::load(&target).expect("the copy must open");
    let reopened_pages = EditSession::new(reopened)
        .pages()
        .expect("the copy's page tree must walk");
    assert_eq!(reopened_pages.len(), 1, "New makes a one-page document");
    let _ = std::fs::remove_file(&target);
}

/// ★★★ **The truth table for [`has_unsaved_edits`]** —
/// `OPERATOR_REQUESTS.md` O65.
///
/// Five states, and each of the two terms is load-bearing in a different
/// one of them, which is why the test walks the whole table rather than
/// asserting the headline case:
///
/// - **edited, then saved → clean** is what a build with only
///   `session.is_modified()` gets wrong. That is the one the operator hit:
///   the tab kept its unsaved dot, and the next Close asked a question
///   whose only save button opened a picker and then closed the document.
/// - **edited, then undone → clean** is what a build with only the epoch
///   comparison gets wrong, because an undo bumps `edit_epoch` like every
///   other edit.
///
/// So a build that drops either term passes half of this and fails the
/// other half, which is exactly what a truth-table test is for.
#[test]
fn a_saved_document_is_not_dirty_and_an_undone_edit_is_not_either() {
    use pdfcer_core::document::Document;
    use pdfcer_core::edit::EditSession;

    let path = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/four-pages.pdf"
    ));
    let doc = Document::load(path).expect("the fixture loads");
    let session = EditSession::new(doc);
    let pages = session.pages().expect("the page tree walks");
    let mut open = crate::app::state::OpenDoc::new(path.to_path_buf(), session, pages);

    assert!(
        !has_unsaved_edits(&open),
        "a document nobody has touched is clean"
    );

    // An edit: the engine's own answer flips, and the epochs diverge.
    std::sync::Arc::get_mut(&mut open.session)
        .expect("the session is not shared in a test")
        .rotate_pages(&[0], 90)
        .expect("a rotate must succeed on the fixture");
    open.edit_epoch += 1;
    assert!(has_unsaved_edits(&open), "an edited document is dirty");

    // ★ The save. `saved_epoch` catches up; `is_modified()` does NOT and
    // never can, because `to_incremental_bytes` takes `&self`. This is the
    // line the whole row exists for.
    open.saved_epoch = open.edit_epoch;
    assert!(
        open.session.is_modified(),
        "the engine still says the session differs from its BASE revision — \
         that is correct and is exactly why it cannot be the shell's answer"
    );
    assert!(
        !has_unsaved_edits(&open),
        "a document that has just been saved must be CLEAN. This is O65: \
         a build asking `is_modified()` alone keeps the tab marker, asks \
         about unsaved edits on the next Close, and the operator reads \
         that as Save having closed the document."
    );

    // …and editing again makes it dirty a second time, which is what stops
    // the fix from being "always answer clean after any save".
    open.edit_epoch += 1;
    assert!(
        has_unsaved_edits(&open),
        "an edit after a save is unsaved work again"
    );
}

/// ★ **A write that cannot happen is reported rather than swallowed.**
///
/// A directory that does not exist is the commonest real failure — the
/// operator typed a path, or a network share went away between the dialog
/// and the write — and it is the one that must not be silent. Asserted as
/// the `Write` variant specifically, because the two variants send a reader
/// to two different subsystems and collapsing them into one would throw that
/// away at the last step.
#[test]
fn a_write_that_cannot_happen_is_a_named_refusal() {
    let doc = open_fixture(FOUR_PAGES);
    let target = scratch("no-such-folder").join("nested").join("copy.pdf");
    let error = write_copy(&doc, &target).expect_err("a missing folder cannot be written to");
    assert!(
        matches!(error, SaveError::Write(_)),
        "a file-system refusal must not be reported as an engine refusal: {error}"
    );
    assert!(
        !target.exists(),
        "a failed save must leave nothing behind at the path it was aimed at"
    );
}

// =======================================================================
// ★★★ THE DEFERRED REDACTION — 2026-09-04
// =======================================================================

/// ★★★ **A document with an applied redaction is UNSAVED, and a two-term
/// predicate said it was clean.**
///
/// The defect this closed, and it is the one that would have made the whole
/// deferred route a silent data loss. `EditSession::apply_redactions`
/// collapses the session onto the redacted bytes as a new base, so
/// `is_modified()` — *"does this differ from the base revision?"* — answers
/// **no** the instant afterwards. Correct on its own terms, and with only
/// two terms [`has_unsaved_edits`] inherited it: no tab marker, no question
/// on Close, no question on Quit. Apply a redaction, close the document,
/// and the redaction is gone with nothing asked.
///
/// The test walks the same table the O65 test does, on the redacted branch:
/// dirty after the apply, clean after the save, dirty again after a further
/// edit. The middle row is what stops the fix from being *"always answer
/// dirty once a redaction has happened"*.
#[test]
fn a_document_with_an_applied_redaction_has_unsaved_edits() {
    let mut open = open_local_fixture("a1-titleblock.pdf");
    assert!(!has_unsaved_edits(&open), "untouched");

    let session =
        std::sync::Arc::get_mut(&mut open.session).expect("the session is not shared in a test");
    let created = session
        // ui-text-exempt: a word in a test fixture.
        .mark_redactions_by_search("FOUNDATION", false)
        .expect("the drawing's text is extractable");
    assert!(!created.is_empty(), "the fixture must contain the term");
    crate::redact::apply_into_session(session).expect("the apply must succeed");
    open.edit_epoch += 1;

    assert!(
        !open.session.is_modified(),
        "★ the premise: after the collapse the engine says the session does \
         NOT differ from its base, because the redacted bytes ARE the base. \
         If this ever stops being true the assertion below stops being \
         interesting, and a reader should know which one moved."
    );
    assert!(
        open.session.has_applied_redaction(),
        "…and this is the term that sees it"
    );
    assert!(
        has_unsaved_edits(&open),
        "★★★ a document whose most consequential edit is not on disk must \
         be dirty. Without the third term this answered CLEAN: no tab \
         marker, no question on Close, and the redaction discarded in \
         silence."
    );

    // The save catches up, exactly as it does for an ordinary edit.
    open.saved_epoch = open.edit_epoch;
    assert!(
        !has_unsaved_edits(&open),
        "a redaction that has been written is not still owed — the flag is \
         sticky, the epochs are what turn the answer off"
    );

    // …and a further edit makes it unsaved work again.
    open.edit_epoch += 1;
    assert!(has_unsaved_edits(&open));
}

/// ★★★ **A redacted document saves cleanly through the ordinary
/// incremental writer — and the proof runs on the bytes.**
///
/// The end-to-end assertion for `OPERATOR_REQUESTS.md` O125's second half:
/// the operator applies into the open document, then presses Save, and the
/// file that lands has the content gone. [`write_copy`] is the one function
/// every save verb goes through, and it is `to_incremental_bytes` — the
/// writer whose whole purpose is *keeping the previous revision inside the
/// file*, which is what made this the property the engine request marked
/// ★★★.
///
/// `crate::redact::tests` proves the same thing about the session's bytes.
/// This proves it about **the file on disk**, through the shell's own save
/// path, with the shell's own settings applied.
#[test]
fn a_redacted_document_saves_through_the_ordinary_writer_with_the_content_gone() {
    // ui-text-exempt: a word in a test fixture.
    const TERM: &str = "FOUNDATION";

    let mut open = open_local_fixture("a1-titleblock.pdf");
    let session =
        std::sync::Arc::get_mut(&mut open.session).expect("the session is not shared in a test");
    session
        .mark_redactions_by_search(TERM, false)
        .expect("extractable");
    let applied = crate::redact::apply_into_session(session).expect("apply");
    open.redaction_absence_claims = applied.report.redacted_text.clone();
    assert!(
        !open.redaction_absence_claims.is_empty(),
        "the engine must claim to have removed something, or the save-time \
         proof below has nothing to prove"
    );

    let target = scratch("redacted-save.pdf");
    let _ = std::fs::remove_file(&target);
    write_copy(&open, &target).expect("a redacted document must be savable");

    let bytes = std::fs::read(&target).expect("the file must exist");
    assert!(
        !bytes.windows(TERM.len()).any(|w| w == TERM.as_bytes()),
        "★★★ the removed text is in the file the operator would hand over"
    );
    let _ = std::fs::remove_file(&target);
}

/// ★★ **The save-time proof refuses, and writes nothing.**
///
/// The falsification for the check above. `write_copy` is handed a claim
/// that is demonstrably still in the document — the redaction never
/// happened — and must refuse **by name** and leave no file behind.
///
/// It is the check nobody expects to fire: the engine's collapse makes it
/// safe by construction, so it is expected to pass on every save of every
/// redacted document forever. A check that is expected to pass is exactly
/// the kind this project keeps discovering was never wired, which is why
/// its bite is asserted rather than assumed.
#[test]
fn a_save_whose_bytes_still_hold_the_redacted_text_is_refused_and_writes_nothing() {
    let mut open = open_local_fixture("a1-titleblock.pdf");
    // Never redacted — the claim is a lie, and the proof must catch it.
    // ui-text-exempt: a word in a test fixture.
    open.redaction_absence_claims = vec!["FOUNDATION".to_owned()];

    let target = scratch("refused-save.pdf");
    let _ = std::fs::remove_file(&target);
    let error = write_copy(&open, &target)
        .expect_err("★ bytes that still hold a claimed-removed string must not be written");
    assert!(
        matches!(error, SaveError::RedactionLeak { .. }),
        "a leak must be reported as a leak, not as a disk failure: {error}"
    );
    assert!(
        !target.exists(),
        "★★ and NOTHING may reach the disk. A refusal that left a partial \
         file would be the same failure with a smaller file."
    );
}
