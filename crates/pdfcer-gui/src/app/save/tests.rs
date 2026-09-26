//! # `app::save::tests` — what is guaranteed about writing this document out
//!
//!
//! The seam is a real one. `save.rs` answers *"how does a document reach a
//! file?"* and grows when a save verb or a save mode is added; this answers
//! *"what must never be true of a file this shell wrote?"* and grows when a
//! way of getting that wrong is discovered. The two have different rates and
//! different readers — the second is the one somebody reads after a file came
//! out wrong.
//!
//! The suite's centre of gravity is deliberately the **truth table for
//! [`super::has_unsaved_edits`]**. Three of its assertions exist because a
//! two-term version of that predicate answered *clean* on a document with
//! unsaved work in it, twice, for two unrelated reasons — an in-place save the
//! engine's `is_modified()` cannot see, and a redaction that collapses the
//! session so that `is_modified()` correctly says *no*. Both were silent, and
//! both reached the operator.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/save/tests.md`.

// The INNER `#![cfg(test)]` is redundant — the module is declared
// `#[cfg(test)] mod tests;` — and it is here anyway, because
// `tools/gates/check-ui-strings.sh` exclusion 2 recognises a test-only FILE by
// exactly this attribute. Without it every assertion message below is read as
// operator copy outside the catalog.
#![cfg(test)]

use super::*;
use crate::app::state::{FOUR_PAGES, Origin, open_fixture, open_local_fixture};

/// A scratch path under the OS temporary directory, unique to this test.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pdfcer-gui-save-tests-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the temporary directory must be creatable");
    dir.join(name)
}

/// **The suggested name is never the file that was opened.**
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

/// **A created document is suggested its own name, with no suffix and no
/// folder.**
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

/// **The copy really is an incremental update: the original file's bytes
/// are its prefix, verbatim.**
#[test]
fn a_saved_copy_begins_with_the_original_file_byte_for_byte() {
    let doc = open_fixture(FOUR_PAGES);
    let source = std::fs::read(&doc.path).expect("the fixture is readable");
    let target = scratch("unedited.pdf");
    let _ = std::fs::remove_file(&target);

    let Written::Ordinary(report) =
        write_copy(&doc, &target).expect("an unedited document must save")
    else {
        panic!("an unstaged document must take the incremental writer")
    };
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

/// **An edit reaches the file, and it reaches it as an appended
/// revision.**
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
    let Written::Ordinary(report) =
        write_copy(&doc, &target).expect("an edited document must save")
    else {
        panic!("an unstaged document must take the incremental writer")
    };
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

/// **Saving does not touch the open document — not its epoch, not its
/// identity.**
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

/// **A document `file.new` made saves too, and re-opens as a document.**
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
    let Written::Ordinary(report) =
        write_copy(&created, &target).expect("a created document must save")
    else {
        panic!("an unstaged document must take the incremental writer")
    };
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

/// **The truth table for [`has_unsaved_edits`]** —
/// `OPERATOR_REQUESTS.md` O65.
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

    // The save. `saved_epoch` catches up; `is_modified()` does NOT and
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

/// **A write that cannot happen is reported rather than swallowed.**
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

//
// This section REPLACES the one that measured `Pass 250.1`'s collapse. The
// tests below are the same questions asked of a different mechanism, and one
// of them has changed its answer:
//
//   * is a document with an armed removal DIRTY?  — still yes, and the term
//     that sees it changed from `has_applied_redaction` to
//     `has_pending_redaction` (see `has_unsaved_edits`' own doc comment for
//     the reachable state that made the change necessary rather than tidy);
//   * does a save through this module remove the content?  — still yes, and
//     now through a completely different writer;
//   * does the proof bite?  — still yes, unchanged;
//   * …and one new question, because the pass brings a new trap with it: what
//     happens when the marks are undone under an armed removal.
// =======================================================================

/// **Stage a removal of `term` on `open`, the way the funnel does.**
fn stage(open: &mut crate::app::state::OpenDoc, term: &str) {
    let session =
        std::sync::Arc::get_mut(&mut open.session).expect("the session is not shared in a test");
    let created = session
        .mark_redactions_by_search(term, false)
        .expect("the drawing's text is extractable");
    assert!(!created.is_empty(), "the fixture must contain the term");
    let staged =
        crate::redact::stage_into_session(session, crate::app::prefs::RedactionReach::default())
            .expect("the staging must succeed");
    open.redaction_absence_claims = staged.report.redacted_text.clone();
    open.edit_epoch += 1;
}

/// **A document with an armed removal is UNSAVED, and the term that sees
/// it is `has_pending_redaction`.**
#[test]
fn a_document_with_a_staged_redaction_has_unsaved_edits() {
    let mut open = open_local_fixture("a1-titleblock.pdf");
    assert!(!has_unsaved_edits(&open), "untouched");

    // ui-text-exempt: a word in a test fixture.
    stage(&mut open, "FOUNDATION");

    assert!(
        open.session.has_pending_redaction(),
        "★ this is the term that sees it"
    );
    assert!(
        !open.session.has_applied_redaction(),
        "★★ and this one never can again. This shell does not collapse a \
         session any more; a true here means something reached the engine's \
         finalizing verb and `redact::sealed`'s counts are wrong."
    );
    assert!(
        has_unsaved_edits(&open),
        "★★★ a document whose most consequential intent is not on disk must \
         be dirty"
    );

    // The save catches up, exactly as it does for an ordinary edit.
    open.saved_epoch = open.edit_epoch;
    assert!(
        !has_unsaved_edits(&open),
        "a removal that has been written is not still owed — the epochs are \
         what turn the answer off"
    );

    // …and a further edit makes it unsaved work again.
    open.edit_epoch += 1;
    assert!(has_unsaved_edits(&open));
}

/// **The third term is load-bearing on a document the first two call
/// clean.**
#[test]
fn an_armed_removal_alone_is_enough_to_make_a_document_dirty() {
    let mut open = open_local_fixture("a1-titleblock.pdf");
    // ui-text-exempt: a word in a test fixture.
    stage(&mut open, "FOUNDATION");
    {
        let session = std::sync::Arc::get_mut(&mut open.session)
            .expect("the session is not shared in a test");
        session.undo().expect("the mark must be undoable");
    }
    open.edit_epoch += 1;

    assert!(
        !open.session.is_modified(),
        "★ the premise: with the mark undone the session no longer differs \
         from its base. If this stops being true the assertion below stops \
         being interesting, and a reader should know which one moved."
    );
    assert!(open.session.has_pending_redaction());
    assert!(
        has_unsaved_edits(&open),
        "★★★ WITHOUT the third term this answers CLEAN, and an armed removal \
         is discarded on Close with nothing asked."
    );
}

/// **A staged document saves through THIS module with the content gone —
/// and the ordinary writer is never reached.**
#[test]
fn a_staged_document_saves_through_the_redaction_writer_with_the_content_gone() {
    // ui-text-exempt: a word in a test fixture.
    const TERM: &str = "FOUNDATION";
    // ui-text-exempt: a word in a test fixture that must SURVIVE.
    const KEEP: &str = "DRAWING NO";

    let mut open = open_local_fixture("a1-titleblock.pdf");
    stage(&mut open, TERM);
    assert!(
        !open.redaction_absence_claims.is_empty(),
        "the engine must claim something will be removed, or the save-time \
         proof below has nothing to prove"
    );

    let target = scratch("staged-save.pdf");
    let _ = std::fs::remove_file(&target);
    let written = write_copy(&open, &target).expect("a staged document must be savable");
    assert!(
        matches!(written, Written::RedactionApplied(_)),
        "★ the staged fork must be the route taken. The incremental writer is \
         refused by the engine while a removal is armed, so a build that did \
         not fork would stop saving rather than save wrongly — which is loud, \
         and is still not what this module promises: {written:?}"
    );

    let bytes = std::fs::read(&target).expect("the file must exist");
    assert!(
        !bytes.windows(TERM.len()).any(|w| w == TERM.as_bytes()),
        "★★★ the removed text is in the file the operator would hand over"
    );
    // The positive control, and on a compressed CAD sheet it is the
    // assertion that does the work: a build that wrote an empty document
    // would satisfy the absence check above.
    let reopened = pdfcer_core::document::Document::load(&target).expect("the file must open");
    let text: String = pdfcer_core::text_extract::extract_document(
        &reopened,
        &pdfcer_core::text_extract::ExtractOptions::default(),
    )
    .expect("the saved drawing's text is extractable")
    .pages
    .iter()
    .flat_map(|p| p.runs.iter().map(|r| r.text.clone()))
    .collect();
    assert!(
        text.contains(KEEP),
        "an unmarked word on the same sheet must survive: {text}"
    );
    let _ = std::fs::remove_file(&target);
}

/// **The save-time proof refuses, and writes nothing.**
#[test]
fn a_save_whose_bytes_still_hold_the_redacted_text_is_refused_and_writes_nothing() {
    let mut open = open_local_fixture("a1-titleblock.pdf");
    // Never staged — the claim is a lie, and the proof must catch it.
    // ui-text-exempt: a word in a test fixture.
    open.redaction_absence_claims = vec!["FOUNDATION".to_owned()];
    assert!(
        !open.session.has_pending_redaction(),
        "the ordinary writer must be the one under test here"
    );

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

/// **Undoing the marks under an armed removal refuses the save BY NAME,
/// rather than writing an un-redacted file.**
#[test]
fn undoing_the_marks_under_an_armed_removal_refuses_the_save_by_name() {
    let mut open = open_local_fixture("a1-titleblock.pdf");
    // ui-text-exempt: a word in a test fixture.
    stage(&mut open, "FOUNDATION");
    {
        let session = std::sync::Arc::get_mut(&mut open.session)
            .expect("the session is not shared in a test");
        session
            .undo()
            .expect("★ the marks must be undoable — that is the whole pass");
        assert!(
            session.has_pending_redaction(),
            "…and undoing the marks does NOT un-arm the removal, which is why \
             this state exists at all"
        );
    }

    let target = scratch("armed-with-no-marks.pdf");
    let _ = std::fs::remove_file(&target);
    let error = write_copy(&open, &target).expect_err(
        "★★★ a save that cannot carry out the armed removal must refuse. \
         Falling back to the ordinary writer here would write the document \
         un-redacted, successfully, with the shell still claiming text had \
         been removed from it.",
    );
    assert!(
        matches!(
            error,
            SaveError::RedactionRefused {
                refusal: crate::redact::RedactApplyRefusal::NothingToApply
            }
        ),
        "and refuse by NAME, so the sentence can point at the control rather \
         than at a disk: {error}"
    );
    assert!(!target.exists(), "nothing may reach the disk");

    // And the sentence really does name the remedy, because a refusal an
    // operator cannot act on is one he learns to ignore.
    let said = crate::text::redact::save_refused_message(
        &crate::redact::RedactApplyRefusal::NothingToApply,
    );
    assert!(
        said.contains("Review & apply"),
        "the way out is a control in a window he has no reason to open; the \
         sentence has to send him there: {said}"
    );
}

//
// Added because the operator opened a file pdfcer had just written and
// found pages in it pdfcer did not believe were there:
//
//   "I tested deleting pages from a pdf. when I open the document in Acrobat
//    there are blank pages at the end of the document equalling the number of
//    pages I deleted."
//
// Everything about how the guard DECIDES is asserted in `crate::pagetree`.
// What is asserted HERE is the only thing that file cannot say: that the guard
// is actually WIRED into the funnel, that it refuses BY NAME, and that it
// leaves no file behind. A guard nobody called is the failure shape this
// project keeps rediscovering, and `check-verb-coverage`'s whole existence is
// the standing evidence that a note is not a mechanism.
// ==========================================================================

/// Open a PDF from an arbitrary path as an `OpenDoc`.
fn open_at(path: &std::path::Path) -> crate::app::state::OpenDoc {
    let doc = pdfcer_core::document::Document::load(path).expect("the document loads");
    let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
    crate::app::state::OpenDoc::new(
        path.to_path_buf(),
        pdfcer_core::edit::EditSession::new(doc),
        pages,
    )
}

/// **A healthy document still saves.**
#[test]
fn a_healthy_nested_document_still_saves() {
    let open = open_local_fixture("nested-page-tree.pdf");
    let target = scratch("pagetree-healthy.pdf");
    let _ = std::fs::remove_file(&target);
    write_copy(&open, &target).expect("★ a document that agrees with itself must save");
    assert!(target.exists());
    let _ = std::fs::remove_file(&target);
}

/// **A document whose page tree does not agree with itself is refused, by
/// name, and nothing reaches the disk.**
#[test]
fn a_document_whose_page_tree_disagrees_with_itself_is_refused_and_writes_nothing() {
    let source = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/nested-page-tree.pdf");
    let bytes = std::fs::read(&source).expect("the fixture is readable");
    const OLD: &[u8] = b"/Count 12";
    const NEW: &[u8] = b"/Count 13";
    let hits: Vec<usize> = bytes
        .windows(OLD.len())
        .enumerate()
        .filter_map(|(i, w)| (w == OLD).then_some(i))
        .collect();
    assert_eq!(
        hits.len(),
        1,
        "★ THE PLANT MUST LAND. The root is the only node declaring 12 pages; \
         if the fixture is regenerated with a different shape this substitution \
         silently stops corrupting anything and the assertions below pass for \
         the wrong reason."
    );
    let mut damaged = bytes.clone();
    damaged[hits[0]..hits[0] + OLD.len()].copy_from_slice(NEW);
    assert_eq!(
        damaged.len(),
        bytes.len(),
        "the plant must not move a byte, or the cross-reference table stops \
         resolving and this becomes a test about a broken file rather than \
         about a stale count"
    );

    let path = scratch("pagetree-stale-base.pdf");
    std::fs::write(&path, &damaged).expect("the damaged copy is writable");
    let open = open_at(&path);
    assert_eq!(
        open.pages.len(),
        12,
        "★★ and pdfcer's OWN reader is not fooled — it walks /Kids and reports \
         a healthy twelve-page document. That is the whole reason this guard \
         exists and the reason it cannot be built out of `page_tree::pages`."
    );

    let target = scratch("pagetree-refused.pdf");
    let _ = std::fs::remove_file(&target);
    let error = write_copy(&open, &target).expect_err(
        "★★★ a document that declares more pages than it has must not be \
         written — the file would open elsewhere with blank pages in it",
    );
    let SaveError::PageTreeStale { audit } = &error else {
        panic!("a stale page tree must be reported as one, not as a disk failure: {error}");
    };
    assert_eq!(audit.declared_pages, Some(13));
    assert_eq!(audit.reachable_pages, 12);
    let root = audit
        .root_disagreement()
        .expect("the ROOT is the node that disagrees, and it is the one Acrobat reads");
    assert_eq!((root.declared, root.reachable), (13, 12));
    assert!(
        !target.exists(),
        "★★ and NOTHING may reach the disk. A refusal that left a partial file \
         would be the same failure with a smaller file."
    );

    // And the sentence he is shown carries his own symptom back to him — a
    // refusal he cannot recognise is one he learns to ignore.
    let said = crate::text::pagetree::save_refused_root("x.pdf", root.declared, root.reachable);
    assert!(said.contains("1 blank page at the end"), "{said}");

    let _ = std::fs::remove_file(&path);
}

/// **The operator's own operation, end to end through this shell's save.**
#[test]
fn deleting_a_page_from_a_nested_document_is_caught_at_the_save() {
    let mut open = open_local_fixture("nested-page-tree.pdf");

    // THE FIXTURE MUST BE NESTED, AND IT IS ASSERTED RATHER THAN NAMED.
    //
    // Falsified 2026-09-05 by pointing this line at `fixtures/four-pages.pdf`
    // — a flat tree. The delete then came out clean (on a flat tree the
    // immediate parent IS the root, so the defect cannot occur), the `Ok` arm
    // below printed *"pdfcer-core now decrements /Count on every ancestor"* —
    // a false statement about a build carrying the defect in full — and the
    // test PASSED. That is precisely the vacuous shape this whole piece of
    // work exists to avoid, reached by editing one identifier.
    //
    // `depth` is the number of `/Pages` levels above a leaf. Anything below 3
    // cannot exhibit an ancestor-above-the-parent going stale, so the test
    // refuses to run rather than reporting a verdict it cannot have reached.
    let before = crate::pagetree::audit(&open.session.graph());
    assert!(
        before.depth >= 3,
        "★★★ this test is meaningless on a page tree shallower than three \
         levels — a build with no upward walk at all produces a correct file. \
         Regenerate the fixture with tools/gen-nested-page-tree-fixture.py: \
         {before:?}"
    );
    assert!(
        before.is_consistent(),
        "the fixture starts clean: {before:?}"
    );
    {
        let session =
            std::sync::Arc::get_mut(&mut open.session).expect("the session is not shared");
        session
            .delete_pages(&[1])
            .expect("page 2 (0-based 1) deletes");
    }

    let target = scratch("pagetree-after-delete.pdf");
    let _ = std::fs::remove_file(&target);
    match write_copy(&open, &target) {
        Err(SaveError::PageTreeStale { audit }) => {
            // The measured shape, v0.38.0 `b01964f`: eleven pages reachable,
            // the root still declaring twelve, and the grandparent stale too —
            // which is what says there is no upward walk at all rather than one
            // that stops one level short.
            assert_eq!(audit.reachable_pages, 11);
            assert_eq!(audit.declared_pages, Some(12));
            assert!(audit.disagreements.len() >= 2, "{audit:?}");
            assert!(!target.exists(), "nothing may reach the disk");
        }
        Ok(_) => {
            //
            // So the skip is conditional on the written FILE being clean, read
            // back independently. A save that succeeded because nobody looked
            // fails here, loudly, naming what is in the file it produced.
            let written = std::fs::read(&target).expect("a successful save wrote a file");
            let audit = crate::pagetree::audit_saved_bytes(&written);
            assert!(
                audit.walked,
                "the file this save produced must at least be walkable: {audit:?}"
            );
            assert!(
                audit.is_consistent(),
                "★★★ THE SAVE SUCCEEDED AND THE FILE IT WROTE IS DAMAGED. This \
                 is not the engine being fixed — it is the page-tree guard not \
                 running at the save. {audit:?}"
            );
            println!(
                "SKIP: pdfcer-core now decrements /Count on every ancestor, and \
                 the file this save wrote was verified consistent. The guard is \
                 still asserted by \
                 a_document_whose_page_tree_disagrees_with_itself_is_refused_and_writes_nothing; \
                 close the engine request and delete this test's engine half."
            );
            let _ = std::fs::remove_file(&target);
        }
        Err(other) => panic!("unexpected refusal: {other}"),
    }
}
