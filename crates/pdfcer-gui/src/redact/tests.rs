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
//! below goes through [`super::prepare_redaction_apply`] or
//! [`super::apply_into_session`], which is exactly the property the monopoly
//! exists to keep true of test code as well as of production code.

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
// ★★★ THE DEFERRED ROUTE — `apply_into_session`, 2026-09-04
//
// `Pass 250.1` shipped `EditSession::apply_redactions`, which applies a
// redaction INTO the open session and leaves the write to the ordinary save
// verbs. Everything below exists because that verb declines to do the one
// thing this shell's engine request asked for by name — refuse an incremental
// save — and offers a different guarantee instead: that after the collapse
// there is no un-redacted base left for any save mode to leak.
//
// ★★ A guarantee stated in a doc comment is a claim about somebody else's
// code. These tests are the measurement. Each one is written so that the
// failure it is looking for makes it fail LOUDLY rather than making it vacuous
// — every assertion of absence is paired with a positive control that would
// catch a build which simply emptied the document.
// ===========================================================================

/// ★★★ **THE HEADLINE: an incremental save of a redacted session cannot leak
/// the removed text.**
///
/// This is `request_apply_redactions_into_the_session.md` §4.1, the property
/// the request marked ★★★ and asked the engine to enforce by refusal. The
/// engine refused to refuse, on the argument that the hazard is gone at the
/// root. This test is why that argument is believed.
///
/// Three states are measured, in the order an operator reaches them:
///
/// 1. **Straight after the apply.** `to_incremental_bytes` must contain none of
///    the removed text, and — the sharper assertion — the output must carry no
///    `/Prev` at all, because the collapsed session's dirty set is empty and
///    there is nothing to append.
/// 2. **After a further ordinary edit.** Now there IS an appended revision and
///    a `/Prev`, which is exactly the shape the request feared. The revision it
///    points back to is the **redacted** base, so the removed text is still
///    absent from the whole file — and the later edit really is in it.
/// 3. **The positive control.** `KEEPTHIS`, never marked, survives all of it.
///    Without this a build that emitted an empty page would pass every absence
///    assertion above.
///
/// ★ The scan is over the **raw bytes** rather than over decoded streams, and
/// on this fixture that is legitimate: the content stream is uncompressed and
/// the font is Base-14, so there is no encoding under which the text could be
/// present-but-unfindable, and no font program in which it could be
/// present-but-innocent. `a_real_drawing_survives_the_deferred_route` is the
/// one that answers the compressed, embedded-font case.
#[test]
fn an_incremental_save_of_a_redacted_session_cannot_leak_the_removed_text() {
    let mut session = session_with_unsaved_mark();
    let applied = apply_into_session(&mut session).expect("the deferred apply must succeed");
    assert!(applied.report.marks_applied >= 1);
    assert!(
        applied.report.redacted_text.iter().any(|t| t == SECRET),
        "the engine must say it removed the secret, or this test is checking \
         the absence of a string nobody claimed to remove: {:?}",
        applied.report.redacted_text
    );

    // -- 1. straight after the apply -------------------------------------
    let (fresh, _) = session
        .to_incremental_bytes(&SaveOptions::default())
        .expect("an incremental save of a redacted session must be possible");
    assert!(
        !proof::contains(&fresh, SECRET.as_bytes()),
        "★★★ the removed text survived an INCREMENTAL save of the redacted \
         session. The engine's whole answer to our §4.1 was that the collapse \
         leaves no un-redacted base for a save mode to leak; if this fires, it \
         does not, and the deferred route must not ship."
    );
    assert!(
        Document::from_bytes(fresh.clone())
            .expect("the incremental output must re-parse")
            .trailer()
            .get(b"Prev")
            .is_none(),
        "immediately after the collapse the dirty set is empty, so there is \
         nothing to append and no prior revision to point at"
    );
    assert!(
        proof::contains(&fresh, b"KEEPTHIS"),
        "positive control: un-marked text must survive the apply"
    );

    // -- 2. after a further ordinary edit ---------------------------------
    session
        .rotate_pages(&[0], 90)
        .expect("an ordinary edit must still work after a redaction");
    let (appended, _) = session
        .to_incremental_bytes(&SaveOptions::default())
        .expect("and so must an incremental save of it");
    let back = Document::from_bytes(appended.clone()).expect("it must re-parse");
    assert!(
        back.trailer().get(b"Prev").is_some(),
        "★ the control for the assertion below: this save really did append a \
         revision, so the file really does contain a prior one. Without this, \
         the absence check underneath would be measuring a single-revision \
         file and proving nothing about /Prev at all."
    );
    assert!(
        !proof::contains(&appended, SECRET.as_bytes()),
        "★★★ the removed text is recoverable from the PRIOR REVISION of an \
         incrementally-saved redacted document. This is the exact leak R35 \
         describes and the exact one the request asked to be made impossible."
    );
    assert!(
        proof::contains(&appended, b"KEEPTHIS"),
        "positive control, again: the document was not emptied"
    );
}

/// ★★ **Both save modes of a redacted session are clean, and they agree.**
///
/// The companion assertion to the headline. `to_full_bytes` is the mode the
/// write-now route uses and has always been safe; the point of running both
/// here is that a build in which only one of them was safe would still pass a
/// test that checked only the other, and the shell's ordinary save verbs use
/// the incremental one.
#[test]
fn both_save_modes_of_a_redacted_session_are_clean() {
    let mut session = session_with_unsaved_mark();
    apply_into_session(&mut session).expect("apply");
    for (label, bytes) in [
        (
            "incremental",
            session
                .to_incremental_bytes(&SaveOptions::default())
                .unwrap()
                .0,
        ),
        (
            "full",
            session.to_full_bytes(&SaveOptions::default()).unwrap().0,
        ),
    ] {
        assert!(
            !proof::contains(&bytes, SECRET.as_bytes()),
            "the {label} save of a redacted session contains the removed text"
        );
        assert_eq!(
            proof::survivors_in_content_streams(&bytes, &[SECRET.to_owned()]),
            None,
            "the {label} save has the removed text in a decoded stream"
        );
    }
}

/// ★★★ **A REAL drawing survives the deferred route** —
/// `fixtures/a1-titleblock.pdf`.
///
/// The redaction path was, until 2026-09-04, *"effectively never tested end to
/// end"*: every other fixture in this file is uncompressed with a Base-14 font,
/// which is a document with nothing for a coincidence to hide in. This one is a
/// CAD title block with compressed content streams and an embedded, subsetted
/// font — and it is the file whose font `name` table describes its ligatures as
/// *"Classic construction"*, which is what made the shell refuse every real
/// redaction until the proof was corrected that morning.
///
/// It asserts through pdfcer's own text extraction as well as through the raw
/// bytes, because on a compressed document the raw scan alone would pass on a
/// build that had not removed anything at all.
#[test]
fn a_real_drawing_survives_the_deferred_route() {
    use pdfcer_core::text_extract::{self, ExtractOptions};

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

    let applied = apply_into_session(&mut session).expect("a real drawing must apply");
    assert!(applied.report.glyphs_removed > 0);

    let (bytes, _) = session
        .to_incremental_bytes(&SaveOptions::default())
        .expect("and must save");
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

/// ★★★ **The apply clears the undo log, and says by how much.**
///
/// The engine's verb *finalizes*, and the operator's ruling was that this is
/// acceptable **because it is disclosed**. This test pins the number the
/// disclosure is built from to the number the engine actually destroys: a build
/// where `undo_steps_cleared` was read after the call instead of before would
/// report 0 on every run, and the sentence would say nothing was lost on
/// exactly the runs where something was.
#[test]
fn the_deferred_apply_clears_the_undo_log_and_reports_how_much() {
    let mut session = session_with_unsaved_mark();
    let before = session.undo_depth();
    assert!(
        before > 0,
        "the fixture must have something in the log, or this test cannot fail"
    );
    let applied = apply_into_session(&mut session).expect("apply");
    assert_eq!(
        applied.undo_steps_cleared, before,
        "the disclosed count must be what was destroyed, measured before the call"
    );
    assert_eq!(
        session.undo_depth(),
        0,
        "the engine finalizes: nothing is left to step back to"
    );
    assert!(
        session.has_applied_redaction(),
        "and the session says so, which is what the shell's unsaved-edits \
         predicate reads"
    );
}

/// ★ **A refused apply leaves the session exactly as it was.**
///
/// The engine's own guarantee — *"the session is left UNCHANGED on any error,
/// so a failed apply never half-redacts"* — asserted from this side rather than
/// quoted. `NothingToApply` is the one refusal a test can produce without
/// breaking the engine, and it is also the one this shell can actually reach
/// (a mark undone in the frame between the panel enabling its button and the
/// action running).
#[test]
fn a_refused_deferred_apply_leaves_the_session_untouched() {
    let doc = Document::from_bytes(secret_pdf()).unwrap();
    let mut session = EditSession::new(doc);
    let err = apply_into_session(&mut session).expect_err("no marks, no apply");
    assert_eq!(err, RedactApplyRefusal::NothingToApply);
    assert!(
        !session.has_applied_redaction(),
        "a refusal must not leave the session flagged as redacted"
    );
    let (bytes, _) = session.to_full_bytes(&SaveOptions::default()).unwrap();
    assert!(
        proof::contains(&bytes, SECRET.as_bytes()),
        "★ nothing was removed, and the document must still say so. A build \
         that half-applied on the way to a refusal would fail here."
    );
}

/// ★★ **The save-time proof costs nothing on an ordinary save and bites on a
/// planted leak.**
///
/// [`super::prove_saved_bytes`] is the check `crate::app::save` runs between
/// the bytes and the syscall on every save of a redacted document. It is
/// expected to pass forever — the engine's collapse makes it so — and a check
/// that is expected to pass is exactly the kind this project keeps finding was
/// never wired. So it is falsified here in both directions: an empty claim list
/// returns `Ok` without decoding anything, and a claim that IS present in a
/// decoded stream comes back as a survivor.
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
    let mut session = session_with_unsaved_mark();
    let applied = apply_into_session(&mut session).expect("apply");
    let (saved, _) = session
        .to_incremental_bytes(&SaveOptions::default())
        .unwrap();
    assert_eq!(
        prove_saved_bytes(&saved, &applied.report.redacted_text),
        Ok(()),
        "a genuinely redacted save must not be refused"
    );
}

/// ★ **The two residual derivations cannot drift.**
///
/// `crate::dialogs::redact::residual_lines` builds the list the operator
/// acknowledges; [`super::residual_count`] produces the number the deferred
/// outcome sentence quotes. They must count the same things, and the one
/// difference — promotion, which only the write-now route can observe — is
/// pinned here rather than left to be rediscovered.
#[test]
fn the_residual_count_matches_the_disclosed_list_except_for_promotion() {
    let session = session_with_unsaved_mark();
    let mut prepared = prepare_redaction_apply(&session).expect("apply");
    assert_eq!(
        residual_count(&prepared.report, &prepared.verification),
        0,
        "the fixture has nothing to disclose"
    );

    prepared.verification.residuals.push(Residual {
        text: "MARGARETHALE".to_owned(),
        site: ResidualSite::RawBytes,
    });
    assert_eq!(
        residual_count(&prepared.report, &prepared.verification),
        1,
        "a raw-byte residual is counted by both"
    );

    prepared
        .promoted_by_materialisation
        .push(pdfcer_core::object::ObjId {
            num: 7,
            generation: 0,
        });
    assert_eq!(
        residual_count(&prepared.report, &prepared.verification),
        1,
        "★ promotion is the ONE thing the count does not include, because the \
         deferred route has no materialisation step of its own to observe it \
         in. The dialog's own test asserts the other half — that its list is \
         one longer."
    );
}
