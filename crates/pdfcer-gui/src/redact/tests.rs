//! # `redact::tests` — the security assertions for the apply pipeline
//!
//! Split out of [`super`] on 2026-09-04 (evening) under rule R2, when the
//! deferred apply route (`Pass 250.1`) took `redact/mod.rs` past the
//! 1,500-line ceiling. **Nothing moved but its address**: the suite, its
//! fixtures and every paragraph of its reasoning are unchanged, and the seam
//! is the one R2 asks for rather than a line count — `mod.rs` answers *"what
//! is the pipeline?"* and this file answers *"what has been proven about
//! it?"*, and the two grow for different reasons.
//!
//! ★ It stays a module named `tests` inside `redact`, deliberately:
//! [`super::proof`]'s own suite reaches [`assemble`] as
//! `super::super::tests::assemble`, and a rename would have made a mechanical
//! move into a second, subtly different PDF assembler — which is the one thing
//! a fixture must not become.
//!
//! ★★ It is also inside the one file the call-site monopoly permits — no, it
//! is not, and that is worth stating rather than leaving to be noticed:
//! `redact::sealed` sweeps **every** `.rs` file in the crate, this one
//! included, and nothing here calls the engine's removal directly. Every test
//! below goes through [`super::prepare_redaction_apply`],
//! [`super::stage_into_session`], [`super::save_applying_pending`] or
//! [`super::cancel_staged_redaction`], which is exactly the property the
//! monopoly exists to keep true of test code as well as of production code.

// ★ The INNER `#![cfg(test)]` is redundant — the module is declared
// `#[cfg(test)] mod tests;` — and it is here anyway, because
// `tools/gates/check-ui-strings.sh` exclusion 2 recognises a test-only FILE by
// exactly this attribute. Without it every assertion message below is read as
// operator copy outside the catalog, which is the 125-hit noise floor that
// exclusion was written to remove.
#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;
use pdfcer_core::annot_author::{Quad, RedactSpec};
use pdfcer_core::page_tree::{self, Rect};
use pdfcer_core::text_extract::{self, ExtractOptions};
use pdfcer_core::vartext::Quadding;

/// The secret this suite proves the absence of.
///
/// Deliberately long and distinctive: a short token could be absent by
/// luck, and a proof that can pass by luck proves nothing.
const SECRET: &str = "CONFIDENTIALWITNESSNAME";

/// A one-page document whose content stream shows `SECRET` followed by a
/// word that must SURVIVE.
///
/// The survivor is what stops the test from passing on a build that simply
/// erased the page.
fn secret_pdf() -> Vec<u8> {
    let content = format!("BT /F1 12 Tf 20 100 Td ({SECRET}) Tj ( KEEPTHIS) Tj ET");
    let stream = format!(
        "<< /Length {} >>\nstream\n{content}\nendstream",
        content.len()
    );
    assemble(&[
        "<< /Type /Catalog /Pages 2 0 R >>",
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 200] \
         /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>",
        &stream,
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
    ])
}

/// Assemble a classic single-revision PDF from object bodies `1..=n` with a
/// correct xref table. Object 1 must be the catalog.
///
/// The same fixture shape `pdfcer-core`'s own redaction tests use —
/// synthetic, so that every byte in the file is one this suite put there.
/// `pub(super)` so [`super::proof`]'s tests share it rather than growing a
/// second, subtly different assembler.
pub(super) fn assemble(bodies: &[&str]) -> Vec<u8> {
    let mut buf = b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in bodies.iter().enumerate() {
        offsets.push(buf.len());
        buf.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
    }
    let xref_at = buf.len();
    let n = bodies.len() + 1;
    buf.extend_from_slice(format!("xref\n0 {n}\n0000000000 65535 f \n").as_bytes());
    for off in &offsets {
        buf.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size {n} /Root 1 0 R >>\nstartxref\n{xref_at}\n%%EOF\n").as_bytes(),
    );
    buf
}

/// A session with ONE unsaved `/Redact` mark over the secret — the exact
/// state an operator is in when they press Apply without having saved.
fn session_with_unsaved_mark() -> EditSession {
    let doc = Document::from_bytes(secret_pdf()).unwrap();
    let mut session = EditSession::new(doc);
    let created = session
        .mark_redactions_by_search(SECRET, false)
        .expect("the fixture's text is extractable");
    assert!(!created.is_empty(), "the search must find the secret");
    session
}

/// A scratch path under the OS temporary directory, unique to this test.
///
/// `std::env::temp_dir` rather than a path in the repository, exactly as
/// `crate::app::save`'s tests do it: a test that writes beside the fixtures
/// leaves a file somebody eventually commits.
fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("pdfcer-gui-redact-tests");
    std::fs::create_dir_all(&dir).expect("the temporary directory must be creatable");
    dir.join(name)
}

// -- THE SECURITY ASSERTION ---------------------------------------------

/// ★★ **The headline gate for the apply path.**
///
/// After apply-and-save through [`prepare_redaction_apply`], the redacted
/// text must not be recoverable from the saved bytes by any means pdfcer
/// itself offers. Three independent measures, because a single one could be
/// satisfied by a build that merely hid the text:
///
/// 1. **`extract-text`** — the very tool `pdfcer extract-text` and this
///    shell's Copy-text both use — finds nothing;
/// 2. **every decoded stream** (content streams, XObjects, object-stream
///    containers, metadata) contains no occurrence;
/// 3. **the raw file bytes** contain no occurrence.
///
/// And the negative control: `KEEPTHIS`, which was never marked, is still
/// extractable. Without it, a build that emitted an empty page would pass
/// all three assertions above while destroying the document.
///
/// This is deliberately an assertion of ABSENCE, not of appearance. A
/// raster test could only show that the region is painted black, which is
/// precisely the false-redaction failure ISO 32000-1 §12.5.6.23 forbids
/// ("clipping or image masks shall not be used to hide that data") — a black
/// box over live text is what this feature exists to never ship.
#[test]
fn applied_redaction_leaves_no_recoverable_trace_in_the_saved_bytes() {
    let session = session_with_unsaved_mark();
    let prepared = prepare_redaction_apply(&session).expect("the apply must succeed");

    // The bytes are private, so the assertion goes through the one door
    // that exists — which is itself the property this module is about.
    let target = scratch("headline.pdf");
    let _ = std::fs::remove_file(&target);
    let written = prepared
        .write_to(&target, ResidualAcknowledgement::Withheld)
        .expect("a clean redaction needs no acknowledgement");
    assert_eq!(written, prepared.byte_len());
    let bytes = std::fs::read(&target).expect("the redacted file must exist");

    // (3) raw bytes.
    assert!(
        !proof::contains(&bytes, SECRET.as_bytes()),
        "the redacted text survived in the raw saved bytes"
    );

    let back = Document::from_bytes(bytes.clone()).expect("the redacted output must re-parse");

    // (2) every decoded stream in the file — asked through the proof's own
    // sweep, which is the wide one.
    assert_eq!(
        proof::survivors_in_content_streams(&bytes, &[SECRET.to_owned()]),
        None,
        "the redacted text survived in a decoded stream of the saved file"
    );

    // (1) pdfcer's own text extraction — the tool an operator would actually
    // reach for to get the text back out.
    let extracted =
        text_extract::extract_document(&back, &ExtractOptions::default()).expect("extract");
    let all_text: String = extracted
        .pages
        .iter()
        .flat_map(|p| p.runs.iter())
        .map(|r| r.text.clone())
        .collect();
    assert!(
        !all_text.contains(SECRET),
        "the redacted text was recoverable via extract-text: {all_text:?}"
    );

    // The negative control — proof the test can fail.
    assert!(
        all_text.contains("KEEPTHIS"),
        "un-redacted text must survive; the page was not supposed to be emptied"
    );

    // And the mark itself is gone (§12.5.6.23 outcome 3).
    assert_eq!(
        redact::count_redaction_marks(&back),
        0,
        "the /Redact mark must be removed by apply"
    );
    let _ = std::fs::remove_file(&target);
}

/// The absence proof must REPORT that it ran, or the wording contract has
/// nothing to read and the summary would have to fall back to the weaker
/// word.
#[test]
fn the_absence_proof_reports_a_clean_verification() {
    let session = session_with_unsaved_mark();
    let prepared = prepare_redaction_apply(&session).unwrap();
    assert!(
        prepared.verification.strings_checked > 0,
        "the proof must have had something to check"
    );
    assert!(
        prepared.verification.is_clean(),
        "no residual expected on this fixture: {:?}",
        prepared.verification.residuals
    );
}

/// ★ **A mark that exists ONLY in the session overlay must still be
/// applied.**
///
/// The un-saved-mark trap §1.2 names: passing `session.document()` to
/// `apply_redactions` would apply nothing and report success. The assertion
/// that makes it bite is `marks_applied` — a build with that bug produces
/// `NothingToApply` or a zero count, never a removal.
#[test]
fn a_mark_that_was_never_saved_is_still_applied() {
    let session = session_with_unsaved_mark();
    // The base revision genuinely has no mark — that is the trap.
    assert_eq!(redact::count_redaction_marks(session.document()), 0);
    assert!(redact::count_redaction_marks(&session.graph()) > 0);

    let prepared = prepare_redaction_apply(&session).unwrap();
    assert!(
        prepared.report.marks_applied >= 1,
        "an unsaved mark must be applied, not silently skipped"
    );
    assert!(prepared.report.glyphs_removed >= SECRET.len() as u64);
}

/// ★ **The output is a SINGLE revision.**
///
/// A `/Prev` in the trailer would mean a prior revision is reachable in the
/// saved file, which for a redaction is the un-redacted content one hop
/// away — R35's whole point, and the reason §1.1 forbids the incremental
/// writer this shell otherwise uses for every save.
#[test]
fn the_output_is_one_revision_with_no_prior_revision_to_walk_back_to() {
    let session = session_with_unsaved_mark();
    let prepared = prepare_redaction_apply(&session).unwrap();
    let target = scratch("one-revision.pdf");
    let _ = std::fs::remove_file(&target);
    prepared
        .write_to(&target, ResidualAcknowledgement::Withheld)
        .unwrap();
    let back = Document::from_bytes(std::fs::read(&target).unwrap()).unwrap();
    assert!(
        back.trailer().get(b"Prev").is_none(),
        "a redaction apply must leave no /Prev — a prior revision holds the un-redacted bytes"
    );
    let _ = std::fs::remove_file(&target);
}

/// A document with no marks is refused by name rather than producing an
/// empty "successful" apply, so the caller can never present a report that
/// describes nothing as if it were a removal.
#[test]
fn an_unmarked_document_is_refused_by_name() {
    let doc = Document::from_bytes(secret_pdf()).unwrap();
    let session = EditSession::new(doc);
    assert_eq!(
        prepare_redaction_apply(&session).unwrap_err(),
        RedactApplyRefusal::NothingToApply
    );
}

/// ★★★ **A region over a raster image now DESTROYS the samples**, and this
/// test is the record of the day that changed.
///
/// It read `a_region_over_an_image_refuses_the_whole_apply` until
/// 2026-09-03 and asserted that the engine declined the entire document —
/// which was true, was the operator's headline complaint
/// (`OPERATOR_REQUESTS.md` O103, *"every time I've tried the redact feature
/// it tells me it can't"*), and stopped being true with `pdfcer-core`
/// v0.26.0 the same day.
///
/// ★★★ **Writing over the source replaces it, and leaves no temporary
/// behind.**
///
/// The destination [`crate::dialogs::redact::Destination::ReplaceOriginal`]
/// produces, added 2026-09-04, and the reason
/// [`PreparedRedaction::write_to`] became atomic on the same day: the old
/// `std::fs::write` was defensible only while the target could never be the
/// operator's own document, and a torn write there destroys the last
/// remaining copy of the content being removed.
///
/// Three assertions, and the third is the one a `write`-based build passes
/// by accident and an unclean temp-file build fails:
///
/// 1. the target holds the **redacted** bytes afterwards, not the original;
/// 2. the original content is **gone** from it;
/// 3. no `.pdfcer-tmp` file is left beside it — and that file would contain
///    a complete redacted document, which is the last kind of stray
///    artefact this feature should scatter around somebody's job folder.
#[test]
fn writing_over_the_source_replaces_it_and_leaves_no_temporary() {
    let target = scratch("replace-in-place.pdf");
    let temporary = target.with_extension("pdfcer-tmp");
    let _ = std::fs::remove_file(&target);
    let _ = std::fs::remove_file(&temporary);

    // The document exists on disk first — this is a REPLACE, not a create.
    std::fs::write(&target, secret_pdf()).expect("the source must exist");
    let before = std::fs::read(&target).expect("read back");
    assert!(
        proof::contains(&before, SECRET.as_bytes()),
        "the file being replaced must contain the secret, or nothing below \
         is a test"
    );

    let session = session_with_unsaved_mark();
    let prepared = prepare_redaction_apply(&session).expect("the apply must succeed");
    prepared
        .write_to(&target, ResidualAcknowledgement::Withheld)
        .expect("writing over the source is now a supported destination");

    let after = std::fs::read(&target).expect("the replaced file is readable");
    assert_ne!(before, after, "the file was not replaced at all");
    assert!(
        !proof::contains(&after, SECRET.as_bytes()),
        "the replacement still contains the secret"
    );
    assert!(
        !temporary.exists(),
        "★ a temporary holding a complete redacted document was left beside \
         the operator's file: {}",
        temporary.display()
    );
    let _ = std::fs::remove_file(&target);
}

/// ★★★ **The whole pipeline, on a REAL document, from mark to written
/// file — the test whose absence let the 2026-09-04 defect ship.**
///
/// # Why this exists, and why every other test in this module missed it
///
/// Everything above runs on [`assemble`]d fixtures: a handful of objects,
/// uncompressed streams, `/Helvetica`, no embedded font, no compression, no
/// object streams. Those fixtures are right for what they assert — *"every
/// byte in this file is one the suite put there"* — and they share one
/// property that turned out to matter more than any of them: **there is
/// nothing in them for a coincidence to hide in.**
///
/// `tools/ui-verify`'s `checks::redaction` — the one end-to-end check that
/// drives the real binary — generates its own fixture too, and its header
/// says what it is: *"Two pages, uncompressed"*, drawing two ASCII strings
/// with a Base-14 font. So the redaction feature had a unit suite and a
/// driven check and **neither had ever seen a document with an embedded
/// font in it**, which is to say neither had ever seen a document a person
/// would open.
///
/// The result was a feature that passed every test and refused every real
/// file. `fixtures/a1-titleblock.pdf` — a drawing sheet in this repository,
/// with JetBrains Mono embedded — refused with
/// `VerificationFailed { survivors: [" construction"] }` because the font's
/// `name` table describes its own ligatures as *"Classic construction"*.
///
/// ⇒ **The fixture that exercises the feature and the fixture that
/// resembles the operator's work are not the same fixture, and a suite
/// needs both.** This is the second.
///
/// # What it asserts, and the negative control
///
/// 1. the apply **is not refused** — the operator's complaint, as a
///    boolean;
/// 2. characters were actually **removed** (`glyphs_removed > 0`), so a
///    build that "passed" by doing nothing fails;
/// 3. the residual **is** disclosed and names
///    [`ResidualSite::FontProgram`] — not refusing must not mean not
///    telling;
/// 4. the file is **written** once the acknowledgement is given, which is
///    the operator's actual demand: *"still make the changes it could"*;
/// 5. ★ the negative control: `FOUNDATION`, a word on the same sheet that
///    was never marked, is **still extractable** from the written file. A
///    build that emptied the page would satisfy 1–4 and fail here.
#[test]
fn a_real_drawing_sheet_with_an_embedded_font_is_applied_rather_than_refused() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/a1-titleblock.pdf");
    let doc = Document::from_bytes(std::fs::read(&path).expect("the fixture is in the repo"))
        .expect("the fixture parses");
    let mut session = EditSession::new(doc);
    let created = session
        .mark_redactions_by_search("construction", false)
        .expect("the sheet's text is extractable");
    assert!(
        !created.is_empty(),
        "the fixture must contain the word, or nothing below is a test"
    );

    let prepared = prepare_redaction_apply(&session).unwrap_or_else(|refusal| {
        panic!(
            "★ THE OPERATOR'S COMPLAINT: a completed redaction on a real \
             drawing sheet was refused outright, so nothing was written and \
             there was no way to proceed — {refusal:?}"
        )
    });
    assert!(
        prepared.report.glyphs_removed > 0,
        "not refusing is worthless if nothing was removed"
    );
    assert_eq!(
        prepared.verification.residuals,
        vec![proof::Residual {
            text: " construction".to_owned(),
            site: proof::ResidualSite::FontProgram,
        }],
        "★★ the byte run IS in the file, inside the embedded font's own \
         name table, and the operator is owed that fact with the place \
         named — going quiet about it would be a worse defect than the \
         refusal was"
    );

    let target = scratch("a1-titleblock-redacted.pdf");
    let _ = std::fs::remove_file(&target);
    prepared
        .write_to(&target, ResidualAcknowledgement::Given)
        .expect("an acknowledged residual must not block the write");

    let written = Document::from_bytes(std::fs::read(&target).expect("the file exists"))
        .expect("the written file parses");
    let text = text_extract::extract_document(&written, &ExtractOptions::default())
        .expect("the written file's text is extractable")
        .pages
        .iter()
        .flat_map(|p| p.runs.iter().map(|r| r.text.clone()))
        .collect::<String>();
    assert!(
        !text.contains("construction"),
        "the marked word must be gone from the extracted text"
    );
    assert!(
        text.contains("FOUNDATION"),
        "★ the negative control: an unmarked word on the same sheet must \
         survive, or this suite would pass on a build that blanked the page"
    );
    let _ = std::fs::remove_file(&target);
}

/// ★★ **A test asserting an external limitation goes red when the
/// limitation lifts, and that red is a REPORT rather than a regression.**
/// It is also the only member of that family that behaves well: the prose
/// version of the same claim — in `text::redact`, in a UI string — went on
/// compiling and passing, and had to be corrected because the engine's reply
/// told us to. ⇒ Where a stale external claim can be spelled as an
/// assertion, spell it as one.
///
/// What it asserts now is the pair the operator cares about: the apply
/// SUCCEEDS, and the report says the image was dealt with rather than
/// quietly stepped over.
#[test]
fn a_region_over_an_image_destroys_the_samples_and_says_so() {
    let content = "q 200 0 0 100 20 20 cm /Im0 Do Q";
    let stream = format!(
        "<< /Length {} >>\nstream\n{content}\nendstream",
        content.len()
    );
    let image = "<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace \
                 /DeviceGray /BitsPerComponent 8 /Length 1 >>\nstream\n\x00\nendstream";
    let bytes = assemble(&[
        "<< /Type /Catalog /Pages 2 0 R >>",
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 200] \
         /Resources << /XObject << /Im0 5 0 R >> >> /Contents 4 0 R >>",
        &stream,
        image,
    ]);
    let doc = Document::from_bytes(bytes).unwrap();
    let mut session = EditSession::new(doc);
    session
        .add_redaction(
            0,
            &RedactSpec {
                quads: vec![Quad::from_rect(Rect::from_corners(30.0, 30.0, 150.0, 90.0))],
                fill: None,
                overlay_text: None,
                quadding: Quadding::Left,
            },
        )
        .unwrap();

    let prepared = prepare_redaction_apply(&session)
        .expect("a region over an image is applied, not refused, since pdfcer-core v0.26.0");
    let report = &prepared.report;
    assert!(
        report.images_cleared > 0 || report.images_removed > 0,
        "the region covers the only image on the page, so the report must say what happened to it — cleared or removed. Got cleared={} removed={} retained={}",
        report.images_cleared,
        report.images_removed,
        report.marks_retained
    );
    // ★ And the mark was APPLIED rather than retained. A retained mark is
    // the honest half-measure for an image the engine cannot decode; this
    // one is a 1x1 DeviceGray it certainly can, so a retention here would
    // mean the destroy path was not reached at all and the assertion above
    // was satisfied by something else.
    assert_eq!(
        report.marks_retained, 0,
        "a decodable image must not leave the mark unapplied"
    );
}

/// The census the review panel lists from and the census the status bar
/// counts from must be the same walk.
///
/// Asserted here because the shell reads both and a disagreement between
/// them is unresolvable from the operator's side.
#[test]
fn the_mark_list_and_the_mark_count_agree() {
    let session = session_with_unsaved_mark();
    let graph = session.graph();
    assert_eq!(
        redact::redaction_marks(&graph).len(),
        redact::count_redaction_marks(&graph)
    );
    let pages = page_tree::pages_in(&graph).unwrap();
    for mark in redact::redaction_marks(&graph) {
        assert!(
            mark.page_index < pages.len(),
            "a listed mark must name a real page"
        );
    }
}

// -- THE WRITE GATE -----------------------------------------------------

/// ★★ **A disclosed residual cannot be written past without an
/// acknowledgement, and the refusal leaves no file behind.**
///
/// §2.3, asserted rather than described. The dialog greys its confirm
/// control until the box is ticked, and **a greyed control is a drawing
/// decision, not a mechanism** — this is the mechanism. The failure it
/// catches is the one that matters most: a partially-redacted file handed
/// over as a complete one.
///
/// The fixture builds the residual by hand rather than hunting for a
/// document that happens to produce one, because the point under test is
/// the *gate*, not the classification (which [`proof`]'s own tests cover).
#[test]
fn an_unacknowledged_residual_refuses_the_write() {
    let session = session_with_unsaved_mark();
    let mut prepared = prepare_redaction_apply(&session).unwrap();
    assert!(
        prepared.verification.is_clean(),
        "the fixture must start clean, or the assertion below proves nothing"
    );
    prepared.verification.residuals.push(proof::Residual {
        text: "MARGARETHALE".to_owned(),
        site: proof::ResidualSite::RawBytes,
    });

    let target = scratch("unacknowledged.pdf");
    let _ = std::fs::remove_file(&target);
    let refusal = prepared
        .write_to(&target, ResidualAcknowledgement::Withheld)
        .expect_err("a withheld acknowledgement must refuse");
    assert!(
        matches!(
            refusal,
            WriteRefusal::ResidualsNotAcknowledged { residuals: 1 }
        ),
        "{refusal}"
    );
    assert!(
        !target.exists(),
        "a refused write must leave nothing behind at the path it was aimed at"
    );

    // …and the same value writes once the operator has acknowledged.
    prepared
        .write_to(&target, ResidualAcknowledgement::Given)
        .expect("an acknowledged residual may proceed");
    assert!(target.is_file());
    let _ = std::fs::remove_file(&target);
}

/// A clean report needs no acknowledgement, in either position.
///
/// The other direction of the gate, and the one that would make the feature
/// unusable if it were wrong: a redaction with nothing to disclose must not
/// demand a tick nobody can give.
#[test]
fn a_clean_report_writes_with_the_acknowledgement_withheld() {
    let session = session_with_unsaved_mark();
    let prepared = prepare_redaction_apply(&session).unwrap();
    for ack in [
        ResidualAcknowledgement::Withheld,
        ResidualAcknowledgement::Given,
    ] {
        let target = scratch("clean.pdf");
        let _ = std::fs::remove_file(&target);
        prepared
            .write_to(&target, ack)
            .expect("a clean redaction writes under either acknowledgement");
        assert!(target.is_file());
        let _ = std::fs::remove_file(&target);
    }
}

/// A write that cannot happen is reported rather than swallowed.
///
/// `crate::app::save`'s equivalent test, for the writer that matters more:
/// a redaction the operator believes landed, at a path that does not exist,
/// is a file they will look for and not find at the moment they need it.
#[test]
fn a_write_that_cannot_happen_is_a_named_refusal() {
    let session = session_with_unsaved_mark();
    let prepared = prepare_redaction_apply(&session).unwrap();
    let target = scratch("no-such-folder").join("nested").join("out.pdf");
    let refusal = prepared
        .write_to(&target, ResidualAcknowledgement::Withheld)
        .expect_err("a missing folder cannot be written to");
    assert!(matches!(refusal, WriteRefusal::FileSystem(_)), "{refusal}");
    assert!(!target.exists());
}

/// ★ **`{:?}` on a prepared redaction does not print the document.**
///
/// §2.1's hand-written [`std::fmt::Debug`], pinned. The failure it prevents
/// is silent and total: a `#[derive(Debug)]` restored during a routine
/// tidy-up would put a whole redacted PDF into any trace, panic or test
/// failure that formatted this value — a log file nobody thinks of as
/// containing document content.
#[test]
fn the_debug_impl_reports_a_length_rather_than_the_bytes() {
    let session = session_with_unsaved_mark();
    let prepared = prepare_redaction_apply(&session).unwrap();
    let rendered = format!("{prepared:?}");
    assert!(
        !rendered.contains("KEEPTHIS"),
        "the Debug impl emitted document content: {rendered}"
    );
    assert!(
        rendered.contains(&prepared.byte_len().to_string()),
        "…and it must still report the length, which is what a diagnostic \
         actually wants from that field: {rendered}"
    );
}

// ===========================================================================
// ★★★ THE DEFERRED ROUTE — `stage_into_session`, `save_applying_pending`,
// 2026-09-05, `pdfcer-core` `Pass 250.2`
//
// This section REPLACES the one that measured `apply_into_session`'s collapse
// (`Pass 250.1`), and the replacement is not a rename: the property under test
// is the opposite one.
//
//   * The collapse's safety claim was that an incremental save of a redacted
//     session is CLEAN, because there was no un-redacted base left. Those tests
//     performed the save and searched the result.
//   * The staging's safety claim is that an ordinary save is REFUSED BY NAME,
//     because the un-redacted content is still live in the session. These tests
//     perform the same save and assert it does not happen.
//
// ★★ Both are the same discipline: a guarantee stated in a doc comment is a
// claim about somebody else's code, and every one of these is the measurement.
// Each is written so the failure it is looking for makes it fail LOUDLY rather
// than vacuously — every assertion of absence is paired with a positive control
// that would catch a build which simply emptied the document.
//
// ★★★ And the leak surface is LARGER than the collapse's, which is why there
// are more of them. Under the collapse the un-redacted document was dropped the
// moment the operator confirmed; under staging it is still in memory, still in
// the file on disk, and still reachable by every verb that serialises a
// session. What stands between it and a file is one engine flag and this
// shell's routing.
// ===========================================================================

/// A session with one unsaved mark over the secret, already STAGED.
///
/// The state an operator is in between pressing the confirm control and
/// pressing Save, and the state every test below starts from.
fn staged_session() -> EditSession {
    let mut session = session_with_unsaved_mark();
    let staged = stage_into_session(&mut session).expect("the fixture must stage");
    assert!(staged.report.marks_applied >= 1);
    assert!(
        session.has_pending_redaction(),
        "the engine must say the removal is armed, or nothing below is a test"
    );
    session
}

/// ★★★ **THE HEADLINE: while a removal is staged, BOTH ordinary save modes are
/// refused BY NAME.**
///
/// This is `request_apply_redactions_into_the_session.md` §4.1 — the property
/// the request marked ★★★ and asked the engine to enforce by refusal. `Pass
/// 250.1` declined to refuse, on the argument that its collapse removed the
/// hazard at the root. `Pass 250.2` cannot make that argument, because it
/// preserves the un-redacted session on purpose, so it ships the refusal — and
/// this test is why that refusal is believed rather than quoted.
///
/// ★ **The leak is measured as well as the refusal.** It would be possible for
/// the engine to refuse `to_incremental_bytes` and not `to_full_bytes`, or to
/// refuse both and for this shell to be reaching for some third serialiser, so
/// the test does not stop at the error type: it asserts that **no bytes came
/// back at all** from either mode, which is the only form of "cannot leak"
/// that does not depend on reading the engine's source.
///
/// ★ The positive control is the fixture itself: the same session's staged save
/// path DOES produce bytes, in `the_staged_save_removes_the_text_and_leaves_no_prior_revision`
/// below. Without that, this test would pass on a build in which the session
/// could not be serialised by any means whatsoever.
#[test]
fn both_ordinary_save_modes_are_refused_by_name_while_staged() {
    use pdfcer_core::writer::WriteError;

    let session = staged_session();

    let incremental = session
        .to_incremental_bytes(&SaveOptions::default())
        .expect_err(
            "★★★ an INCREMENTAL save of a staged session must be refused. The \
             un-redacted content is still live in this session and an \
             incremental save would append a delta over it — which is the exact \
             /Prev leak R35 describes and the exact one our request asked to be \
             made impossible.",
        );
    assert!(
        matches!(incremental, WriteError::RedactionPending),
        "★★ and refused BY NAME. A generic write failure would be \
         indistinguishable from a broken document, and this shell's save path \
         branches on the answer: {incremental}"
    );

    let full = session.to_full_bytes(&SaveOptions::default()).expect_err(
        "★★ a FULL save of a staged session must be refused too. It would not \
         leak — a full rewrite carries no prior revision — but it would emit \
         the /Redact marks with the content still under them, which is an \
         UNAPPLIED redaction in a file the operator believes is redacted.",
    );
    assert!(
        matches!(full, WriteError::RedactionPending),
        "the full mode must refuse by the same name: {full}"
    );
}

/// ★★★ **The staged save removes the text, and leaves no prior revision to
/// walk back to.**
///
/// The other half of the headline, and the positive control for it: the save
/// that IS permitted while a removal is armed must actually produce bytes, must
/// not contain the removed text, and must be a single revision.
///
/// The `/Prev` assertion is the sharp one. A staged save is a full rewrite by
/// construction (`crate::redact::save_applying_pending`), so a `/Prev` in the
/// trailer would mean the un-redacted document is reachable one `startxref` hop
/// away in a file this shell has told the operator is redacted — R35's whole
/// point.
///
/// ★ The scan is over the **raw bytes** rather than over decoded streams, and
/// on this fixture that is legitimate: the content stream is uncompressed and
/// the font is Base-14, so there is no encoding under which the text could be
/// present-but-unfindable, and no font program in which it could be
/// present-but-innocent. `a_real_drawing_survives_the_staged_route` answers the
/// compressed, embedded-font case.
#[test]
fn the_staged_save_removes_the_text_and_leaves_no_prior_revision() {
    let session = staged_session();
    let (bytes, report) = save_applying_pending(&session, &SaveOptions::default())
        .expect("the staged save is the one save that must work");

    assert!(
        report.redacted_text.iter().any(|t| t == SECRET),
        "the engine must say it removed the secret, or this test is checking \
         the absence of a string nobody claimed to remove: {:?}",
        report.redacted_text
    );
    assert!(
        !proof::contains(&bytes, SECRET.as_bytes()),
        "★★★ the removed text survived in the bytes the operator is about to \
         receive"
    );
    assert_eq!(
        proof::survivors_in_content_streams(&bytes, &[SECRET.to_owned()]),
        None,
        "…and it survived in a decoded stream of them"
    );
    assert!(
        proof::contains(&bytes, b"KEEPTHIS"),
        "★ the positive control: un-marked text must survive, or this suite \
         passes on a build that emptied the page"
    );
    assert!(
        Document::from_bytes(bytes)
            .expect("the staged save must re-parse")
            .trailer()
            .get(b"Prev")
            .is_none(),
        "★★ a staged save is a full rewrite: a /Prev would put the \
         un-redacted document one startxref hop away in a file pdfcer has \
         called redacted"
    );
}

/// ★★★ **Staging preserves the undo log — the whole reason `Pass 250.2`
/// exists.**
///
/// The route this replaced cleared the log outright, and the operator accepted
/// that with a *"for now"* attached. This is the assertion that the *for now*
/// is over, and it is deliberately three separate claims because a build that
/// had silently reverted to the collapsing verb would fail a different one of
/// them depending on how it reverted:
///
/// 1. **the depth is unchanged** — the log is not merely non-empty, it is the
///    same size it was before the staging;
/// 2. **an undo actually works** and takes the marks back off, which is the
///    operator-visible form of the same claim;
/// 3. **`has_applied_redaction()` is false** — this shell never collapses, and
///    if that verb ever answers true here, something is calling the engine's
///    other apply and `redact::sealed`'s count has been wrong.
#[test]
fn staging_preserves_the_undo_log() {
    let mut session = session_with_unsaved_mark();
    let before = session.undo_depth();
    assert!(
        before > 0,
        "the fixture must have something in the log, or this test cannot fail"
    );

    let staged = stage_into_session(&mut session).expect("the fixture stages");

    assert_eq!(
        session.undo_depth(),
        before,
        "★★★ the undo log lost a step. That is `Pass 250.1`'s collapsing verb \
         behaving, and this shell must not be calling it — check \
         `redact::sealed`'s counts before anything else."
    );
    assert_eq!(
        staged.undo_depth_preserved, before,
        "the reported depth must be the real one, read after the call"
    );
    assert!(
        !session.has_applied_redaction(),
        "★ nothing in this shell collapses a session any more. A `true` here \
         means the engine's finalizing verb was reached by some route the \
         call-site monopoly did not see."
    );
    assert!(session.has_pending_redaction(), "…and the removal is armed");

    // (2) the operator-visible form: an undo takes the marks back off.
    session.undo().expect("the mark must be undoable");
    assert_eq!(
        redact::count_redaction_marks(&session.graph()),
        0,
        "★★ undoing after a staging must reach the marks. This is the \
         capability the whole pass was for, and a build in which the log \
         survived but no longer described the document would pass claim (1) \
         and fail here."
    );
}

/// ★★★ **A staged removal can be called off, and calling it off restores
/// ordinary saving.**
///
/// *A stageable operation that cannot be un-staged is a trap*, asserted. The
/// trap has teeth here rather than being a matter of taste: while a removal is
/// armed the engine refuses both ordinary save modes, so an operator who
/// changed his mind and had no way to say so could not save his document at
/// all.
///
/// The second assertion is the one that makes the first mean something. A
/// cancel that cleared the flag and left the session unable to serialise would
/// satisfy *"the flag is off"* and leave him exactly where he was.
///
/// ★ And the third: the marks survive. Un-arming is not un-marking, and a
/// cancel that silently removed the operator's marks would destroy work while
/// claiming to be the safe button.
#[test]
fn a_staged_removal_can_be_called_off_and_saving_works_again() {
    let mut session = staged_session();
    let marks_before = redact::count_redaction_marks(&session.graph());
    assert!(marks_before > 0);

    cancel_staged_redaction(&mut session);

    assert!(
        !session.has_pending_redaction(),
        "the flag must be off, or the control does nothing"
    );
    let (bytes, _) = session
        .to_incremental_bytes(&SaveOptions::default())
        .expect(
            "★★★ an ordinary save must work again. A cancel that cleared the \
             flag and left the document unsaveable would leave the operator \
             exactly where he was, with a control that claimed to help.",
        );
    assert!(
        proof::contains(&bytes, SECRET.as_bytes()),
        "★★ and the content is still there, which is CORRECT and is the point \
         of the control: he called the removal off. A build that removed it \
         anyway would pass every 'is the flag clear' assertion and destroy the \
         content he decided to keep."
    );
    assert_eq!(
        redact::count_redaction_marks(&session.graph()),
        marks_before,
        "★ un-arming is not un-marking. The marks are the operator's work and \
         a cancel that took them off would destroy it while claiming to be the \
         safe button."
    );

    // Idempotent, because a control may be reached from a stale frame.
    cancel_staged_redaction(&mut session);
    assert!(!session.has_pending_redaction());
}

/// ★★★ **A staged document with NO MARKS LEFT can still be called off — the
/// trap this feature would otherwise close on the operator.**
///
/// The sequence, and every step of it is something a reasonable person does:
///
/// 1. mark, then *Review & apply* ▸ *this document* — the removal is armed;
/// 2. change his mind about the marks and take them off in the panel, one
///    Remove at a time (not an undo — an ordinary edit);
/// 3. press `Ctrl+S`.
///
/// The save is refused, because the armed removal has nothing to remove and
/// the engine refuses both ordinary modes while it stands. So he goes back to
/// *Review & apply* to call the removal off — **and if the pipeline asked the
/// mark census before the pending flag, he would be told `NothingToApply`,
/// which the dialog draws as a refusal with no control on it.** The document
/// would then be unsaveable by every route in the program, with the one button
/// that frees him behind a phase he cannot reach.
///
/// ★ Two things keep it open and both are asserted here: `AlreadyStaged` is
/// answered ahead of the census, and the command that opens the window is
/// `enabled_when("doc.pages")` rather than on a marks predicate — so the
/// ribbon control stays live on a document with none.
#[test]
fn a_staged_document_with_no_marks_left_can_still_be_called_off() {
    let mut session = staged_session();

    // Take the marks off the ordinary way — a delete, not an undo, because
    // that is the route the panel offers and it leaves the arming standing.
    for mark in redact::redaction_marks(&session.graph()) {
        session
            .delete_redaction_mark(mark.annot_id)
            .expect("the panel's Remove must work while a removal is armed");
    }
    assert_eq!(redact::count_redaction_marks(&session.graph()), 0);
    assert!(session.has_pending_redaction(), "…and the arming stands");

    assert_eq!(
        prepare_redaction_apply(&session).unwrap_err(),
        RedactApplyRefusal::AlreadyStaged,
        "★★★ NOT `NothingToApply`. The dialog draws that one as a refusal with \
         no control on it, and this document cannot be saved by any other \
         route — so the operator would be stuck with an armed removal, no way \
         to save, and no way to call it off."
    );

    // And the way out really does work.
    cancel_staged_redaction(&mut session);
    session
        .to_incremental_bytes(&SaveOptions::default())
        .expect("an ordinary save must work again once the removal is off");
}

/// ★★ **Staging twice is refused by name rather than silently re-arming.**
///
/// Two reachable causes and one refusal: the operator opens *Review & apply* a
/// second time on a document he has already staged, or a second `Stage` action
/// arrives before the first frame after the first one.
///
/// ★ The refusal has to be **this shell's**, not the engine's, and that is the
/// assertion. `EditSession::apply_redactions_deferred` would happily run a
/// second preview and set an already-set flag; what makes the second open
/// legible is `prepare_redaction_apply` naming the state, because otherwise the
/// engine's `to_full_bytes` refusal would surface as *"this document cannot be
/// rewritten in full"* — a true sentence about the wrong subject, at the one
/// surface where a wrong diagnosis costs most.
#[test]
fn a_second_staging_and_a_second_report_are_both_refused_by_name() {
    let mut session = staged_session();
    assert_eq!(
        stage_into_session(&mut session).unwrap_err(),
        RedactApplyRefusal::AlreadyStaged
    );
    assert_eq!(
        prepare_redaction_apply(&session).unwrap_err(),
        RedactApplyRefusal::AlreadyStaged,
        "★★ the dialog reopened on a staged document must be told WHICH state \
         it is in. Without this it reaches `to_full_bytes`, which the engine \
         refuses while a removal is armed, and the operator reads that a \
         document that can be rewritten cannot be."
    );
}

/// ★ **A refused staging leaves the session exactly as it was.**
///
/// The engine's own guarantee — *"on any error the pending flag is NOT set"* —
/// asserted from this side rather than quoted. `NothingToApply` is the one
/// refusal a test can produce without breaking the engine, and it is also the
/// one this shell can actually reach (a mark undone in the frame between the
/// panel enabling its button and the action running).
#[test]
fn a_refused_staging_leaves_the_session_untouched() {
    let doc = Document::from_bytes(secret_pdf()).unwrap();
    let mut session = EditSession::new(doc);
    let err = stage_into_session(&mut session).expect_err("no marks, no staging");
    assert_eq!(err, RedactApplyRefusal::NothingToApply);
    assert!(
        !session.has_pending_redaction(),
        "a refusal must not leave the session armed"
    );
    let (bytes, _) = session
        .to_incremental_bytes(&SaveOptions::default())
        .expect("…and the ordinary save must still work");
    assert!(
        proof::contains(&bytes, SECRET.as_bytes()),
        "★ nothing was removed, and the document must still say so"
    );
}

/// ★★★ **The staging survives the save, and every later save applies it
/// again.**
///
/// `save_applying_redaction` takes `&self`. It does not mutate the session and
/// it does not clear the flag, so a saved document is still armed — which is
/// the fact `crate::text::redact::saved_applying_redaction` tells the operator
/// and which nothing else on screen would.
///
/// The reason it is a test rather than a sentence is the assumption it
/// contradicts: *"I saved it, so it is done."* A build that cleared the flag on
/// save would look correct for one save and then quietly write the un-redacted
/// document on the second one — with no refusal, because the flag would be off.
///
/// ★ Undo across the save is asserted in the same test, because the two facts
/// have the same cause (`&self`) and a build that broke one would break both.
#[test]
fn the_staging_and_the_undo_log_both_survive_the_save() {
    let mut session = staged_session();
    let depth = session.undo_depth();

    let (first, _) = save_applying_pending(&session, &SaveOptions::default()).expect("first save");
    assert!(!proof::contains(&first, SECRET.as_bytes()));

    assert!(
        session.has_pending_redaction(),
        "★★★ the removal must still be armed after the save. A build that \
         cleared it here would write the un-redacted document on the NEXT \
         save, with no refusal, because the flag it depends on would be off."
    );
    assert_eq!(
        session.undo_depth(),
        depth,
        "★ and undo survived the save, which is what `&self` buys"
    );

    // …and a second save is still clean, over a session that has been edited
    // since the first one.
    session
        .rotate_pages(&[0], 90)
        .expect("an ordinary edit must still work while a removal is armed");
    let (second, _) =
        save_applying_pending(&session, &SaveOptions::default()).expect("second save");
    assert!(
        !proof::contains(&second, SECRET.as_bytes()),
        "★★ the removal re-runs over the CURRENT state, so an edit between \
         the two saves must not carry the removed text back into the file"
    );
    assert!(
        proof::contains(&second, b"KEEPTHIS"),
        "positive control, again: the document was not emptied"
    );
    assert_ne!(
        first, second,
        "★ the control for the assertion above: the second save really did \
         reflect the edit, so the absence check over it is measuring a \
         different document rather than the same bytes twice"
    );
}

/// ★★★ **A REAL drawing survives the staged route** —
/// `fixtures/a1-titleblock.pdf`.
///
/// Every other fixture in this file is uncompressed with a Base-14 font, which
/// is a document with nothing for a coincidence to hide in. This one is a CAD
/// title block with compressed content streams and an embedded, subsetted
/// font — and it is the file whose font `name` table describes its ligatures as
/// *"Classic construction"*, which is what made the shell refuse every real
/// redaction until the proof was corrected on 2026-09-04.
///
/// It asserts through pdfcer's own text extraction as well as through the raw
/// bytes, because on a compressed document the raw scan alone would pass on a
/// build that had not removed anything at all — and it asserts the refusal of
/// the ordinary modes on the same document, because the leak surface this pass
/// introduces is the un-redacted base, and a compressed real document is where
/// a partial guard would hide.
#[test]
fn a_real_drawing_survives_the_staged_route() {
    use pdfcer_core::text_extract::{self, ExtractOptions};
    use pdfcer_core::writer::WriteError;

    // ui-text-exempt: a word in a test fixture, matched against extracted text.
    const TERM: &str = "FOUNDATION";
    // ui-text-exempt: a word in a test fixture that must SURVIVE.
    const KEEP: &str = "DRAWING NO";

    let source =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/a1-titleblock.pdf");
    let doc = Document::from_bytes(std::fs::read(&source).expect("the fixture must be readable"))
        .expect("the fixture must parse");
    let mut session = EditSession::new(doc);
    let created = session
        .mark_redactions_by_search(TERM, false)
        .expect("the drawing's text is extractable");
    assert!(
        !created.is_empty(),
        "the fixture must contain {TERM}, or this test proves nothing"
    );

    let staged = stage_into_session(&mut session).expect("a real drawing must stage");
    assert!(staged.report.glyphs_removed > 0);

    // ★ The refusal, on a real document. This is the assertion that the guard
    // is not a property of a four-object synthetic fixture.
    assert!(
        matches!(
            session.to_incremental_bytes(&SaveOptions::default()),
            Err(WriteError::RedactionPending)
        ),
        "★★★ an ordinary incremental save of a staged REAL drawing must be \
         refused. The un-redacted content is compressed inside this file and a \
         raw scan of an appended revision would not have found it."
    );

    let (bytes, _) = save_applying_pending(&session, &SaveOptions::default())
        .expect("and the staged save must work");
    assert!(
        !proof::contains(&bytes, TERM.as_bytes()),
        "the redacted term survived in the raw bytes of a real drawing"
    );
    let back = Document::from_bytes(bytes).expect("the saved drawing must re-parse");
    let text: String = text_extract::extract_document(&back, &ExtractOptions::default())
        .expect("extract")
        .pages
        .iter()
        .flat_map(|p| p.runs.iter())
        .map(|r| r.text.clone())
        .collect::<Vec<_>>()
        .join("|");
    assert!(
        !text.contains(TERM),
        "the redacted term was recoverable by extract-text: {text}"
    );
    assert!(
        text.contains(KEEP),
        "★ the positive control, and on a compressed document it is the \
         assertion that does the work: a build that removed the whole page \
         would satisfy every absence check above. Extracted: {text}"
    );
}

/// ★★ **The save-time proof costs nothing on an ordinary save and bites on a
/// planted leak.**
///
/// [`super::prove_saved_bytes`] is the check `crate::app::save` runs between
/// the bytes and the syscall on every save, and
/// [`super::save_applying_pending`] runs it once more before handing the bytes
/// over. It is expected to pass forever, and a check that is expected to pass
/// is exactly the kind this project keeps finding was never wired. So it is
/// falsified in both directions: an empty claim list returns `Ok` without
/// decoding anything, and a claim that IS present in a decoded stream comes
/// back as a survivor.
#[test]
fn the_save_time_proof_is_free_when_there_is_nothing_to_prove_and_bites_when_there_is() {
    let unredacted = secret_pdf();

    // The ordinary save: no claims, no work, no refusal — asserted against
    // bytes that DO contain the string, so a build that ignored the empty list
    // and scanned anyway would fail here rather than pass silently.
    assert!(
        proof::contains(&unredacted, SECRET.as_bytes()),
        "the fixture must contain the string for the next assertion to mean \
         anything"
    );
    assert_eq!(prove_saved_bytes(&unredacted, &[]), Ok(()));

    // The planted leak: the same bytes, now claimed to have been redacted.
    let survivors = prove_saved_bytes(&unredacted, &[SECRET.to_owned()])
        .expect_err("★ a claim that is still in a decoded content stream is a leak");
    assert_eq!(survivors, vec![SECRET.to_owned()]);

    // And the real article passes.
    let session = staged_session();
    let (saved, report) = save_applying_pending(&session, &SaveOptions::default()).unwrap();
    assert_eq!(
        prove_saved_bytes(&saved, &report.redacted_text),
        Ok(()),
        "a genuinely redacted save must not be refused"
    );
}

/// ★ **The two residual derivations cannot drift, and the deferred one counts
/// no verification.**
///
/// `crate::dialogs::redact::residual_lines` builds the list the operator
/// acknowledges; [`super::residual_count`] produces the number the staged
/// outcome sentence quotes. They must count the same things, and the
/// differences — promotion, which only the write-now route can observe, and the
/// absence proof, which the staging route has not run — are pinned here rather
/// than left to be rediscovered.
///
/// ★★ The `None` case is the important half and it is a claim rather than a
/// convenience: the staging verb discards its bytes, so no sweep has run, and a
/// caller passing a default `AbsenceVerification` would have told the operator
/// that one had and found nothing.
#[test]
fn the_residual_count_matches_the_disclosed_list_except_for_promotion() {
    let session = session_with_unsaved_mark();
    let mut prepared = prepare_redaction_apply(&session).expect("apply");
    assert_eq!(
        residual_count(&prepared.report, Some(&prepared.verification)),
        0,
        "the fixture has nothing to disclose"
    );

    prepared.verification.residuals.push(Residual {
        text: "MARGARETHALE".to_owned(),
        site: ResidualSite::RawBytes,
    });
    assert_eq!(
        residual_count(&prepared.report, Some(&prepared.verification)),
        1,
        "a raw-byte residual is counted by both"
    );
    assert_eq!(
        residual_count(&prepared.report, None),
        0,
        "★★ …and NOT counted when no sweep has run. The staged route passes \
         `None` because the engine's staging verb discards the bytes there \
         would have been to sweep; a default verification would have said a \
         sweep found nothing, which is a different claim from nobody having \
         looked."
    );

    prepared
        .promoted_by_materialisation
        .push(pdfcer_core::object::ObjId {
            num: 7,
            generation: 0,
        });
    assert_eq!(
        residual_count(&prepared.report, Some(&prepared.verification)),
        1,
        "★ promotion is the ONE thing the count does not include, because the \
         staged route has no materialisation step of its own to observe it \
         in. The dialog's own test asserts the other half — that its list is \
         one longer."
    );
}
